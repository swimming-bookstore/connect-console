mod api;
mod chrome;
mod pages;
mod templates;
mod types;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::hooks::use_location;
use leptos_router::path;

use api::{event_value, get_json, post};
use chrome::{apply_theme, theme_now, Mark, ThemeBtn};
use templates::{Breadcrumb, Page, Subpage};
use types::{space_href, LabLogin, Me, Space};

pub use types::{Collection, Record, Wall};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let me = RwSignal::new(None::<Me>);
    Effect::new(move |_| {
        apply_theme(&theme_now());
        spawn_local(async move {
            loop {
                if let Some(m) = get_json::<Me>("/api/auth/me").await {
                    if me.get_untracked() != Some(m.clone()) {
                        me.set(Some(m));
                    }
                }
                gloo_timers::future::TimeoutFuture::new(2000).await;
            }
        });
    });
    let phase = Memo::new(move |_| match me.get() {
        None => 0u8,
        Some(m) if m.email.is_none() => 1,
        Some(_) => 2,
    });
    view! {
        <style>{include_str!("../public/style.css")}</style>
        {move || match phase.get() {
            0 => view! { <p class="empty">"Loading…"</p> }.into_any(),
            1 => view! { <Gate me /> }.into_any(),
            _ => view! { <Shell me /> }.into_any(),
        }}
    }
}

