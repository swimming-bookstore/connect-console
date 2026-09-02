//! Fetch helpers. POSTs send `x-csrf-token` from GET /api/auth/csrf.

use gloo_net::http::Request;
use serde::Deserialize;
use std::cell::RefCell;

thread_local! {
    static CSRF: RefCell<String> = RefCell::new(String::new());
}

async fn csrf() -> String {
    let cached = CSRF.with(|g| g.borrow().clone());
    if !cached.is_empty() {
        return cached;
    }
    refresh_csrf().await
}

async fn refresh_csrf() -> String {
    let t = Request::get("/api/auth/csrf")
        .send()
        .await
        .ok()
        .and_then(|r| r.headers().get("x-csrf-token"))
        .unwrap_or_default();
    CSRF.with(|g| *g.borrow_mut() = t.clone());
    t
}

pub async fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Option<T> {
    let r = Request::get(url).send().await.ok()?;
    r.json().await.ok()
}

pub async fn post(url: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    post_once(url, body, false).await
}

async fn post_once(
    url: &str,
    body: &serde_json::Value,
    retried: bool,
) -> Result<serde_json::Value, String> {
    let token = csrf().await;
    let req = Request::post(url)
        .header("content-type", "application/json")
        .header("x-csrf-token", &token)
        .body(body.to_string())
        .map_err(|e| e.to_string())?;
    let r = req.send().await.map_err(|e| e.to_string())?;
    if r.status() == 403 && !retried {
        let _ = refresh_csrf().await;
        return Box::pin(post_once(url, body, true)).await;
    }
    let t = r.text().await.map_err(|e| e.to_string())?;
    if !r.ok() {
        return Err(t);
    }
    Ok(serde_json::from_str(&t).unwrap_or(serde_json::json!({"ok": true})))
}

pub fn event_value(ev: &leptos::ev::Event) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        ev.target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|el| el.value())
            .or_else(|| {
                ev.target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                    .map(|el| el.value())
            })
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        String::new()
    }
}
