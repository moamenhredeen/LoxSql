use gpui::*;

use crate::pg::{
    CatalogNode, ConnectionProfile, PgCommand, PgEvent, PgRuntime, QueryStatus, ResultSetState,
};

/// Non-view model holding all connection/query/catalog state. Views hold an
/// `Entity<Session>`, observe it, and render from it; user intents are routed
/// here by `AppShell`.
pub struct Session {
    pg_runtime: Option<PgRuntime>,
    pub connected_profile: Option<String>,
    pub status_message: SharedString,
    pub query_status: QueryStatus,
    pub catalog: Vec<CatalogNode>,
    pub result: ResultSetState,
    pub event_log: Vec<PgEvent>,
}

impl Session {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (pg_runtime, events_rx) = match PgRuntime::new() {
            Ok((runtime, events_rx)) => (Some(runtime), Some(events_rx)),
            Err(error) => {
                eprintln!("failed to start postgres runtime: {error}");
                (None, None)
            }
        };

        if let Some(mut events_rx) = events_rx {
            cx.spawn(async move |this, cx| {
                while let Some(event) = events_rx.recv().await {
                    let _ = this.update(cx, |this, cx| {
                        this.apply_pg_event(event, cx);
                    });
                }
            })
            .detach();
        }

        Self {
            pg_runtime,
            connected_profile: None,
            status_message: "Ready".into(),
            query_status: QueryStatus::Idle,
            catalog: sample_catalog(),
            result: ResultSetState::empty(),
            event_log: Vec::new(),
        }
    }

    pub fn connect(&mut self, profile: ConnectionProfile, cx: &mut Context<Self>) {
        self.status_message = format!("Connecting to {}", profile.name).into();
        self.query_status = QueryStatus::Idle;
        self.send_pg(PgCommand::Connect(profile));
        cx.notify();
    }

    pub fn run_query(&mut self, sql: String, cx: &mut Context<Self>) {
        self.query_status = QueryStatus::Running;
        self.status_message = "Running query".into();
        self.send_pg(PgCommand::Execute { sql });
        cx.notify();
    }

    pub fn cancel_query(&mut self, cx: &mut Context<Self>) {
        self.status_message = "Cancel requested".into();
        self.send_pg(PgCommand::Cancel);
        cx.notify();
    }

    pub fn explain_query(&mut self, sql: String, cx: &mut Context<Self>) {
        self.query_status = QueryStatus::Running;
        self.status_message = "Running explain".into();
        self.send_pg(PgCommand::Execute {
            sql: format!("explain {sql}"),
        });
        cx.notify();
    }

    pub fn refresh_catalog(&mut self, cx: &mut Context<Self>) {
        self.status_message = "Refreshing catalog".into();
        self.send_pg(PgCommand::LoadCatalogNode {
            node_id: "root".into(),
        });
        cx.notify();
    }

    pub fn set_status(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.status_message = message.into();
        cx.notify();
    }

    fn send_pg(&mut self, command: PgCommand) {
        if let Some(runtime) = self.pg_runtime.as_ref() {
            runtime.spawn_command(command);
        } else {
            self.status_message = "Postgres runtime failed to start".into();
        }
    }

    fn apply_pg_event(&mut self, event: PgEvent, cx: &mut Context<Self>) {
        match &event {
            PgEvent::Connected { profile_id } => {
                self.connected_profile = Some(profile_id.clone());
                self.status_message = format!("Connected to {profile_id}").into();
            }
            PgEvent::CatalogNodeLoaded { nodes, .. } => {
                self.catalog = nodes.clone();
                self.status_message = format!("Loaded {} catalog objects", nodes.len()).into();
            }
            PgEvent::QueryStarted => {
                self.query_status = QueryStatus::Running;
                self.status_message = "Query started".into();
            }
            PgEvent::QueryResult {
                columns,
                rows,
                elapsed_ms,
            } => {
                self.result = ResultSetState {
                    columns: columns.clone(),
                    rows: rows.clone(),
                };
                self.query_status = QueryStatus::Completed {
                    rows: rows.len(),
                    elapsed_ms: *elapsed_ms,
                };
                self.status_message = format!("{} rows · {elapsed_ms} ms", rows.len()).into();
            }
            PgEvent::QueryCompleted { rows, elapsed_ms } => {
                self.query_status = QueryStatus::Completed {
                    rows: *rows,
                    elapsed_ms: *elapsed_ms,
                };
            }
            PgEvent::QueryFailed(message) => {
                self.query_status = QueryStatus::Failed {
                    message: message.clone(),
                };
                self.status_message = message.clone().into();
            }
            PgEvent::Notice(message) => {
                self.status_message = message.clone().into();
            }
            PgEvent::Disconnected | PgEvent::TransactionStatusChanged(_) => {}
        }

        self.event_log.push(event);
        if self.event_log.len() > 64 {
            self.event_log.remove(0);
        }
        cx.notify();
    }
}

// Placeholder catalog shown before the first connection.
fn sample_catalog() -> Vec<CatalogNode> {
    vec![
        CatalogNode::database("app_db"),
        CatalogNode::schema("public"),
        CatalogNode::folder("tables"),
        CatalogNode::table("users"),
        CatalogNode::table("orders"),
        CatalogNode::folder("views"),
        CatalogNode::folder("functions"),
        CatalogNode::folder("extensions"),
    ]
}
