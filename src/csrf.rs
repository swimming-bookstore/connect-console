//! Double-submit CSRF. Cookie is httponly; the WASM sends `x-csrf-token`.
//! GET /api/auth/csrf issues (or echoes) the token. POSTs must match.

use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use subtle::ConstantTimeEq;
use tower_cookies::{Cookie, Cookies};

pub const COOKIE: &str = "connect_csrf";
pub const HEADER: &str = "x-csrf-token";

pub fn token(cookies: &Cookies) -> String {
    if let Some(c) = cookies.get(COOKIE) {
        let v = c.value().to_string();
        if v.len() == 64 && v.chars().all(|c| c.is_ascii_hexdigit()) {
            return v;
        }
    }
    let t = generate();
    set(cookies, &t);
    t
}

pub fn set(cookies: &Cookies, token: &str) {
    let mut c = Cookie::new(COOKIE, token.to_string());
    c.set_http_only(true);
    c.set_path("/");
    c.set_same_site(tower_cookies::cookie::SameSite::Lax);
    c.set_max_age(time::Duration::days(14));
    cookies.add(c);
}

pub async fn gate(cookies: Cookies, req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.method() == Method::GET || req.method() == Method::HEAD || req.method() == Method::OPTIONS
    {
        return Ok(next.run(req).await);
    }
    if !same_origin(&req) {
        return Err(StatusCode::FORBIDDEN);
    }
    let Some(cookie) = cookies.get(COOKIE) else {
        return Err(StatusCode::FORBIDDEN);
    };
    let Some(hdr) = req.headers().get(HEADER).and_then(|v| v.to_str().ok()) else {
        return Err(StatusCode::FORBIDDEN);
    };
    if !eq(cookie.value(), hdr) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

fn same_origin(req: &Request) -> bool {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if host.is_empty() {
        return true;
    }
    if let Some(origin) = req.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        return origin_host(origin) == host;
    }
    if let Some(referer) = req.headers().get(header::REFERER).and_then(|v| v.to_str().ok()) {
        return origin_host(referer) == host;
    }
    true
}

fn origin_host(url: &str) -> &str {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    rest.split('/').next().unwrap_or(rest)
}

fn generate() -> String {
    let mut raw = [0u8; 32];
    let _ = getrandom::getrandom(&mut raw);
    hex::encode(raw)
}

fn eq(a: &str, b: &str) -> bool {
    bool::from(a.as_bytes().ct_eq(b.as_bytes()))
}

pub fn attach(mut res: Response, token: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(token) {
        res.headers_mut().insert(HEADER, v);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_ok() {
        assert!(eq("aa", "aa"));
        assert!(!eq("aa", "ab"));
        assert!(!eq("aa", "aaa"));
        assert!(eq("", ""));
    }

    #[test]
    fn origin_host_strips() {
        assert_eq!(origin_host("http://127.0.0.1:3040/login"), "127.0.0.1:3040");
        assert_eq!(origin_host("https://console.example/x"), "console.example");
    }

    #[test]
    fn generate_is_64_hex() {
        let t = generate();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
