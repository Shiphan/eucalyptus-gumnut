use std::{cell::LazyCell, env, ffi::OsStr, path::PathBuf, time::Duration};

use futures::{
    AsyncReadExt, FutureExt, SinkExt,
    channel::{
        self,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    io::BufReader,
    select,
};
use gpui::{App, AppContext, AsyncApp, QuitMode};
use smol::net::unix::UnixListener;

use crate::window::Window;

pub const SOCKET_PATH: LazyCell<PathBuf> = LazyCell::new(|| {
    let socket_path = OsStr::new("eucalyptus-gumnut/daemon.sock");
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(runtime_dir) => [runtime_dir.as_os_str(), socket_path].into_iter().collect(),
        None => [
            OsStr::new(&format!("/run/user/{}", nix::unistd::getuid())),
            socket_path,
        ]
        .into_iter()
        .collect(),
    }
});

pub fn start() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::Explicit)
        .run(|cx: &mut App| {
            let (tx, rx) = channel::mpsc::unbounded();
            println!("hi");
            cx.background_spawn(daemon(tx)).detach();
            cx.spawn(|cx: &mut AsyncApp| {
                timer(
                    5.0,
                    rx,
                    {
                        let cx = cx.clone();
                        move || {
                            cx.update(|cx| {
                                let displays = cx.displays();
                                for display in displays {
                                    cx.open_window(
                                        Window::window_options(Some(display)),
                                        // FIXME: this will spawn a new task every time a window is opened, need to find a way to only fix this,
                                        // maybe create only one "app",
                                        // or just spawn the task without use gpui's api
                                        |window, cx| Window::build_root_view(window, cx),
                                    )
                                    .unwrap();
                                }
                            })
                        }
                    },
                    {
                        let cx = cx.clone();
                        move || {
                            cx.update(|cx| {
                                for window in cx.windows() {
                                    window
                                        .update(cx, |_view, window, _cx| window.remove_window())
                                        .unwrap();
                                }
                            })
                        }
                    },
                )
            })
            .detach();
        });
}

async fn daemon(mut tx: UnboundedSender<TimerAction>) {
    println!("try to bind {SOCKET_PATH:?}");
    if let Some(parent) = SOCKET_PATH.parent() {
        smol::fs::create_dir_all(parent).await.unwrap();
    }
    // FIXME: when the daemon dies, we need to clean up socket so that next one can use the same path
    // also need to handle when another daemon is running and the socket path is used by it (maybe need a "ping")
    let listener = UnixListener::bind(SOCKET_PATH.as_path()).unwrap();
    println!("start listening at {SOCKET_PATH:?}");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut stream = BufReader::new(stream);
                let mut buf = String::new();
                stream.read_to_string(&mut buf).await.unwrap();
                match buf.as_str() {
                    "activate/power_profile" => {
                        println!("get activate/power_profile");
                        tx.send(TimerAction::Activate).await.unwrap();
                    }
                    _ => eprintln!("???"),
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }
}

enum TimerAction {
    Activate,
    Cancel,
}

async fn timer(
    secs: f64,
    mut rx: UnboundedReceiver<TimerAction>,
    start_callback: impl Fn(),
    end_callback: impl Fn(),
) {
    loop {
        match rx.recv().await {
            Ok(TimerAction::Activate) => (),
            Ok(TimerAction::Cancel) => continue,
            Err(e) => {
                eprintln!("Error: {e}");
                return;
            }
        }
        start_callback();
        loop {
            select! {
                 action = rx.recv() => match action {
                    Ok(TimerAction::Activate) => (),
                    Ok(TimerAction::Cancel) => break,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return;
                    },
                },
                _ = smol::Timer::after(Duration::from_secs_f64(secs)).fuse() => break,
            }
        }
        end_callback();
    }
}
