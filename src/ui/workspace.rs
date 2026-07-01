use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::app::AppShell;
use crate::fonts::JETBRAINS_MONO;
use crate::pg::{QueryStatus, ResultColumn, ResultSetState, SessionStatus, WorkspaceTab};
use crate::ui::shared::{action_button, muted};

pub(crate) struct Workspace {
    tabs: Vec<WorkspaceTab>,
    pub(crate) active_tab: usize,
    editor: SqlEditorState,
    result: ResultSetState,
    session: SessionStatus,
}

impl Workspace {
    pub(crate) fn sample() -> Self {
        Self {
            tabs: vec![
                WorkspaceTab::Query {
                    title: "scratch.sql".into(),
                },
                WorkspaceTab::ObjectPreview {
                    title: "public.users".into(),
                },
                WorkspaceTab::Activity,
            ],
            active_tab: 0,
            editor: SqlEditorState::sample(),
            result: ResultSetState::sample(),
            session: SessionStatus {
                profile_id: Some("local-dev".into()),
                database: Some("app_db".into()),
                transaction: "idle".into(),
                query: QueryStatus::Completed {
                    rows: 42,
                    elapsed_ms: 82,
                },
            },
        }
    }

    pub(crate) fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w(px(0.))
            .child(self.render_tabs(cx))
            .child(self.render_query_toolbar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .child(self.editor.render(cx))
                    .child(self.result.render(cx)),
            )
    }

    fn render_tabs(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        h_flex()
            .h(px(31.))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(self.render_tab(0, cx))
            .child(self.render_tab(1, cx))
            .child(self.render_tab(2, cx))
            .child(div().flex_1())
    }

    fn render_tab(&self, ix: usize, cx: &mut Context<AppShell>) -> impl IntoElement {
        let title = self
            .tabs
            .get(ix)
            .map(WorkspaceTab::title)
            .unwrap_or("untitled")
            .to_string();
        let click_title = title.clone();

        div()
            .id(("workspace-tab", ix))
            .h_full()
            .px_3()
            .gap_2()
            .flex()
            .items_center()
            .border_r_1()
            .border_color(cx.theme().border)
            .text_sm()
            .cursor_pointer()
            .hover(|el| el.bg(gpui::white().opacity(0.05)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.workspace.active_tab = ix;
                this.status_message = format!("Opened {}", click_title).into();
                cx.notify();
            }))
            .when(ix == self.active_tab, |el| {
                el.bg(cx.theme().secondary.opacity(0.75))
            })
            .when(ix != self.active_tab, |el| {
                el.text_color(cx.theme().muted_foreground)
            })
            .child(title)
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("×"),
            )
    }

    fn render_query_toolbar(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        h_flex()
            .h(px(32.))
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(action_button("Run", cx))
            .child(action_button("Stop", cx))
            .child(action_button("Explain", cx))
            .child(div().flex_1())
            .child(muted(self.session.query.summary()))
    }
}

struct SqlEditorState {
    text: ropey::Rope,
    current_line: usize,
}

impl SqlEditorState {
    fn sample() -> Self {
        Self {
            text: ropey::Rope::from_str(
                "select id, email, created_at\nfrom public.users\nwhere created_at > now() - interval '7 days'\norder by created_at desc;\n",
            ),
            current_line: 0,
        }
    }

    fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        v_flex()
            .h(px(260.))
            .min_h(px(180.))
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .h(px(26.))
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(muted("scratch.sql"))
                    .child(div().flex_1())
                    .child(muted("PostgreSQL")),
            )
            .child(self.render_line(0, cx))
            .child(self.render_line(1, cx))
            .child(self.render_line(2, cx))
            .child(self.render_line(3, cx))
    }

    fn render_line(&self, ix: usize, cx: &mut Context<AppShell>) -> impl IntoElement {
        let line = self
            .text
            .get_line(ix)
            .map(|line| line.to_string())
            .unwrap_or_default();

        h_flex()
            .h(px(24.))
            .items_center()
            .bg(if ix == self.current_line {
                cx.theme().secondary
            } else {
                cx.theme().background
            })
            .child(
                div()
                    .w(px(46.))
                    .pr_2()
                    .text_right()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child((ix + 1).to_string()),
            )
            .child(
                div()
                    .font_family(JETBRAINS_MONO)
                    .text_sm()
                    .child(line.trim_end().to_string()),
            )
    }
}

impl ResultSetState {
    pub(crate) fn sample() -> Self {
        Self {
            columns: vec![
                ResultColumn::new("id", "int8"),
                ResultColumn::new("email", "text"),
                ResultColumn::new("created_at", "timestamptz"),
            ],
            rows: vec![
                vec![
                    "1001".into(),
                    "ada@example.dev".into(),
                    "2026-07-01 09:41:02".into(),
                ],
                vec![
                    "1000".into(),
                    "grace@example.dev".into(),
                    "2026-06-30 18:12:44".into(),
                ],
                vec![
                    "999".into(),
                    "linus@example.dev".into(),
                    "2026-06-30 10:07:19".into(),
                ],
            ],
        }
    }

    fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_h(px(0.))
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h(px(28.))
                    .bg(cx.theme().secondary)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(grid_header(&self.columns[0], cx))
                    .child(grid_header(&self.columns[1], cx))
                    .child(grid_header(&self.columns[2], cx)),
            )
            .child(grid_row(&self.rows[0], cx))
            .child(grid_row(&self.rows[1], cx))
            .child(grid_row(&self.rows[2], cx))
    }
}

fn grid_header(column: &ResultColumn, cx: &mut Context<AppShell>) -> impl IntoElement {
    h_flex()
        .w(relative(0.333))
        .h_full()
        .px_2()
        .gap_2()
        .border_r_1()
        .border_color(cx.theme().border)
        .child(div().text_sm().child(column.name.clone()))
        .child(muted(column.pg_type.clone()))
}

fn grid_row(row: &[String], cx: &mut Context<AppShell>) -> impl IntoElement {
    h_flex()
        .h(px(30.))
        .border_b_1()
        .border_color(cx.theme().border)
        .child(grid_cell(&row[0], cx))
        .child(grid_cell(&row[1], cx))
        .child(grid_cell(&row[2], cx))
}

fn grid_cell(text: &str, cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .w(relative(0.333))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .border_r_1()
        .border_color(cx.theme().border)
        .text_sm()
        .child(text.to_string())
}
