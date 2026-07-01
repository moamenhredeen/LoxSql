use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};

use crate::app::AppShell;
use crate::ui::shared::{label, panel_icon};

pub(crate) struct DatabasePanel {
    pub(crate) selected_object: String,
}

impl DatabasePanel {
    pub(crate) fn sample() -> Self {
        Self {
            selected_object: "users".into(),
        }
    }

    pub(crate) fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        v_flex()
            .w(px(250.))
            .min_w(px(220.))
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .h(px(30.))
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(label("DATABASE").text_xs().text_color(cx.theme().muted_foreground))
                    .child(div().flex_1())
                    .child(panel_icon("↻", cx)),
            )
            .child(
                div()
                    .mx_2()
                    .my_1()
                    .px_2()
                    .py_1()
                    .rounded(px(6.))
                    .bg(cx.theme().secondary.opacity(0.55))
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Filter"),
            )
            .child(
                v_flex()
                    .px_1()
                    .child(tree_row(
                        TreeIcon::OpenFolder,
                        "app_db",
                        0,
                        "app_db" == self.selected_object,
                        cx,
                    ))
                    .child(tree_row(
                        TreeIcon::OpenFolder,
                        "public",
                        1,
                        "public" == self.selected_object,
                        cx,
                    ))
                    .child(tree_row(TreeIcon::OpenFolder, "tables", 2, false, cx))
                    .child(tree_row(
                        TreeIcon::File,
                        "users",
                        3,
                        "users" == self.selected_object,
                        cx,
                    ))
                    .child(tree_row(
                        TreeIcon::File,
                        "orders",
                        3,
                        "orders" == self.selected_object,
                        cx,
                    ))
                    .child(tree_row(TreeIcon::ClosedFolder, "views", 2, false, cx))
                    .child(tree_row(TreeIcon::ClosedFolder, "functions", 2, false, cx))
                    .child(tree_row(TreeIcon::ClosedFolder, "extensions", 1, false, cx)),
            )
    }
}

#[derive(Clone, Copy)]
enum TreeIcon {
    OpenFolder,
    ClosedFolder,
    File,
}

fn tree_row(
    icon: TreeIcon,
    text: &str,
    indent: usize,
    selected: bool,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let object_name = text.to_string();

    h_flex()
        .id(format!("catalog-row-{object_name}"))
        .h(px(22.))
        .gap_1()
        .mx_1()
        .pl(px((indent * 13 + 4) as f32))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(|el| el.bg(gpui::white().opacity(0.06)))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.database_panel.selected_object = object_name.clone();
            this.status_message = format!("Selected {}", object_name).into();
            cx.notify();
        }))
        .when(selected, |el| el.bg(cx.theme().secondary.opacity(0.85)))
        .when(!selected, |el| el.text_color(cx.theme().muted_foreground))
        .child(tree_icon(icon, cx))
        .child(div().text_sm().child(text.to_string()))
}

fn tree_icon(icon: TreeIcon, cx: &mut Context<AppShell>) -> impl IntoElement {
    let icon_name = match icon {
        TreeIcon::OpenFolder => IconName::FolderOpen,
        TreeIcon::ClosedFolder => IconName::FolderClosed,
        TreeIcon::File => IconName::File,
    };

    div()
        .w(px(16.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            Icon::new(icon_name)
                .xsmall()
                .text_color(cx.theme().muted_foreground),
        )
}
