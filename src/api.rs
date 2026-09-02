//! JSON API over the plane store. No App, no Session.

use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use crate::cms::{self, Who};
use connect_control_plane::store::{Kind, Store};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};

const COOKIE: &str = "connect_console";

#[derive(Clone)]
pub struct App {
    pub store: Store,
}

pub fn router(app: App) -> Router {
    let state = Arc::new(app);
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/me", get(auth_me))
        .route("/api/auth/lab", get(auth_lab))
        .route("/api/auth/register", post(auth_register))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/profile", post(auth_profile))
        .route("/api/orgs", get(orgs).post(org_add))
        .route("/api/orgs/{org}/wall", get(wall))
        .route("/api/orgs/{org}/agents", post(agent_add))
        .route("/api/orgs/{org}/agents/{name}/revoke", post(agent_revoke))
        .route("/api/orgs/{org}/agents/{name}/role", post(agent_role))
        .route("/api/orgs/{org}/agents/{name}/access", get(box_access))
        .route("/api/orgs/{org}/agents/{name}/owner", post(box_owner))
        .route("/api/orgs/{org}/settings", get(org_settings).post(org_settings_set))
        .route("/api/orgs/{org}/acl/grant", post(acl_grant))
        .route("/api/orgs/{org}/acl/open", post(acl_open))
        .route("/api/auth/csrf", get(auth_csrf))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(middleware::from_fn(crate::csrf::gate))
        .with_state(state)
}

async fn who_of(app: &App, cookies: &Cookies) -> Result<Who, StatusCode> {
    let Some(c) = cookies.get(COOKIE) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some((email, tenant, name, role, personal, _display)) = app.store.console_who(c.value()).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let scope = app
        .store
        .console_scope(c.value(), if personal { "personal" } else { &tenant })
        .await
        .unwrap_or_else(|| tenant.clone());
    Ok(Who {
        email,
        org: if personal { "personal".into() } else { tenant },
        name,
        role,
        personal,
        scope,
    })
}

async fn scoped(app: &App, cookies: &Cookies, org: &str) -> Result<Who, (StatusCode, String)> {
    let mut who = who_of(app, cookies)
        .await
        .map_err(|s| (s, "sign in".into()))?;
    if org != "personal" && who.org != org {
        return Err((StatusCode::FORBIDDEN, "not your organization".into()));
    }
    let token = cookies.get(COOKIE).map(|c| c.value().to_string()).unwrap_or_default();
    let Some(scope) = app.store.console_scope(&token, org).await else {
        return Err((StatusCode::FORBIDDEN, "not your organization".into()));
    };
    who.scope = scope;
    who.org = if org == "personal" {
        "personal".into()
    } else {
        org.into()
    };
    who.personal = org == "personal";
    Ok(who)
}

fn forbid(msg: &str) -> (StatusCode, String) {
    (StatusCode::FORBIDDEN, msg.into())
}

fn admin_ok(who: &Who) -> Result<(), (StatusCode, String)> {
    if who.admin() {
        Ok(())
    } else {
        Err(forbid("admins only"))
    }
}

fn owner_ok(who: &Who) -> Result<(), (StatusCode, String)> {
    if who.owner() {
        Ok(())
    } else {
        Err(forbid("owners only"))
    }
}

fn err(e: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.to_string())
}

fn set_session(cookies: &Cookies, token: String) {
    let mut c = Cookie::new(COOKIE, token);
    c.set_http_only(true);
    c.set_path("/");
    c.set_same_site(tower_cookies::cookie::SameSite::Lax);
    c.set_max_age(time::Duration::days(14));
    cookies.add(c);
}

async fn require_auth(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match req.uri().path() {
        "/api/health"
        | "/api/auth/me"
        | "/api/auth/lab"
        | "/api/auth/csrf"
        | "/api/auth/register"
        | "/api/auth/login"
        | "/api/auth/logout" => return Ok(next.run(req).await),
        _ => {}
    }
    let _ = who_of(&app, &cookies).await?;
    Ok(next.run(req).await)
}

#[derive(Serialize)]
struct Me {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(default)]
    personal: bool,
}

async fn auth_me(State(app): State<Arc<App>>, cookies: Cookies) -> Json<Me> {
    if let Some(c) = cookies.get(COOKIE) {
        if let Some((email, tenant, name, role, personal, display)) = app.store.console_who(c.value()).await {
            return Json(Me {
                email: Some(email),
                org: Some(if personal { "personal".into() } else { tenant }),
                name: Some(name),
                display: Some(display),
                role: Some(role),
                personal,
            });
        }
    }
    Json(Me {
        email: None,
        org: None,
        name: None,
        display: None,
        role: None,
        personal: false,
    })
}

#[derive(Serialize, Deserialize)]
struct LabLogin {
    email: String,
    password: String,
    #[serde(default)]
    org: String,
}

async fn auth_csrf(cookies: Cookies) -> impl axum::response::IntoResponse {
    let t = crate::csrf::token(&cookies);
    crate::csrf::attach(Json(serde_json::json!({"token": t})).into_response(), &t)
}

