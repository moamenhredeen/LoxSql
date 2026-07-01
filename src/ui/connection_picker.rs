use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, v_flex};

use crate::app::AppShell;
use crate::pg::ConnectionProfile;
use crate::ui::shared::{label, muted};

pub(crate) fn render(cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .absolute()
        .top(px(66.))
        .left(px(12.))
        .w(px(340.))
        .rounded(px(8.))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .shadow_lg()
        .child(
            v_flex()
                .child(
                    div()
                        .m_2()
                        .px_2()
                        .py_1()
                        .rounded(px(6.))
                        .bg(cx.theme().secondary.opacity(0.7))
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Search connections"),
                )
                .child(connection_option(
                    "local-dev",
                    "localhost:5432 · app_db",
                    true,
                    cx,
                ))
                .child(connection_option(
                    "docker-postgres",
                    "localhost:5432 · app_db",
                    false,
                    cx,
                ))
                .child(connection_option("staging", "not connected", false, cx)),
        )
}

fn connection_option(
    name: &str,
    detail: &str,
    selected: bool,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let name = name.to_string();
    let display_name = name.clone();

    gpui_component::h_flex()
        .id(format!("connection-option-{name}"))
        .h(px(38.))
        .mx_1()
        .px_2()
        .gap_2()
        .rounded(px(6.))
        .cursor_pointer()
        .hover(|el| el.bg(cx.theme().secondary_hover))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.connection_picker_open = false;
            this.connect(ConnectionProfile::local_dev(), cx);
            this.status_message = format!("Selected connection {}", name).into();
            cx.notify();
        }))
        .when(selected, |el| el.bg(cx.theme().secondary.opacity(0.8)))
        .child(
            div()
                .w(px(12.))
                .text_color(cx.theme().muted_foreground)
                .child(if selected { "✓" } else { "" }),
        )
        .child(
            v_flex()
                .gap_0()
                .child(label(display_name).text_sm())
                .child(muted(detail.to_string()).text_xs()),
        )
}
