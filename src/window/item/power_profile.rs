use futures::StreamExt;
use gpui::{
    AsyncApp, Context, IntoElement, ParentElement, Render, Styled, WeakEntity, Window, black, div,
    white,
};
use zbus::{Connection, proxy};

pub struct PowerProfile {
    error_message: Option<String>,
    active_profile: Option<String>,
}

impl PowerProfile {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async |this, cx| task(this, cx).await).detach();

        Self {
            error_message: None,
            active_profile: None,
        }
    }
}

impl Render for PowerProfile {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(e) = &self.error_message {
            div().child(e.clone()).into_any_element()
        } else {
            div()
                .flex()
                .children(
                    [
                        ("power-saver", "\u{ec1a}"),
                        ("balanced", "\u{e9e4}"),
                        ("performance", "\u{eb9b}"),
                    ]
                    .into_iter()
                    .map(|(profile, icon)| {
                        div()
                            .flex()
                            .flex_col()
                            .bg(
                                if self.active_profile.as_ref().is_some_and(|x| profile == x) {
                                    white()
                                } else {
                                    black()
                                },
                            )
                            .text_color(
                                if self.active_profile.as_ref().is_some_and(|x| profile == x) {
                                    black()
                                } else {
                                    white()
                                },
                            )
                            .child(div().font_family("Material Symbols Rounded").child(icon))
                            .child(profile)
                    }),
                )
                .child(format!("self.active_profile = {:?}", self.active_profile))
                .into_any_element()
        }
    }
}

async fn task(this: WeakEntity<PowerProfile>, cx: &mut AsyncApp) {
    println!("hello from task");

    let connection = match Connection::system().await {
        Ok(x) => x,
        Err(e) => {
            let _ = this.update(cx, |this, cx| {
                this.error_message = Some(format!("Failed to connect to system bus: {e}"));
                cx.notify();
            });
            tracing::error!(error = %e, "Failed to connect to system bus");
            return;
        }
    };
    let proxy = match PowerProfilesProxy::new(&connection).await {
        Ok(x) => x,
        Err(e) => {
            let _ = this.update(cx, |this, cx| {
                this.error_message = Some(format!("Failed to create properties proxy: {e}"));
                cx.notify();
            });
            tracing::error!(error = %e, "Failed to create properties proxy");
            return;
        }
    };
    let mut stream = proxy.receive_active_profile_changed().await;

    while let Some(new_active_profile) = stream.next().await {
        println!("get a message from active profile stream...");
        match new_active_profile.get().await {
            Ok(new_active_profile) => {
                tracing::info!(new_active_profile, "Power profile changed");
                let _ = this.update(cx, |this, cx| {
                    this.active_profile = Some(new_active_profile.clone());
                    cx.notify();
                });
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to get new ActiveProfile");
            }
        }
    }
    tracing::warn!("Receive ActiveProfile stream ended");
}

// <https://upower.pages.freedesktop.org/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html>
#[proxy(
    interface = "org.freedesktop.UPower.PowerProfiles",
    default_service = "org.freedesktop.UPower.PowerProfiles",
    default_path = "/org/freedesktop/UPower/PowerProfiles"
)]
trait PowerProfiles {
    fn hold_profile(
        &self,
        profile: String,
        reason: String,
        application_id: String,
    ) -> zbus::Result<u32>;
    fn release_profile(&self, cookie: u32) -> zbus::Result<()>;
    fn set_action_enabled(&self, action: String, enabled: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    fn profile_released(&self, cookie: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_active_profile(&self, active_profile: &str) -> zbus::Result<()>;
    #[zbus(property)]
    fn performance_degraded(&self) -> zbus::Result<String>;
    // TODO: fix this: <https://upower.pages.freedesktop.org/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html#gdbus-property-org-freedesktop-UPower-PowerProfiles.Profiles>
    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<String>;
}
