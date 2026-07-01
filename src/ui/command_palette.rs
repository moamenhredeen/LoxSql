use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::ui::shared::muted;

#[derive(Default)]
pub(crate) struct CommandPalette {
    pub(crate) open: bool,
}

impl CommandPalette {
    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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

fn command_row<T>(label_text: &str, shortcut: &str, cx: &mut Context<T>) -> impl IntoElement {
    h_flex()
        .h(px(34.))
        .px_3()
        .gap_2()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(div().text_sm().child(label_text.to_string()))
        .child(div().flex_1())
        .child(muted(shortcut.to_string()))
}
