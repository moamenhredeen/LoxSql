use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

use crate::pg::{PgEvent, QueryStatus};
use crate::session::Session;

pub(crate) struct BottomPanel {
    session: Entity<Session>,
}

impl BottomPanel {
    pub(crate) fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();

        Self { session }
    }
}

impl Render for BottomPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.session.read(cx);
        let database = session.database.clone().unwrap_or_else(|| "—".into());
        let server = session
            .server_version
            .clone()
            .map(|version| format!("PostgreSQL {version}"))
            .unwrap_or_else(|| "not connected".into());
        let query = match &session.query_status {
            QueryStatus::Idle => "idle".to_string(),
            QueryStatus::Running => "running…".to_string(),
            QueryStatus::Completed { rows, elapsed_ms } => {
                format!("{rows} rows · {elapsed_ms} ms")
            }
            QueryStatus::Failed { message } => format!("error · {message}"),
        };
        let last_event = session
            .event_log
            .last()
            .map(PgEvent::summary)
            .unwrap_or_else(|| "Ready".into());

        h_flex()
            .h(px(24.))
            .px_2()
            .gap_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(status_item(database, cx))
            .child(status_item(server, cx))
            .child(status_item(query, cx))
            .child(div().flex_1())
            .child(status_item(last_event, cx))
    }
}

fn status_item(text: impl Into<SharedString>, cx: &mut Context<BottomPanel>) -> impl IntoElement {
    div()
        .h_full()
        .px_1()
        .flex()
        .items_center()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .cursor_pointer()
        .hover(|el| el.bg(cx.theme().secondary_hover))
        .child(text.into())
}