#[component]
fn Gate(me: RwSignal<Option<Me>>) -> impl IntoView {
    let mode = RwSignal::new("in");
    let email = RwSignal::new(String::new());
    let pass = RwSignal::new(String::new());
    let flash = RwSignal::new(String::new());
    let lab = RwSignal::new(Vec::<LabLogin>::new());
    Effect::new(move |_| {
        spawn_local(async move {
            loop {
                let rows = get_json::<Vec<LabLogin>>("/api/auth/lab").await.unwrap_or_default();
                if lab.get_untracked() != rows {
                    lab.set(rows);
                }
                gloo_timers::future::TimeoutFuture::new(4000).await;
            }
        });
    });
    view! {
        <div class="gate-bar">
            <span class="brand-mark">
                <Mark />
                <span class="brand">"Connect"</span>
            </span>
            <ThemeBtn />
        </div>
        <main class="wrap narrow gate" data-testid="gate">
            <h1 class="gate-title">
                <Mark big=true />
                "Connect"
            </h1>
            <div class="tabs gate-tabs">
                <button type="button" data-testid="tab-signin" class:on=move || mode.get() == "in" on:click=move |_| {
                    mode.set("in");
                    flash.set(String::new());
                }>"Sign in"</button>
                <button type="button" data-testid="tab-create" class:on=move || mode.get() == "up" on:click=move |_| {
                    mode.set("up");
                    flash.set(String::new());
                }>"Create account"</button>
            </div>
            <form class="stack gate-form" on:submit=move |ev| {
                ev.prevent_default();
                let e = email.get();
                let p = pass.get();
                let up = mode.get() == "up";
                spawn_local(async move {
                    let body = if up {
                        serde_json::json!({"email": e, "password": p, "org": ""})
                    } else {
                        serde_json::json!({"email": e, "password": p})
                    };
                    let path = if up { "/api/auth/register" } else { "/api/auth/login" };
                    match post(path, &body).await {
                        Ok(_) => {
                                if let Some(m) = get_json::<Me>("/api/auth/me").await {
                                    #[cfg(target_arch = "wasm32")]
                                    if let Some(w) = web_sys::window() {
                                        let _ = w.location().set_href("/");
                                    }
                                    me.set(Some(m));
                                }
                        }
                        Err(err) => flash.set(err),
                    }
                });
            }>
                <label>
                    <span>"Email"</span>
                    <input data-testid="email" type="email" autocomplete="username" prop:value=move || email.get()
                        on:input=move |ev| email.set(event_value(&ev)) />
                </label>
                <label>
                    <span>"Password"</span>
                    <input data-testid="password" type="password" autocomplete="current-password" prop:value=move || pass.get()
                        on:input=move |ev| pass.set(event_value(&ev)) />
                </label>
                {move || {
                    let f = flash.get();
                    if f.is_empty() {
                        view! { <span></span> }.into_any()
                    } else {
                        view! { <p class="flash">{f}</p> }.into_any()
                    }
                }}
                <button class="btn-ink" type="submit" data-testid="submit">{move || if mode.get() == "up" { "Create account" } else { "Sign in" }}</button>
            </form>
            {move || {
                let rows = lab.get();
                if mode.get() != "in" || rows.is_empty() {
                    return view! { <span></span> }.into_any();
                }
                view! {
                    <table class="lab-logins" data-testid="lab-logins">
                        <tr><th>"Email"</th><th>"Password"</th><th>"Org"</th></tr>
                        {rows.into_iter().map(|row| {
                            let email_v = row.email.clone();
                            let pass_v = row.password.clone();
                            let org_v = if row.org.is_empty() { "none".into() } else { row.org.clone() };
                            let show_email = row.email.clone();
                            let show_pass = row.password.clone();
                            view! {
                                <tr class="pick" on:click=move |_| {
                                    email.set(email_v.clone());
                                    pass.set(pass_v.clone());
                                }>
                                    <td>{show_email}</td><td>{show_pass}</td><td>{org_v}</td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </table>
                }.into_any()
            }}
        </main>
    }
}

#[component]
fn Shell(me: RwSignal<Option<Me>>) -> impl IntoView {
    view! {
        <Router>
            <ShellInner me />
        </Router>
    }
}

#[component]
fn ShellInner(me: RwSignal<Option<Me>>) -> impl IntoView {
    provide_context(me);
    let loc = use_location();
    view! {
        <div class="app">
            <header class="top">
                <A href="/" attr:class="brand-mark" attr:data-testid="nav-home">
                    <Mark />
                    <span class="brand">"Connect"</span>
                </A>
                <nav class="top-nav">
                    {move || {
                        let path = loc.pathname.get();
                        let Some((personal, prefix)) = space_nav(&path) else {
                            return view! { <span></span> }.into_any();
                        };
                        let boxes = format!("{prefix}/boxes");
                        let people = format!("{prefix}/people");
                        let access = format!("{prefix}/access");
                        view! {
                            <>
                                <A href=boxes attr:class="top-link" attr:aria-current="false" attr:data-testid="nav-boxes">"Boxes"</A>
                                {if personal {
                                    view! { <span></span> }.into_any()
                                } else {
                                    view! {
                                        <>
                                            <A href=people attr:class="top-link" attr:aria-current="false" attr:data-testid="nav-people">"People"</A>
                                            <A href=access attr:class="top-link" attr:aria-current="false" attr:data-testid="nav-access">"Access"</A>
                                            <A href=format!("{prefix}/settings") attr:class="top-link" attr:aria-current="false" attr:data-testid="nav-settings">"Settings"</A>
                                        </>
                                    }.into_any()
                                }}
                            </>
                        }.into_any()
                    }}
                </nav>
                <div class="top-foot">
                    <span class="who" data-testid="who">
                        <A href="/profile" attr:class="who-link">{move || {
                            me.get().and_then(|m| m.email).unwrap_or_default()
                        }}</A>
                    </span>
                    <ThemeBtn />
                    <button class="ghost" data-testid="sign-out" on:click=move |_| {
                        spawn_local(async move {
                            let _ = post("/api/auth/logout", &serde_json::json!({})).await;
                            me.set(get_json::<Me>("/api/auth/me").await);
                        });
                    }>"Sign out"</button>
                </div>
            </header>
            <div class="stage">
                <Routes fallback=|| view! { <p class="empty">"Not found"</p> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/profile") view=pages::Profile />
                    <Route path=path!("/new-organization") view=pages::NewOrg />
                    <ParentRoute path=path!("/personal") view=|| view! { <Outlet/> }>
                        <Route path=path!("") view=pages::Org />
                        <Route path=path!("boxes/new") view=pages::NewBox />
                        <Route path=path!("boxes/:box/access") view=pages::BoxAccess />
                        <Route path=path!("boxes/:box/remove") view=pages::BoxRemove />
                        <Route path=path!("boxes/:box") view=pages::BoxPage />
                        <Route path=path!("boxes") view=pages::Boxes />
                    </ParentRoute>
                    <ParentRoute path=path!("/org/:name") view=|| view! { <Outlet/> }>
                        <Route path=path!("") view=pages::Org />
                        <Route path=path!("boxes/new") view=pages::NewBox />
                        <Route path=path!("boxes/:box/access/new") view=pages::BoxGrant />
                        <Route path=path!("boxes/:box/access") view=pages::BoxAccess />
                        <Route path=path!("boxes/:box/remove") view=pages::BoxRemove />
                        <Route path=path!("boxes/:box") view=pages::BoxPage />
                        <Route path=path!("boxes") view=pages::Boxes />
                        <Route path=path!("people/new") view=pages::NewPerson />
                        <Route path=path!("people/:person/remove") view=pages::PersonRemove />
                        <Route path=path!("people/:person") view=pages::PersonPage />
                        <Route path=path!("people") view=pages::People />
                        <Route path=path!("access/new") view=pages::GrantAccess />
                        <Route path=path!("access/:person/new") view=pages::GrantAccess />
                        <Route path=path!("access/:person/open") view=pages::AccessOpen />
                        <Route path=path!("access/:person") view=pages::AccessPerson />
                        <Route path=path!("access") view=pages::Access />
                        <Route path=path!("settings") view=pages::OrgSettings />
                    </ParentRoute>
                </Routes>
            </div>
        </div>
    }
}

fn space_nav(path: &str) -> Option<(bool, String)> {
    let mut it = path.trim_start_matches('/').split('/').filter(|s| !s.is_empty());
    match it.next()? {
        "personal" => Some((true, "/personal".into())),
        "org" => {
            let name = it.next()?;
            if name.is_empty() || name == "personal" {
                None
            } else {
                Some((false, space_href(name)))
            }
        }
        _ => None,
    }
}

#[component]
fn Home() -> impl IntoView {
    let me = expect_context::<RwSignal<Option<Me>>>();
    let spaces = RwSignal::new(Vec::<Space>::new());
    Effect::new(move |_| {
        spawn_local(async move {
            loop {
                spaces.set(get_json::<Vec<Space>>("/api/orgs").await.unwrap_or_default());
                gloo_timers::future::TimeoutFuture::new(2000).await;
            }
        });
    });
    view! {
        <main class="wrap" data-testid="page-home">
            {move || {
                let list = spaces.get();
                let company = me.get().as_ref().and_then(|m| m.company());
                let mut pages: Vec<Subpage> = list
                    .into_iter()
                    .map(|s| {
                        let test = if s.name == "personal" { "home-personal" } else { "home-org" };
                        Subpage::new(s.label(), s.href(), s.machines.unwrap_or(0)).with_test(test)
                    })
                    .collect();
                if company.is_none() {
                    pages.push(
                        Subpage::new("Create organization", "/new-organization", "—")
                            .with_test("home-create-org"),
                    );
                }
                let crumbs = vec![Breadcrumb::current("Home")];
                view! {
                    <Page breadcrumbs=crumbs subpages=pages>
                        <span></span>
                    </Page>
                }
            }}
        </main>
    }
}
