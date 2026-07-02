#![allow(dead_code)]

mod runtime;
mod worker;

pub use runtime::PgRuntime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub user: String,
    pub password: Option<String>,
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
            password: Some("postgres".into()),
            ssl_mode: "disable".into(),
        }
    }

    pub fn database_name(&self) -> &str {
        self.database.as_deref().unwrap_or("postgres")
    }

    fn connection_string(&self) -> String {
        let mut parts = format!(
            "host={} port={} user={} dbname={}",
            self.host,
            self.port,
            self.user,
            self.database_name()
        );
        if let Some(password) = self.password.as_deref() {
            parts.push_str(&format!(" password={password}"));
        }
        parts
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
    /// Schema this object belongs to (set for tables).
    pub schema: Option<String>,
}

impl CatalogNode {
    pub fn database(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Database,
            schema: None,
        }
    }

    pub fn schema(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Schema,
            schema: None,
        }
    }

    pub fn folder(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Folder,
            schema: None,
        }
    }

    pub fn table(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CatalogNodeKind::Table,
            schema: Some(schema.into()),
        }
    }

    /// Schema-qualified, quoted identifier for tables.
    pub fn qualified_name(&self) -> String {
        match self.schema.as_deref() {
            Some(schema) => format!("\"{}\".\"{}\"", schema, self.name),
            None => format!("\"{}\"", self.name),
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
        database: String,
        server_version: String,
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
            PgEvent::Connected {
                profile_id,
                database,
                ..
            } => format!("connected to {profile_id} ({database})"),
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

#[derive(Clone, Debug, Default)]
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
