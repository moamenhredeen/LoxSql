#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc;

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
    Connected { profile_id: String },
    Disconnected,
    CatalogNodeLoaded { parent_id: String, nodes: Vec<CatalogNode> },
    QueryStarted,
    RowBatch(Vec<Vec<String>>),
    QueryCompleted { rows: usize, elapsed_ms: u128 },
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
            PgEvent::RowBatch(rows) => format!("received {} rows", rows.len()),
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
    events_tx: mpsc::UnboundedSender<PgEvent>,
}

impl PgRuntime {
    pub fn new(events_tx: mpsc::UnboundedSender<PgEvent>) -> anyhow::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("loxql-pg")
            .build()?;

        Ok(Self { runtime, events_tx })
    }

    pub fn spawn_command(&self, command: PgCommand) {
        let events = self.events_tx.clone();
        self.runtime.spawn(async move {
            let event = match command {
                PgCommand::Connect(profile) => PgEvent::Connected {
                    profile_id: profile.id,
                },
                PgCommand::Execute { sql } => {
                    let _ = pg_query::parse(&sql);
                    PgEvent::QueryCompleted {
                        rows: 0,
                        elapsed_ms: 0,
                    }
                }
                PgCommand::Cancel => PgEvent::Notice("cancel requested".into()),
                PgCommand::LoadCatalogNode { node_id } => PgEvent::CatalogNodeLoaded {
                    parent_id: node_id,
                    nodes: Vec::new(),
                },
            };

            let _ = events.send(event);
        });
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
pub struct SessionStatus {
    pub profile_id: Option<String>,
    pub database: Option<String>,
    pub transaction: String,
    pub query: QueryStatus,
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
