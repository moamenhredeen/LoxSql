use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

use crate::app::AppShell;
use crate::pg::PgEvent;
use crate::ui::shared::status_item;

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

    pub(crate) fn render(&self, cx: &mut Context<AppShell>) -> impl IntoElement {
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
            .child(status_item(self.events[0].summary(), cx))
    }
}
