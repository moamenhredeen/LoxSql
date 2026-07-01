use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};

use crate::fonts::JETBRAINS_MONO;
use crate::pg::{ResultColumn, ResultSetState, WorkspaceTab};
use crate::session::Session;
use crate::ui::shared::muted;

#[derive(Clone)]
pub(crate) enum WorkspaceEvent {
    TabSelected { index: usize, title: String },
    RunRequested { sql: String },
    CancelRequested,
    ExplainRequested { sql: String },
}

pub(crate) struct Workspace {
    tabs: Vec<WorkspaceTab>,
    pub(crate) active_tab: usize,
    editor: SqlEditorState,
    session: Entity<Session>,
}

impl EventEmitter<WorkspaceEvent> for Workspace {}

impl Workspace {
    pub(crate) fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();

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
            session,
        }
    }

    pub(crate) fn current_sql(&self) -> String {
        self.editor.text.to_string()
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let result = self.session.read(cx).result.clone();

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
                    .child(result.render(cx)),
            )
    }
}

impl Workspace {
    fn render_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
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

    fn render_tab(&self, ix: usize, cx: &mut Context<Self>) -> impl IntoElement {
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
            .hover(|el| el.bg(cx.theme().secondary_hover))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.active_tab = ix;
                cx.emit(WorkspaceEvent::TabSelected {
                    index: ix,
                    title: click_title.clone(),
                });
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

    fn render_query_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let query_summary = self.session.read(cx).query_status.summary();

        h_flex()
            .h(px(32.))
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(self.action_button("Run", cx))
            .child(self.action_button("Stop", cx))
            .child(self.action_button("Explain", cx))
            .child(div().flex_1())
            .child(muted(query_summary))
    }

    fn action_button(&self, text: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let action = text.to_string();

        div()
            .id(format!("action-button-{action}"))
            .px_2()
            .py_1()
            .rounded(px(5.))
            .text_sm()
            .cursor_pointer()
            .hover(|el| el.bg(cx.theme().secondary_hover))
            .on_click(cx.listener(move |this, _, _, cx| {
                match action.as_str() {
                    "Run" => cx.emit(WorkspaceEvent::RunRequested {
                        sql: this.current_sql(),
                    }),
                    "Stop" => cx.emit(WorkspaceEvent::CancelRequested),
                    "Explain" => cx.emit(WorkspaceEvent::ExplainRequested {
                        sql: this.current_sql(),
                    }),
                    _ => {}
                }
                cx.notify();
            }))
            .child(text.to_string())
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
                "select id,\n       'user-' || id::text as email,\n       now()::text as created_at\nfrom generate_series(1, 5) as id;\n",
            ),
            current_line: 0,
        }
    }

    fn render(&self, cx: &mut Context<Workspace>) -> impl IntoElement {
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

    fn render_line(&self, ix: usize, cx: &mut Context<Workspace>) -> impl IntoElement {
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
    #[allow(dead_code)]
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

    fn render(&self, cx: &mut Context<Workspace>) -> impl IntoElement {
        if self.columns.is_empty() {
            return v_flex()
                .flex_1()
                .min_h(px(0.))
                .border_t_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .p_3()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Run a query to see results."),
                );
        }

        let headers = self
            .columns
            .iter()
            .map(|column| grid_header(column, cx).into_any_element())
            .collect::<Vec<_>>();
        let rows = self
            .rows
            .iter()
            .map(|row| grid_row(row, cx).into_any_element())
            .collect::<Vec<_>>();

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
                    .children(headers),
            )
            .children(rows)
    }
}

fn grid_header(column: &ResultColumn, cx: &mut Context<Workspace>) -> impl IntoElement {
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

fn grid_row(row: &[String], cx: &mut Context<Workspace>) -> impl IntoElement {
    let cells = row
        .iter()
        .map(|cell| grid_cell(cell, cx).into_any_element())
        .collect::<Vec<_>>();

    h_flex()
        .h(px(30.))
        .border_b_1()
        .border_color(cx.theme().border)
        .children(cells)
}

fn grid_cell(text: &str, cx: &mut Context<Workspace>) -> impl IntoElement {
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
