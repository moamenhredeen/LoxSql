use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt,
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    popover::Popover,
    scroll::ScrollableElement,
    v_flex,
};

use crate::{
    pg::ConnectionProfile,
    ui::shared::{label, muted},
};

#[derive(Clone)]
pub(crate) enum ConnectionPickerEvent {
    ConnectionSelected(ConnectionProfile),
}

pub struct ConnectionPicker {
    profiles: Vec<ConnectionProfile>,
    selected_profile: Option<String>,
    search_state: Entity<InputState>,
    open: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ConnectionPickerEvent> for ConnectionPicker {}

impl Render for ConnectionPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let picker = cx.entity().downgrade();

        Popover::new("connection-picker")
            .anchor(Anchor::TopLeft)
            .appearance(false)
            .open(self.open)
            .on_open_change(cx.listener(|picker, open, _, cx| {
                picker.open = *open;
                cx.notify();
            }))
            .trigger(self.render_trigger(cx))
            .content(move |_, _, cx| {
                picker
                    .update(cx, |picker, cx| picker.render_popover(cx))
                    .unwrap_or_else(|_| div().into_any_element())
            })
            .into_any_element()
    }
}

impl ConnectionPicker {
    pub fn new(window: &mut Window, cx: &mut Context<ConnectionPicker>) -> Self {
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search connections"));
        let _subscriptions = vec![cx.subscribe(&search_state, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })];
        Self {
            profiles: vec![ConnectionProfile::local_dev()],
            selected_profile: None,
            search_state,
            open: false,
            _subscriptions,
        }
    }

    pub(crate) fn set_selected_profile(&mut self, profile_id: Option<String>) {
        self.selected_profile = profile_id;
    }

    fn render_trigger(&self, cx: &mut Context<ConnectionPicker>) -> Button {
        let selected_profile = self
            .selected_profile
            .as_deref()
            .and_then(|selected_id| {
                self.profiles
                    .iter()
                    .find(|profile| profile.id == selected_id)
                    .map(|profile| profile.name.as_str())
            })
            .or_else(|| self.profiles.first().map(|profile| profile.name.as_str()))
            .unwrap_or("local-dev");

        Button::new("connection-switcher")
            .h(px(24.))
            .px_2()
            .gap_1()
            .rounded(px(5.))
            .text_sm()
            .bg(cx.theme().secondary.opacity(0.45))
            .child(div().text_color(cx.theme().muted_foreground).child("*"))
            .child(label(selected_profile))
            .child(muted("v"))
    }

    fn render_popover(&mut self, cx: &mut Context<ConnectionPicker>) -> AnyElement {
        let selected_profile = self.selected_profile.clone();
        let query = self.search_state.read(cx).value().to_lowercase();
        let options = self
            .profiles
            .clone()
            .into_iter()
            .filter(|profile| query.is_empty() || profile.name.to_lowercase().contains(&query))
            .map(|profile| {
                let selected = selected_profile.as_deref() == Some(profile.id.as_str());
                self.connection_option(profile, selected, cx)
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        div()
            .v_flex()
            .w(px(340.))
            .max_h(px(240.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .shadow_lg()
            .overflow_hidden()
            .child(
                div()
                    .p_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(Input::new(&self.search_state).appearance(false).small()),
            )
            .child(v_flex().overflow_y_scrollbar().children(options))
            .into_any_element()
    }

    fn connection_option(
        &self,
        profile: ConnectionProfile,
        selected: bool,
        cx: &mut Context<ConnectionPicker>,
    ) -> impl IntoElement {
        let name = profile.name.clone();
        let id = profile.id.clone();
        let detail = profile
            .database
            .clone()
            .unwrap_or_else(|| profile.host.clone());

        h_flex()
            .id(format!("connection-option-{id}"))
            .gap_2()
            .px_1()
            .cursor_pointer()
            .hover(|el| el.bg(cx.theme().secondary_hover))
            .on_click(cx.listener(move |picker, _, _, cx| {
                picker.open = false;
                cx.emit(ConnectionPickerEvent::ConnectionSelected(profile.clone()));
                cx.notify();
            }))
            .when(selected, |el| el.bg(cx.theme().secondary.opacity(0.8)))
            .child(
                div()
                    .w(px(12.))
                    .text_color(cx.theme().muted_foreground)
                    .child(if selected {
                        Icon::new(IconName::Star)
                    } else {
                        Icon::empty()
                    }),
            )
            .child(
                v_flex()
                    .gap_0()
                    .p_1()
                    .child(label(name).text_sm())
                    .child(muted(detail).text_xs()),
            )
    }
}
