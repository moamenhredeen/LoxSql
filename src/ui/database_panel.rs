use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};

use crate::pg::{CatalogNode, CatalogNodeKind};
use crate::ui::shared::label;

#[derive(Clone)]
pub(crate) enum DatabasePanelEvent {
    ObjectSelected(String),
    RefreshRequested,
}

pub(crate) struct DatabasePanel {
    pub(crate) selected_object: String,
    nodes: Vec<CatalogNode>,
}

impl EventEmitter<DatabasePanelEvent> for DatabasePanel {}

impl DatabasePanel {
    pub(crate) fn sample() -> Self {
        Self {
            selected_object: "users".into(),
            nodes: vec![
                CatalogNode::database("app_db"),
                CatalogNode::schema("public"),
                CatalogNode::folder("tables"),
                CatalogNode::table("users"),
                CatalogNode::table("orders"),
                CatalogNode::folder("views"),
                CatalogNode::folder("functions"),
                CatalogNode::folder("extensions"),
            ],
        }
    }

    pub(crate) fn set_nodes(&mut self, nodes: Vec<CatalogNode>) {
        self.nodes = nodes;
    }

}

impl Render for DatabasePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .nodes
            .iter()
            .enumerate()
            .map(|(ix, node)| {
                let (icon, indent) = tree_node_style(node, ix);
                tree_row(
                    icon,
                    &node.name,
                    indent,
                    node.name == self.selected_object,
                    cx,
                )
                .into_any_element()
            })
            .collect::<Vec<_>>();

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
                    .child(panel_icon(cx)),
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
                    .children(rows),
            )
    }
}

#[derive(Clone, Copy)]
enum TreeIcon {
    OpenFolder,
    ClosedFolder,
    File,
}

fn tree_node_style(node: &CatalogNode, ix: usize) -> (TreeIcon, usize) {
    match node.kind {
        CatalogNodeKind::Database => (TreeIcon::OpenFolder, 0),
        CatalogNodeKind::Schema => (TreeIcon::OpenFolder, 1),
        CatalogNodeKind::Folder => {
            if node.name == "tables" {
                (TreeIcon::OpenFolder, 2)
            } else {
                (TreeIcon::ClosedFolder, if ix < 3 { 2 } else { 1 })
            }
        }
        CatalogNodeKind::Table => (TreeIcon::File, 3),
    }
}

fn tree_row(
    icon: TreeIcon,
    text: &str,
    indent: usize,
    selected: bool,
    cx: &mut Context<DatabasePanel>,
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
        .hover(|el| el.bg(cx.theme().secondary_hover))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.selected_object = object_name.clone();
            cx.emit(DatabasePanelEvent::ObjectSelected(object_name.clone()));
            cx.notify();
        }))
        .when(selected, |el| el.bg(cx.theme().secondary.opacity(0.85)))
        .when(!selected, |el| el.text_color(cx.theme().muted_foreground))
        .child(tree_icon(icon, cx))
        .child(div().text_sm().child(text.to_string()))
}

fn tree_icon(icon: TreeIcon, cx: &mut Context<DatabasePanel>) -> impl IntoElement {
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

fn panel_icon(cx: &mut Context<DatabasePanel>) -> impl IntoElement {
    div()
        .id("database-refresh")
        .size(px(22.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .cursor_pointer()
        .hover(|el| el.bg(cx.theme().secondary_hover))
        .on_click(cx.listener(|_, _, _, cx| {
            cx.emit(DatabasePanelEvent::RefreshRequested);
            cx.notify();
        }))
        .child("↻")
}
