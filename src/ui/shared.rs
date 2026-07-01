use gpui::*;

pub(crate) fn label(text: impl Into<SharedString>) -> Div {
    div().child(text.into())
}

pub(crate) fn muted(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(gpui::rgb(0x7c8490))
        .child(text.into())
}
