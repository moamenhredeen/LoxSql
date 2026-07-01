use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

use crate::app::AppShell;

pub(crate) fn label(text: impl Into<SharedString>) -> Div {
    div().child(text.into())
}

pub(crate) fn muted(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(gpui::rgb(0x7c8490))
        .child(text.into())
}

pub(crate) fn thin_status(text: &str, cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .px_1()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text.to_string())
}

pub(crate) fn status_item(
    text: impl Into<SharedString>,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    div()
        .h_full()
        .px_1()
        .flex()
        .items_center()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .cursor_pointer()
        .hover(|el| el.bg(gpui::white().opacity(0.05)))
        .child(text.into())
}

pub(crate) fn action_button(text: &str, cx: &mut Context<AppShell>) -> impl IntoElement {
    let action = text.to_string();

    div()
        .id(format!("action-button-{action}"))
        .px_2()
        .py_1()
        .rounded(px(5.))
        .text_sm()
        .cursor_pointer()
        .hover(|el| el.bg(gpui::white().opacity(0.07)))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.status_message = format!("{action} requested").into();
            cx.notify();
        }))
        .child(text.to_string())
}

pub(crate) fn panel_icon(text: &str, cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .size(px(22.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .cursor_pointer()
        .hover(|el| el.bg(gpui::white().opacity(0.06)))
        .child(text.to_string())
}

pub(crate) fn command_row(
    label_text: &str,
    shortcut: &str,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
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
