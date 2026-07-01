use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Root, TitleBar, h_flex, v_flex};

use crate::fonts::JETBRAINS_MONO;
use crate::ui::{
    BottomPanel, CommandPalette, DatabasePanel, Workspace,
    connection_picker,
    shared::{label, muted, thin_status},
};

pub struct AppShell {
    pub(crate) workspace: Workspace,
    pub(crate) database_panel: DatabasePanel,
    pub(crate) command_palette: CommandPalette,
    pub(crate) bottom_panel: BottomPanel,
    pub(crate) connection_picker_open: bool,
    pub(crate) status_message: SharedString,
}

impl AppShell {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            workspace: Workspace::sample(),
            database_panel: DatabasePanel::sample(),
            command_palette: CommandPalette::default(),
            bottom_panel: BottomPanel::sample(),
            connection_picker_open: false,
            status_message: "Ready".into(),
        }
    }

    fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> TitleBar {
        TitleBar::new().child(
            h_flex()
                .size_full()
                .px_2()
                .gap_2()
                .child(label("LoxQL").text_sm().font_weight(FontWeight::MEDIUM))
                .child(div().w(px(8.)))
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
            .hover(|el| el.bg(gpui::white().opacity(0.06)))
            .on_click(cx.listener(|this, _, _, cx| {
                this.connection_picker_open = !this.connection_picker_open;
                cx.notify();
            }))
            .child(div().text_color(cx.theme().muted_foreground).child("●"))
            .child(label("local-dev"))
            .child(muted("⌄"))
    }

    fn render_top_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(32.))
            .px_3()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                div()
                    .id("command-search")
                    .flex_1()
                    .max_w(px(460.))
                    .px_3()
                    .py_1()
                    .rounded(px(6.))
                    .text_sm()
                    .cursor_pointer()
                    .text_color(cx.theme().muted_foreground)
                    .bg(cx.theme().secondary.opacity(0.55))
                    .hover(|el| el.bg(gpui::white().opacity(0.07)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_palette.open = !this.command_palette.open;
                        cx.notify();
                    }))
                    .child("Search commands, objects, or actions"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(thin_status("connected", cx))
                    .child(thin_status("tx idle", cx))
                    .child(thin_status("limit 1000", cx)),
            )
    }

    fn render_main(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_1()
            .items_stretch()
            .child(self.database_panel.render(cx))
            .child(self.workspace.render(cx))
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .relative()
            .font_family(JETBRAINS_MONO)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_title_bar(window, cx))
            .child(self.render_top_bar(window, cx))
            .child(self.render_main(window, cx))
            .child(self.bottom_panel.render(cx))
            .when(self.command_palette.open, |el| {
                el.child(self.command_palette.render(cx))
            })
            .when(self.connection_picker_open, |el| {
                el.child(connection_picker::render(cx))
            })
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
