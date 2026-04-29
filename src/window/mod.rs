use std::ops::Deref;

use gpui::{
    AppContext, Bounds, Context, Entity, IntoElement, ParentElement, PlatformDisplay, Render, Size,
    Styled, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, black, div,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    point, px, white,
};

use item::power_profile::PowerProfile;

mod item;

const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 60.0;

pub struct Window {
    power_profile: Entity<PowerProfile>,
}

impl Window {
    pub fn new(cx: &mut impl AppContext) -> Entity<Self> {
        cx.new(|cx| Self {
            power_profile: cx.new(|cx| PowerProfile::new(cx)),
        })
    }
    pub fn window_options(
        display: Option<impl Deref<Target = impl PlatformDisplay + ?Sized>>,
    ) -> WindowOptions {
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(
                // TODO: I want the window height to fit the content, and the width based on screen width
                if let Some(display) = display.as_ref()
                    && false
                {
                    let mut bounds = display.bounds();
                    bounds.size.height = px(HEIGHT);
                    bounds
                } else {
                    Bounds {
                        origin: point(px(0.0), px(0.0)),
                        size: Size::new(px(WIDTH), px(HEIGHT)),
                    }
                },
            )),
            titlebar: None,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "eucalyptus-twig".to_owned(),
                layer: Layer::Overlay,
                anchor: Anchor::BOTTOM,
                exclusive_zone: Some(px(0.0)),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            display_id: display.as_ref().map(|x| x.id()),
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        }
    }
}

impl Render for Window {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(black())
            .text_color(white())
            .child(self.power_profile.clone())
    }
}