async fn auth_lab() -> Json<Vec<LabLogin>> {
    let path = std::env::var("CONNECT_CONSOLE_LAB")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let Some(path) = path else {
        return Json(Vec::new());
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Json(Vec::new());
    };
    Json(serde_json::from_str(&text).unwrap_or_default())
}

#[derive(Deserialize)]
struct Creds {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct Register {
    #[serde(default, alias = "tenant")]
    org: String,
    #[serde(default)]
    name: String,
    email: String,
    password: String,
}

fn name_from_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or(email);
    let mut out = String::new();
    for c in local.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        }
    }
    if out.is_empty() || !out.starts_with(|c: char| c.is_ascii_lowercase()) {
        out.insert(0, 'u');
    }
    out.truncate(64);
    out
}

async fn auth_register(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Json(body): Json<Register>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let name = if body.name.trim().is_empty() {
        name_from_email(&body.email)
    } else {
        body.name.clone()
    };
    let token = app
        .store
        .console_register(&body.org, &name, &body.email, &body.password)
        .await
        .map_err(err)?;
    set_session(&cookies, token);
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn auth_login(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Json(body): Json<Creds>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let token = app
        .store
        .console_login(&body.email, &body.password)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "bad email or password".into()))?;
    set_session(&cookies, token);
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn auth_logout(
    State(app): State<Arc<App>>,
    cookies: Cookies,
) -> Json<serde_json::Value> {
    if let Some(c) = cookies.get(COOKIE) {
        let _ = app.store.console_logout(c.value()).await;
    }
    cookies.remove(Cookie::from(COOKIE));
    Json(serde_json::json!({"ok": true}))
}

#[derive(Deserialize)]
struct ProfileIn {
    display: String,
    #[serde(default)]
    password: String,
}

async fn auth_profile(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Json(body): Json<ProfileIn>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = who_of(&app, &cookies)
        .await
        .map_err(|s| (s, "sign in".into()))?;
    let pass = body.password.trim();
    app.store
        .console_set_profile(
            &who.email,
            &body.display,
            if pass.is_empty() { None } else { Some(pass) },
        )
        .await
        .map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Serialize)]
struct TenantOut {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    machines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    online: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    people: Option<u32>,
}

async fn space_out(app: &App, token: &str, name: &str) -> Option<TenantOut> {
    let scope = app.store.console_scope(token, name).await?;
    let agents = app.store.list_agents(&scope).await.unwrap_or_default();
    let live = app.store.list_online(&scope).await.unwrap_or_default();
    let machines = agents.iter().filter(|a| a.kind == Kind::Box && !a.revoked).count() as u32;
    let people = agents.iter().filter(|a| a.kind == Kind::Client && !a.revoked).count() as u32;
    let online = live.iter().filter(|a| a.kind == Kind::Box).count() as u32;
    Some(TenantOut {
        id: scope,
        name: name.into(),
        machines: Some(machines),
        online: Some(online),
        people: Some(people),
    })
}

async fn orgs(
    State(app): State<Arc<App>>,
    cookies: Cookies,
) -> Result<Json<Vec<TenantOut>>, (StatusCode, String)> {
    let who = who_of(&app, &cookies).await.map_err(|s| (s, "sign in".into()))?;
    let token = cookies.get(COOKIE).map(|c| c.value().to_string()).unwrap_or_default();
    let mut out = Vec::new();
    if let Some(p) = space_out(&app, &token, "personal").await {
        out.push(p);
    }
    if !who.org.is_empty() && who.org != "personal" {
        if let Some(c) = space_out(&app, &token, &who.org).await {
            out.push(c);
        }
    }
    Ok(Json(out))
}

#[derive(Deserialize)]
struct TenantIn {
    name: String,
}

async fn org_add(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Json(body): Json<TenantIn>,
) -> Result<Json<TenantOut>, (StatusCode, String)> {
    let who = who_of(&app, &cookies).await.map_err(|s| (s, "sign in".into()))?;
    let name = body.name.trim().to_ascii_lowercase();
    if matches!(name.as_str(), "personal" | "org" | "profile" | "new-organization" | "") {
        return Err((StatusCode::BAD_REQUEST, "reserved name".into()));
    }
    let org = app
        .store
        .console_create_org(&who.email, &name)
        .await
        .map_err(err)?;
    Ok(Json(TenantOut {
        id: String::new(),
        name: org,
        machines: Some(0),
        online: Some(0),
        people: Some(1),
    }))
}

async fn wall(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
) -> Result<Json<cms::Wall>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    cms::wall(&app.store, &who).await.map(Json).map_err(err)
}

#[derive(Deserialize)]
struct AgentIn {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    client: bool,
    #[serde(default)]
    owner: String,
}

#[derive(Serialize)]
struct IssuedOut {
    name: String,
    kind: String,
    id: String,
    token: String,
}

