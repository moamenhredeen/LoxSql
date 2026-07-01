use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Root, TitleBar, h_flex, v_flex};

use crate::session::Session;
use crate::ui::connection_picker::{ConnectionPicker, ConnectionPickerEvent};
use crate::ui::{
    BottomPanel, CommandPalette, DatabasePanel, DatabasePanelEvent, Workspace, WorkspaceEvent,
    shared::{label, muted},
};

pub struct AppShell {
    pub session: Entity<Session>,
    pub workspace: Entity<Workspace>,
    pub database_panel: Entity<DatabasePanel>,
    pub command_palette: Entity<CommandPalette>,
    pub bottom_panel: Entity<BottomPanel>,
    pub connection_picker: Entity<ConnectionPicker>,
}

impl AppShell {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let session = cx.new(Session::new);
        let workspace = cx.new(|cx| Workspace::new(session.clone(), cx));
        let database_panel = cx.new(|cx| DatabasePanel::new(session.clone(), cx));
        let command_palette = cx.new(|_| CommandPalette::default());
        let bottom_panel = cx.new(|cx| BottomPanel::new(session.clone(), cx));
        let connection_picker = cx.new(|cx| ConnectionPicker::new(session.clone(), window, cx));

        cx.subscribe(&workspace, |this, _, event, cx| {
            this.handle_workspace_event(event.clone(), cx);
        })
        .detach();
        cx.subscribe(&database_panel, |this, _, event, cx| {
            this.handle_database_panel_event(event.clone(), cx);
        })
        .detach();
        cx.subscribe(&connection_picker, |this, _, event, cx| {
            this.handle_connection_picker_event(event.clone(), cx);
        })
        .detach();

        Self {
            session,
            workspace,
            database_panel,
            command_palette,
            bottom_panel,
            connection_picker,
        }
    }

    fn handle_workspace_event(&mut self, event: WorkspaceEvent, cx: &mut Context<Self>) {
        self.session.update(cx, |session, cx| match event {
            WorkspaceEvent::TabSelected { index, title } => {
                session.set_status(format!("Opened tab {}: {title}", index + 1), cx);
            }
            WorkspaceEvent::RunRequested { sql } => session.run_query(sql, cx),
            WorkspaceEvent::CancelRequested => session.cancel_query(cx),
            WorkspaceEvent::ExplainRequested { sql } => session.explain_query(sql, cx),
        });
    }

    fn handle_database_panel_event(&mut self, event: DatabasePanelEvent, cx: &mut Context<Self>) {
        self.session.update(cx, |session, cx| match event {
            DatabasePanelEvent::ObjectSelected(name) => {
                session.set_status(format!("Selected {name}"), cx);
            }
            DatabasePanelEvent::RefreshRequested => session.refresh_catalog(cx),
        });
    }

    fn handle_connection_picker_event(
        &mut self,
        event: ConnectionPickerEvent,
        cx: &mut Context<Self>,
    ) {
        self.session.update(cx, |session, cx| match event {
            ConnectionPickerEvent::ConnectionSelected(profile) => session.connect(profile, cx),
            ConnectionPickerEvent::CreateNewConnection => {
                session.set_status("Create connection: not implemented yet", cx);
            }
        });
    }

    fn render_title_bar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> TitleBar {
        TitleBar::new().child(
            h_flex()
                .size_full()
                .px_2()
                .gap_2()
                .child(label("LoxQL").text_sm().font_weight(FontWeight::MEDIUM))
                .child(self.connection_picker.clone())
                .child(div().flex_1())
                .child(muted("app_db / public")),
        )
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
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
