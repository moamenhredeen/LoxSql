use gpui::*;
use gpui_component::{ActiveTheme, v_flex};

use crate::app::AppShell;
use crate::ui::shared::command_row;

#[derive(Default)]
pub(crate) struct CommandPalette {
    pub(crate) open: bool,
}

impl CommandPalette {
    pub(crate) fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        div()
            .absolute()
            .top(px(64.))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                v_flex()
                    .w(px(560.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .shadow_lg()
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .text_color(cx.theme().muted_foreground)
                            .child("Run command"),
                    )
                    .child(command_row("Run selected query", "Ctrl+Enter", cx))
                    .child(command_row("Explain analyze", "Ctrl+Shift+E", cx))
                    .child(command_row("Show DDL", "", cx))
                    .child(command_row("Open pg_stat_activity", "", cx)),
            )
    }
}
