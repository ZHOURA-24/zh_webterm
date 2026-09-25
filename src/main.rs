use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tokio::sync::mpsc;

#[derive(Clone)]
struct AppState {
    password: Option<String>,
}

#[derive(Deserialize)]
struct AuthQuery {
    password: Option<String>,
}

#[derive(Serialize)]
struct AuthStatus {
    auth_required: bool,
}

#[derive(Deserialize, Debug)]
struct Resize {
    cols: u16,
    rows: u16,
}

fn load_config() -> (u16, Option<String>) {
    let mut port = std::env::var("PORT").ok().and_then(|v| v.parse().ok());
    let mut password = std::env::var("WEBTERM_PASSWORD").ok();

    let exe_env = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("zh_webterm.env")));
    let paths: &[&std::path::Path] = &[
        exe_env.as_deref().unwrap_or(std::path::Path::new("")),
        std::path::Path::new("/etc/zh_webterm.env"),
        std::path::Path::new(".env"),
    ];

    for path in paths {
        if let Ok(s) = std::fs::read_to_string(path) {
            for (k, v) in s.lines().filter_map(|l| l.split_once('=')) {
                let (k, v) = (k.trim(), v.trim().trim_matches(['"', '\'']));
                if k.starts_with('#') {
                    continue;
                }
                if k == "PORT" && port.is_none() {
                    port = v.parse().ok();
                } else if k == "WEBTERM_PASSWORD" && password.is_none() {
                    password = Some(v.to_string());
                }
            }
            break;
        }
    }

    (
        port.unwrap_or(2424),
        password.filter(|p| !p.trim().is_empty()),
    )
}

#[tokio::main]
async fn main() {
    let (mut port, password) = load_config();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" => {
                if let Some(val) = args.next() {
                    port = val.parse().unwrap_or(2424);
                }
            }
            _ => {
                println!("Usage: zh_webterm [-p <port>]");
                return;
            }
        }
    }

    let router = Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../static/index.html")) }),
        )
        .route(
            "/auth-status",
            get(|state: State<AppState>| async move {
                Json(AuthStatus {
                    auth_required: state.password.is_some(),
                })
            }),
        )
        .route(
            "/ws",
            get(
                |state: State<AppState>, query: Query<AuthQuery>, ws: WebSocketUpgrade| async move {
                    if let Some(ref required_pw) = state.password {
                        if query.password.as_deref() != Some(required_pw) {
                            return StatusCode::UNAUTHORIZED.into_response();
                        }
                    }
                    ws.on_upgrade(handle_ws).into_response()
                },
            ),
        )
        .with_state(AppState { password });

    let addr = format!("0.0.0.0:{port}");
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn handle_ws(mut socket: WebSocket) {
    let pty = native_pty_system();
    let pair = match pty.openpty(PtySize::default()) {
        Ok(pair) => pair,
        Err(_) => return,
    };

    let shell = if cfg!(windows) {
        "powershell.exe"
    } else {
        "bash"
    };

    let mut cmd = CommandBuilder::new(shell);
    if !cfg!(windows) {
        cmd.args(["-l"]);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    if pair.slave.spawn_command(cmd).is_err() {
        return;
    }

    let mut reader = match pair.master.try_clone_reader() {
        Ok(reader) => reader,
        Err(_) => return,
    };
    let mut writer = match pair.master.take_writer() {
        Ok(writer) => writer,
        Err(_) => return,
    };

    let master = pair.master;

    let (sender, mut receiver) = mpsc::channel::<Vec<u8>>(100);

    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 || sender.blocking_send(buffer[..count].to_vec()).is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            Some(data) = receiver.recv() => {
                if socket.send(Message::Binary(data.into())).await.is_err() {
                    break;
                }
            }
            Some(Ok(msg)) = socket.recv() => match msg {
                Message::Text(text) => {
                    if let Ok(resize) = serde_json::from_str::<Resize>(&text) {
                        let _ = master.resize(PtySize {
                            rows: resize.rows,
                            cols: resize.cols,
                            pixel_width: 0,
                            pixel_height: 0,
                        });
                    } else {
                        let _ = writer.write_all(text.as_bytes());
                    }
                }
                Message::Binary(bytes) => {
                    let _ = writer.write_all(&bytes);
                }
                Message::Close(_) => break,
                _ => {}
            },
            else => break,
        }
    }
}
