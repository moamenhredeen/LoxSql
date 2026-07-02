use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, h_flex, v_flex};

use crate::pg::{CatalogNode, CatalogNodeKind};
use crate::session::Session;
use crate::ui::shared::label;

#[derive(Clone)]
pub(crate) enum DatabasePanelEvent {
    ObjectSelected(String),
    TableSelected { qualified_name: String },
    RefreshRequested,
}

pub(crate) struct DatabasePanel {
    pub(crate) selected_object: String,
    session: Entity<Session>,
}

impl EventEmitter<DatabasePanelEvent> for DatabasePanel {}

impl DatabasePanel {
    pub(crate) fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();

        Self {
            selected_object: String::new(),
            session,
        }
    }
}

impl Render for DatabasePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nodes = self.session.read(cx).catalog.clone();
        let connected = self.session.read(cx).is_connected();
        let rows = nodes
            .iter()
            .map(|node| {
                let (icon, indent) = tree_node_style(node);
                tree_row(
                    icon,
                    node,
                    indent,
                    node.qualified_name() == self.selected_object,
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
                    .child(
                        label("DATABASE")
                            .text_xs()
                            .text_color(cx.theme().muted_foreground),
                    )
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
            .when(!connected, |el| {
                el.child(
                    div()
                        .p_3()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Not connected. Pick a connection in the title bar."),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .px_1()
                    .overflow_hidden()
                    .children(rows),
            )
    }
}

#[derive(Clone, Copy)]
enum TreeIcon {
    OpenFolder,
    File,
}

fn tree_node_style(node: &CatalogNode) -> (TreeIcon, usize) {
    match node.kind {
        CatalogNodeKind::Database => (TreeIcon::OpenFolder, 0),
        CatalogNodeKind::Schema => (TreeIcon::OpenFolder, 1),
        CatalogNodeKind::Folder => (TreeIcon::OpenFolder, 2),
        CatalogNodeKind::Table => (TreeIcon::File, 3),
    }
}

fn tree_row(
    icon: TreeIcon,
    node: &CatalogNode,
    indent: usize,
    selected: bool,
    cx: &mut Context<DatabasePanel>,
) -> impl IntoElement {
    let display_name = node.name.clone();
    let qualified_name = node.qualified_name();
    let is_table = matches!(node.kind, CatalogNodeKind::Table);

    h_flex()
        .id(SharedString::from(format!("catalog-row-{qualified_name}")))
        .h(px(22.))
        .gap_1()
        .mx_1()
        .pl(px((indent * 13 + 4) as f32))
        .rounded(px(4.))
        .cursor_pointer()
        .hover(|el| el.bg(cx.theme().secondary_hover))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.selected_object = qualified_name.clone();
            if is_table {
                cx.emit(DatabasePanelEvent::TableSelected {
                    qualified_name: qualified_name.clone(),
                });
            } else {
                cx.emit(DatabasePanelEvent::ObjectSelected(qualified_name.clone()));
            }
            cx.notify();
        }))
        .when(selected, |el| el.bg(cx.theme().secondary.opacity(0.85)))
        .when(!selected, |el| el.text_color(cx.theme().muted_foreground))
        .child(tree_icon(icon, cx))
        .child(div().text_sm().child(display_name))
}

fn tree_icon(icon: TreeIcon, cx: &mut Context<DatabasePanel>) -> impl IntoElement {
    let icon_name = match icon {
        TreeIcon::OpenFolder => IconName::FolderOpen,
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
