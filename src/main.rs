mod app;
mod fonts;
mod pg;
mod ui;

use gpui::*;
use gpui_component::{Root, Theme, TitleBar};

use crate::app::AppShell;
use crate::fonts::JETBRAINS_MONO;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);
        fonts::register(cx).expect("failed to register JetBrains Mono");
        {
            let theme = Theme::global_mut(cx);
            theme.font_family = JETBRAINS_MONO.into();
            theme.mono_font_family = JETBRAINS_MONO.into();
        }

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    window_decorations: Some(WindowDecorations::Client),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| AppShell::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("Failed to open window");
        })
        .detach();
    });
}