async fn agent_add(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
    Json(body): Json<AgentIn>,
) -> Result<Json<IssuedOut>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    let kind = if body.client { Kind::Client } else { Kind::Box };
    if kind == Kind::Client {
        admin_ok(&who)?;
    }
    let name = if kind == Kind::Client {
        if !body.email.trim().is_empty() {
            name_from_email(&body.email)
        } else if !body.name.trim().is_empty() {
            body.name.clone()
        } else {
            return Err((StatusCode::BAD_REQUEST, "email required".into()));
        }
    } else {
        body.name.clone()
    };
    let owner = if kind == Kind::Box {
        if body.owner.is_empty() {
            if who.admin() {
                String::new()
            } else {
                who.name.clone()
            }
        } else if who.admin() {
            body.owner.clone()
        } else if body.owner == who.name {
            who.name.clone()
        } else {
            return Err(forbid("you can only add machines you own"));
        }
    } else {
        String::new()
    };
    let i = if kind == Kind::Box {
        app.store
            .create_box(&who.scope, &name, &owner)
            .await
            .map_err(err)?
    } else {
        app.store
            .create_agent(&who.scope, &name, kind)
            .await
            .map_err(err)?
    };
    Ok(Json(IssuedOut {
        name: i.agent.name,
        kind: kind.as_str().into(),
        id: i.agent.agent_id.to_string(),
        token: i.token,
    }))
}

async fn agent_revoke(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path((org, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    let a = app.store.agent_in(&who.scope, &name).await.map_err(err)?;
    if a.kind == Kind::Client {
        admin_ok(&who)?;
        if a.org_role == "owner" && who.role != "owner" {
            return Err(forbid("owners only"));
        }
    } else if !who.may_box(&a.owner) {
        return Err(forbid("not your machine"));
    }
    app.store.revoke_agent(&who.scope, &name).await.map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct RoleIn {
    role: String,
}

async fn agent_role(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path((org, name)): Path<(String, String)>,
    Json(body): Json<RoleIn>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    owner_ok(&who)?;
    app.store
        .set_org_role(&who.scope, &name, &body.role)
        .await
        .map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Serialize)]
struct BoxAccess {
    org: bool,
    people: Vec<String>,
}

async fn box_access(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path((org, name)): Path<(String, String)>,
) -> Result<Json<BoxAccess>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    let a = app.store.agent_in(&who.scope, &name).await.map_err(err)?;
    if a.kind != Kind::Box {
        return Err((StatusCode::BAD_REQUEST, "not a box".into()));
    }
    if !who.may_box(&a.owner) {
        return Err(forbid("not your machine"));
    }
    let (restricted, people) = app
        .store
        .list_box_clients(&who.scope, &name)
        .await
        .map_err(err)?;
    Ok(Json(BoxAccess {
        org: !restricted,
        people,
    }))
}

#[derive(Deserialize)]
struct OwnerIn {
    owner: String,
}

async fn box_owner(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path((org, name)): Path<(String, String)>,
    Json(body): Json<OwnerIn>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    admin_ok(&who)?;
    let owner = if body.owner.trim().is_empty() || body.owner == "organization" {
        ""
    } else {
        body.owner.as_str()
    };
    app.store
        .set_box_owner(&who.scope, &name, owner)
        .await
        .map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Serialize, Deserialize)]
struct OrgSettings {
    open: bool,
}

async fn org_settings(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
) -> Result<Json<OrgSettings>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    let open = app.store.org_open_access(&who.scope).await.map_err(err)?;
    Ok(Json(OrgSettings { open }))
}

async fn org_settings_set(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
    Json(body): Json<OrgSettings>,
) -> Result<Json<OrgSettings>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    admin_ok(&who)?;
    app.store
        .set_org_open_access(&who.scope, body.open)
        .await
        .map_err(err)?;
    Ok(Json(OrgSettings { open: body.open }))
}

#[derive(Deserialize)]
struct GrantIn {
    client: String,
    r#box: String,
}

async fn acl_grant(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
    Json(body): Json<GrantIn>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    let b = app.store.agent_in(&who.scope, &body.r#box).await.map_err(err)?;
    if !who.may_box(&b.owner) {
        return Err(forbid("not your machine"));
    }
    app.store
        .grant_box(&who.scope, &body.client, &body.r#box)
        .await
        .map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct ClientIn {
    client: String,
}

async fn acl_open(
    State(app): State<Arc<App>>,
    cookies: Cookies,
    Path(org): Path<String>,
    Json(body): Json<ClientIn>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let who = scoped(&app, &cookies, &org).await?;
    admin_ok(&who)?;
    app.store.clear_grants(&who.scope, &body.client).await.map_err(err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[cfg(test)]
mod tests {
    use super::name_from_email;

    #[test]
    fn handle_from_email() {
        assert_eq!(name_from_email("alice@acme.test"), "alice");
        assert_eq!(name_from_email("Alice.Smith@x"), "alicesmith");
        assert_eq!(name_from_email("12@x"), "u12");
    }
}
