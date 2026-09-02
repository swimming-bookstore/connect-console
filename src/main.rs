//! Operator console. Same Postgres as the plane. No App.

mod api;
mod cms;
mod csrf;

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use connect_control_plane::store::Store;
use tower_cookies::CookieManagerLayer;

#[derive(Parser)]
#[command(name = "connect-console", about = "Organizations, people, machines, ACL")]
struct Cli {
    /// Shared TOML. Default `CONNECT_CONFIG` or `/etc/connect/connect.toml`.
    #[arg(long, env = "CONNECT_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long)]
    database_url: Option<String>,
    #[arg(long)]
    bind: Option<SocketAddr>,
    /// Serve WASM from this directory instead of the baked build (scripts/watch-ui.sh).
    #[arg(long)]
    ui_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "connect_console=info".into()),
        )
        .compact()
        .init();
    let cli = Cli::parse();
    let cfg = connect_control_plane::cfg::Cfg::load(cli.config.as_deref())?;
    let url = cfg.database_url(cli.database_url)?;
    let bind = connect_control_plane::cfg::addr(
        cli.bind,
        cfg.console.bind.as_deref(),
        "127.0.0.1:3040",
    )?;
    let store = Store::connect(&url).await?;
    let api = api::router(api::App { store });
    let mut app = Router::new().merge(api);
    if let Some(dir) = cli.ui_dir {
        tracing::info!(dir = %dir.display(), "ui from directory");
        app = app
            .route("/pkg/connect_console_ui.js", get({
                let dir = dir.clone();
                move || serve_ui(dir.clone(), "connect_console_ui.js", "text/javascript; charset=utf-8")
            }))
            .route("/pkg/connect_console_ui_bg.wasm", get({
                let dir = dir.clone();
                move || serve_ui(dir.clone(), "connect_console_ui_bg.wasm", "application/wasm")
            }));
    } else {
        app = app
            .route("/pkg/connect_console_ui.js", get(js))
            .route("/pkg/connect_console_ui_bg.wasm", get(wasm));
    }
    let app = app.fallback(index).layer(CookieManagerLayer::new());
    tracing::info!(bind = %bind, "console");
    let lis = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(lis, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX)
}

async fn serve_ui(dir: PathBuf, name: &'static str, ctype: &'static str) -> Response {
    let path = dir.join(name);
    match std::fs::read(&path) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, ctype),
                (header::CACHE_CONTROL, "no-store"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (axum::http::StatusCode::NOT_FOUND, "ui missing").into_response(),
    }
}

async fn js() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_str!(concat!(env!("OUT_DIR"), "/webui/connect_console_ui.js")),
    )
        .into_response()
}

async fn wasm() -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/wasm"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        include_bytes!(concat!(env!("OUT_DIR"), "/webui/connect_console_ui_bg.wasm")).as_slice(),
    )
        .into_response()
}

const INDEX: &str = r#"<!DOCTYPE html>
<meta charset="utf-8"><title>connect console</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap">
<body>
<p id="boot">loading…</p>
<script type="module">
  import init from "/pkg/connect_console_ui.js";
  try {
    await init({ module_or_path: "/pkg/connect_console_ui_bg.wasm" });
    document.getElementById("boot")?.remove();
  } catch (e) {
    document.getElementById("boot").textContent = String(e && e.message ? e.message : e);
  }
</script>
"#;
