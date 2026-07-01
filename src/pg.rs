#![allow(dead_code)]

use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls, Row};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub user: String,
    pub ssl_mode: String,
}

impl ConnectionProfile {
    pub fn local_dev() -> Self {
        Self {
            id: "local-dev".into(),
            name: "local-dev".into(),
            host: "localhost".into(),
            port: 5432,
            database: Some("app_db".into()),
            user: "postgres".into(),
            ssl_mode: "disable".into(),
        }
    }

    fn connection_string(&self) -> String {
        let database = self.database.as_deref().unwrap_or("postgres");
        format!(
            "host={} port={} user={} password=postgres dbname={}",
            self.host, self.port, self.user, database
        )
    }
}

#[derive(Clone, Debug)]
pub enum CatalogNodeKind {
    Database,
    Schema,
    Folder,
    Table,
}

#[derive(Clone, Debug)]
pub struct CatalogNode {
    pub name: String,
    pub kind: CatalogNodeKind,
}

impl CatalogNode {
    pub fn database(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Database,
        }
    }

    pub fn schema(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Schema,
        }
    }

    pub fn folder(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Folder,
        }
    }

    pub fn table(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Table,
        }
    }
}

pub enum PgCommand {
    Connect(ConnectionProfile),
    Execute { sql: String },
    Cancel,
    LoadCatalogNode { node_id: String },
}

#[derive(Clone, Debug)]
pub enum PgEvent {
    Connected {
        profile_id: String,
    },
    Disconnected,
    CatalogNodeLoaded {
        parent_id: String,
        nodes: Vec<CatalogNode>,
    },
    QueryStarted,
    QueryResult {
        columns: Vec<ResultColumn>,
        rows: Vec<Vec<String>>,
        elapsed_ms: u128,
    },
    QueryCompleted {
        rows: usize,
        elapsed_ms: u128,
    },
    QueryFailed(String),
    Notice(String),
    TransactionStatusChanged(String),
}

impl PgEvent {
    pub fn summary(&self) -> String {
        match self {
            PgEvent::Connected { profile_id } => format!("connected to {profile_id}"),
            PgEvent::Disconnected => "disconnected".into(),
            PgEvent::CatalogNodeLoaded { nodes, .. } => {
                format!("loaded {} catalog objects", nodes.len())
            }
            PgEvent::QueryStarted => "query started".into(),
            PgEvent::QueryResult {
                rows, elapsed_ms, ..
            } => format!("query returned {} rows in {elapsed_ms} ms", rows.len()),
            PgEvent::QueryCompleted { rows, elapsed_ms } => {
                format!("query completed: {rows} rows in {elapsed_ms} ms")
            }
            PgEvent::QueryFailed(error) => format!("query failed: {error}"),
            PgEvent::Notice(notice) => notice.clone(),
            PgEvent::TransactionStatusChanged(status) => format!("transaction: {status}"),
        }
    }
}

pub struct PgRuntime {
    runtime: Runtime,
    commands_tx: mpsc::UnboundedSender<PgCommand>,
}

impl PgRuntime {
    pub fn new() -> anyhow::Result<(Self, mpsc::UnboundedReceiver<PgEvent>)> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("loxql-pg")
            .build()?;
        let (commands_tx, commands_rx) = mpsc::unbounded_channel();
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        runtime.spawn(pg_worker(commands_rx, events_tx));

        Ok((
            Self {
                runtime,
                commands_tx,
            },
            events_rx,
        ))
    }

    pub fn spawn_command(&self, command: PgCommand) {
        let _ = self.commands_tx.send(command);
    }
}

