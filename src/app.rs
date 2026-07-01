use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Root, TitleBar, h_flex, v_flex};

use crate::pg::{ConnectionProfile, PgCommand, PgEvent, PgRuntime, QueryStatus};
use crate::ui::{
    BottomPanel, CommandPalette, DatabasePanel, DatabasePanelEvent, Workspace, WorkspaceEvent,
    connection_picker,
    shared::{label, muted},
};

pub struct AppShell {
    pub workspace: Entity<Workspace>,
    pub database_panel: Entity<DatabasePanel>,
    pub command_palette: Entity<CommandPalette>,
    pub bottom_panel: Entity<BottomPanel>,
    pub pg_runtime: Option<PgRuntime>,
    pub connection_picker_open: bool,
    pub connected_profile: Option<String>,
    pub status_message: SharedString,
}

impl AppShell {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = cx.new(|_| Workspace::sample());
        let database_panel = cx.new(|_| DatabasePanel::sample());
        let command_palette = cx.new(|_| CommandPalette::default());
        let bottom_panel = cx.new(|_| BottomPanel::sample());

        cx.subscribe(&workspace, |this, _, event, cx| {
            this.handle_workspace_event(event.clone(), cx);
        })
        .detach();
        cx.subscribe(&database_panel, |this, _, event, cx| {
            this.handle_database_panel_event(event.clone(), cx);
        })
        .detach();

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
                        cx.notify();
                    });
                }
            })
            .detach();
        }

        Self {
            workspace,
            database_panel,
            command_palette,
            bottom_panel,
            pg_runtime,
            connection_picker_open: false,
            connected_profile: None,
            status_message: "Ready".into(),
        }
    }

    pub(crate) fn connect(&mut self, profile: ConnectionProfile, cx: &mut Context<Self>) {
        self.status_message = format!("Connecting to {}", profile.name).into();
        self.workspace.update(cx, |workspace, cx| {
            workspace.set_query_status(QueryStatus::Idle);
            cx.notify();
        });
        self.send_pg(PgCommand::Connect(profile));
    }

    pub(crate) fn run_query(&mut self, sql: String, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.set_query_status(QueryStatus::Running);
            cx.notify();
        });
        self.status_message = "Running query".into();
        self.send_pg(PgCommand::Execute { sql });
    }

    pub(crate) fn cancel_query(&mut self) {
        self.status_message = "Cancel requested".into();
        self.send_pg(PgCommand::Cancel);
    }

    pub(crate) fn explain_query(&mut self, sql: String, cx: &mut Context<Self>) {
        self.workspace.update(cx, |workspace, cx| {
            workspace.set_query_status(QueryStatus::Running);
            cx.notify();
        });
        self.status_message = "Running explain".into();
        self.send_pg(PgCommand::Execute {
            sql: format!("explain {sql}"),
        });
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
                self.database_panel.update(cx, |database_panel, cx| {
                    database_panel.set_nodes(nodes.clone());
                    cx.notify();
                });
                self.status_message = format!("Loaded {} catalog objects", nodes.len()).into();
            }
            PgEvent::QueryStarted => {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.set_query_status(QueryStatus::Running);
                    cx.notify();
                });
                self.status_message = "Query started".into();
            }
            PgEvent::QueryResult {
                columns,
                rows,
                elapsed_ms,
            } => {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.set_result(columns.clone(), rows.clone());
                    workspace.set_query_status(QueryStatus::Completed {
                        rows: rows.len(),
                        elapsed_ms: *elapsed_ms,
                    });
                    cx.notify();
                });
                self.status_message = format!("{} rows · {elapsed_ms} ms", rows.len()).into();
            }
            PgEvent::QueryCompleted { rows, elapsed_ms } => {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.set_query_status(QueryStatus::Completed {
                        rows: *rows,
                        elapsed_ms: *elapsed_ms,
                    });
                    cx.notify();
                });
            }
            PgEvent::QueryFailed(message) => {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.set_query_status(QueryStatus::Failed {
                        message: message.clone(),
                    });
                    cx.notify();
                });
                self.status_message = message.clone().into();
            }
            PgEvent::Notice(message) => {
                self.status_message = message.clone().into();
            }
            PgEvent::Disconnected | PgEvent::TransactionStatusChanged(_) => {}
        }

        self.bottom_panel.update(cx, |bottom_panel, cx| {
            bottom_panel.push_event(event);
            cx.notify();
        });
    }

    fn handle_workspace_event(&mut self, event: WorkspaceEvent, cx: &mut Context<Self>) {
        match event {
            WorkspaceEvent::TabSelected { index, title } => {
                self.status_message = format!("Opened tab {}: {title}", index + 1).into();
            }
            WorkspaceEvent::RunRequested { sql } => self.run_query(sql, cx),
            WorkspaceEvent::CancelRequested => self.cancel_query(),
            WorkspaceEvent::ExplainRequested { sql } => self.explain_query(sql, cx),
        }
        cx.notify();
    }

    fn handle_database_panel_event(&mut self, event: DatabasePanelEvent, cx: &mut Context<Self>) {
        match event {
            DatabasePanelEvent::ObjectSelected(name) => {
                self.status_message = format!("Selected {name}").into();
            }
            DatabasePanelEvent::RefreshRequested => {
                self.status_message = "Refreshing catalog".into();
                self.send_pg(PgCommand::LoadCatalogNode {
                    node_id: "root".into(),
                });
            }
        }
        cx.notify();
    }

    fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> TitleBar {
        TitleBar::new().child(
            h_flex()
                .size_full()
                .px_2()
                .gap_2()
                .child(label("LoxQL").text_sm().font_weight(FontWeight::MEDIUM))
                .child(self.render_connection_switcher(cx))
                .child(div().flex_1())
                .child(muted("app_db / public")),
        )
    }

    fn render_connection_switcher(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id("connection-switcher")
            .h(px(24.))
            .px_2()
            .gap_1()
            .rounded(px(5.))
            .text_sm()
            .cursor_pointer()
            .bg(cx.theme().secondary.opacity(0.45))
            .hover(|el| el.bg(cx.theme().secondary_hover))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, _, cx| {
                this.connection_picker_open = !this.connection_picker_open;
                cx.notify();
            }))
            .child(div().text_color(cx.theme().muted_foreground).child("●"))
            .child(label(
                self.connected_profile.as_deref().unwrap_or("local-dev"),
            ))
            .child(muted("⌄"))
    }

    fn render_main(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .items_stretch()
            .child(self.database_panel.clone())
            .child(self.workspace.clone())
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .relative()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_title_bar(window, cx))
            .child(self.render_main(window, cx))
            .child(self.bottom_panel.clone())
            .when(self.command_palette.read(cx).open, |el| {
                el.child(self.command_palette.clone())
            })
            .when(self.connection_picker_open, |el| {
                el.child(connection_picker::render(cx))
            })
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
