use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

use crate::pg::PgEvent;

pub(crate) struct BottomPanel {
    events: Vec<PgEvent>,
}

impl BottomPanel {
    pub(crate) fn sample() -> Self {
        Self {
            events: vec![
                PgEvent::Notice("NOTICE: relation users was scanned".into()),
                PgEvent::QueryCompleted {
                    rows: 42,
                    elapsed_ms: 82,
                },
            ],
        }
    }

    pub(crate) fn push_event(&mut self, event: PgEvent) {
        self.events.push(event);
        if self.events.len() > 64 {
            self.events.remove(0);
        }
    }
}

impl Render for BottomPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let last_event = self
            .events
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
            .child(status_item("app_db", cx))
            .child(status_item("public", cx))
            .child(status_item("PostgreSQL 17", cx))
            .child(status_item("42 rows", cx))
            .child(status_item("82 ms", cx))
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