async fn pg_worker(
    mut commands_rx: mpsc::UnboundedReceiver<PgCommand>,
    events_tx: mpsc::UnboundedSender<PgEvent>,
) {
    let mut client: Option<Client> = None;

    while let Some(command) = commands_rx.recv().await {
        match command {
            PgCommand::Connect(profile) => {
                let connect_result =
                    tokio_postgres::connect(&profile.connection_string(), NoTls).await;

                match connect_result {
                    Ok((pg_client, connection)) => {
                        tokio::spawn(async move {
                            if let Err(error) = connection.await {
                                eprintln!("postgres connection failed: {error}");
                            }
                        });

                        client = Some(pg_client);
                        let _ = events_tx.send(PgEvent::Connected {
                            profile_id: profile.id.clone(),
                        });
                        load_catalog(client.as_ref(), &events_tx).await;
                    }
                    Err(error) => {
                        let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    }
                }
            }
            PgCommand::Execute { sql } => {
                let Some(pg_client) = client.as_ref() else {
                    let _ = events_tx.send(PgEvent::QueryFailed(
                        "Not connected. Select local-dev first.".into(),
                    ));
                    continue;
                };

                if let Err(error) = pg_query::parse(&sql) {
                    let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    continue;
                }

                let _ = events_tx.send(PgEvent::QueryStarted);
                let started = Instant::now();

                match pg_client.query(sql.as_str(), &[]).await {
                    Ok(rows) => {
                        let elapsed_ms = started.elapsed().as_millis();
                        let columns = rows
                            .first()
                            .map(|row| {
                                row.columns()
                                    .iter()
                                    .map(|column| {
                                        ResultColumn::new(column.name(), column.type_().name())
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let rendered_rows = rows.iter().map(render_row).collect::<Vec<_>>();
                        let row_count = rendered_rows.len();

                        let _ = events_tx.send(PgEvent::QueryResult {
                            columns,
                            rows: rendered_rows,
                            elapsed_ms,
                        });
                        let _ = events_tx.send(PgEvent::QueryCompleted {
                            rows: row_count,
                            elapsed_ms,
                        });
                    }
                    Err(error) => {
                        let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
                    }
                }
            }
            PgCommand::Cancel => {
                let _ = events_tx.send(PgEvent::Notice("cancel requested".into()));
            }
            PgCommand::LoadCatalogNode { .. } => {
                load_catalog(client.as_ref(), &events_tx).await;
            }
        }
    }
}

async fn load_catalog(client: Option<&Client>, events_tx: &mpsc::UnboundedSender<PgEvent>) {
    let Some(client) = client else {
        return;
    };

    let sql = "\
        select schemaname, tablename \
        from pg_catalog.pg_tables \
        where schemaname not in ('pg_catalog', 'information_schema') \
        order by schemaname, tablename \
        limit 64";

    match client.query(sql, &[]).await {
        Ok(rows) => {
            let mut nodes = vec![
                CatalogNode::database("app_db"),
                CatalogNode::schema("public"),
                CatalogNode::folder("tables"),
            ];

            nodes.extend(rows.iter().filter_map(|row| {
                let table: Option<String> = row.try_get("tablename").ok();
                table.map(CatalogNode::table)
            }));

            let _ = events_tx.send(PgEvent::CatalogNodeLoaded {
                parent_id: "root".into(),
                nodes,
            });
        }
        Err(error) => {
            let _ = events_tx.send(PgEvent::QueryFailed(error.to_string()));
        }
    }
}

fn render_row(row: &Row) -> Vec<String> {
    row.columns()
        .iter()
        .enumerate()
        .map(|(ix, column)| render_cell(row, ix, column.type_().name()))
        .collect()
}

fn render_cell(row: &Row, ix: usize, pg_type: &str) -> String {
    match pg_type {
        "bool" => row
            .try_get::<usize, Option<bool>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int2" => row
            .try_get::<usize, Option<i16>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int4" => row
            .try_get::<usize, Option<i32>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "int8" => row
            .try_get::<usize, Option<i64>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "float4" => row
            .try_get::<usize, Option<f32>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "float8" => row
            .try_get::<usize, Option<f64>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        "json" | "jsonb" => row
            .try_get::<usize, Option<serde_json::Value>>(ix)
            .ok()
            .flatten()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".into()),
        _ => row
            .try_get::<usize, Option<String>>(ix)
            .ok()
            .flatten()
            .unwrap_or_else(|| "NULL".into()),
    }
}

#[derive(Clone, Debug)]
pub enum WorkspaceTab {
    Query { title: String },
    ObjectPreview { title: String },
    TableData { title: String },
    Explain { title: String },
    Activity,
    Locks,
}

impl WorkspaceTab {
    pub fn title(&self) -> &str {
        match self {
            WorkspaceTab::Query { title }
            | WorkspaceTab::ObjectPreview { title }
            | WorkspaceTab::TableData { title }
            | WorkspaceTab::Explain { title } => title,
            WorkspaceTab::Activity => "activity",
            WorkspaceTab::Locks => "locks",
        }
    }
}

#[derive(Clone, Debug)]
pub enum QueryStatus {
    Idle,
    Running,
    Completed { rows: usize, elapsed_ms: u128 },
    Failed { message: String },
}

impl QueryStatus {
    pub fn summary(&self) -> String {
        match self {
            QueryStatus::Idle => "idle".into(),
            QueryStatus::Running => "running".into(),
            QueryStatus::Completed { rows, elapsed_ms } => {
                format!("{rows} rows · {elapsed_ms} ms")
            }
            QueryStatus::Failed { message } => format!("failed · {message}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResultColumn {
    pub name: String,
    pub pg_type: String,
}

impl ResultColumn {
    pub fn new(name: impl Into<String>, pg_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pg_type: pg_type.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResultSetState {
    pub columns: Vec<ResultColumn>,
    pub rows: Vec<Vec<String>>,
}

impl ResultSetState {
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}
