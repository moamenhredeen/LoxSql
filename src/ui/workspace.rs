use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, h_flex,
    input::{Input, InputState},
    table::{Column, DataTable, TableDelegate, TableState},
    v_flex,
};

use crate::pg::ResultSetState;
use crate::session::Session;
use crate::ui::shared::muted;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum WorkspaceTab {
    Query { title: String },
    ObjectPreview { title: String },
    TableData { title: String },
    Explain { title: String },
    Activity,
    Locks,
}

impl WorkspaceTab {
    pub(crate) fn title(&self) -> &str {
        match self {
            WorkspaceTab::Query { title }
            | WorkspaceTab::ObjectPreview { title }
            | WorkspaceTab::TableData { title }
            | WorkspaceTab::Explain { title } => title,
            WorkspaceTab::Activity => "activity",
            WorkspaceTab::Locks => "locks",
        }
    }
}

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
    editor: Entity<InputState>,
    results_table: Entity<TableState<ResultsTableDelegate>>,
    session: Entity<Session>,
}

impl EventEmitter<WorkspaceEvent> for Workspace {}

impl Workspace {
    pub(crate) fn new(
        session: Entity<Session>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("sql")
                .line_number(true)
                .placeholder("Write SQL here, then press Run")
        });

        let results_table =
            cx.new(|cx| TableState::new(ResultsTableDelegate::default(), window, cx));

        cx.observe(&session, |this: &mut Workspace, session, cx| {
            let result = session.read(cx).result.clone();
            this.results_table.update(cx, |table, cx| {
                table.delegate_mut().set_result(result);
                table.refresh(cx);
            });
            cx.notify();
        })
        .detach();

        Self {
            tabs: vec![WorkspaceTab::Query {
                title: "scratch.sql".into(),
            }],
            active_tab: 0,
            editor,
            results_table,
            session,
        }
    }

    pub(crate) fn current_sql(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_string()
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w(px(0.))
            .child(self.render_tabs(cx))
            .child(self.render_query_toolbar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .child(self.render_editor(cx))
                    .child(self.render_results(cx)),
            )
    }
}

impl Workspace {
    fn render_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = (0..self.tabs.len())
            .map(|ix| self.render_tab(ix, cx).into_any_element())
            .collect::<Vec<_>>();

        h_flex()
            .h(px(31.))
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .children(tabs)
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
                    "Run" => {
                        let sql = this.current_sql(cx);
                        cx.emit(WorkspaceEvent::RunRequested { sql });
                    }
                    "Stop" => cx.emit(WorkspaceEvent::CancelRequested),
                    "Explain" => {
                        let sql = this.current_sql(cx);
                        cx.emit(WorkspaceEvent::ExplainRequested { sql });
                    }
                    _ => {}
                }
                cx.notify();
            }))
            .child(text.to_string())
    }

    fn render_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .child(Input::new(&self.editor).h_full().appearance(false)),
            )
    }

    fn render_results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_columns = !self
            .results_table
            .read(cx)
            .delegate()
            .result
            .columns
            .is_empty();

        v_flex()
            .flex_1()
            .min_h(px(0.))
            .border_t_1()
            .border_color(cx.theme().border)
            .when(!has_columns, |el| {
                el.child(
                    div()
                        .p_3()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Run a query to see results."),
                )
            })
            .when(has_columns, |el| {
                el.child(
                    DataTable::new(&self.results_table)
                        .bordered(false)
                        .stripe(true)
                        .small(),
                )
            })
    }
}

#[derive(Default)]
struct ResultsTableDelegate {
    result: ResultSetState,
    columns: Vec<Column>,
}

impl ResultsTableDelegate {
    fn set_result(&mut self, result: ResultSetState) {
        self.columns = result
            .columns
            .iter()
            .enumerate()
            .map(|(ix, column)| {
                Column::new(
                    ix.to_string(),
                    format!("{} · {}", column.name, column.pg_type),
                )
                .width(180.)
            })
            .collect();
        self.result = result;
    }
}

impl TableDelegate for ResultsTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.result.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        self.result
            .rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default()
    }
}
