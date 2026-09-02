use crate::api::{event_value, get_json, post};
use crate::templates::{space_label, ActionLink, Breadcrumb, Noun, Page, Subpage};
use crate::types::{space_href, Collection, Me, Wall};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde_json::json;

fn go(href: impl AsRef<str>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_href(href.as_ref());
    }
}

fn names(w: &Wall, id: &str) -> Vec<String> {
    w.collections
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.records.iter().filter_map(|r| r.cells.first().cloned()).collect())
        .unwrap_or_default()
}

fn pick(signal: &RwSignal<String>, fallback: &str) -> String {
    let v = signal.get();
    if v.is_empty() {
        fallback.into()
    } else {
        v
    }
}

fn org_slug() -> String {
    let n = use_params_map().read().get("name").unwrap_or_default();
    if n.is_empty() {
        "personal".into()
    } else {
        n
    }
}

fn crumbs_here(org: &str, current: &str) -> Vec<Breadcrumb> {
    vec![
        Breadcrumb::link("Home", "/"),
        Breadcrumb::link(space_label(org), space_href(org)),
        Breadcrumb::current(current),
    ]
}

fn crumbs_into(org: &str, section: &str, path: &str, current: impl ToString) -> Vec<Breadcrumb> {
    vec![
        Breadcrumb::link("Home", "/"),
        Breadcrumb::link(space_label(org), space_href(org)),
        Breadcrumb::link(section, format!("{}/{path}", space_href(org))),
        Breadcrumb::current(current.to_string()),
    ]
}

fn crumbs_box_access(org: &str, box_name: &str, current: &str) -> Vec<Breadcrumb> {
    let kind = Noun::BOXES;
    let mut crumbs = kind.crumbs_item(org, box_name);
    if let Some(item) = crumbs.pop() {
        crumbs.push(Breadcrumb::link(item.label, kind.item_href(org, box_name)));
    }
    if current == "Access" {
        crumbs.push(Breadcrumb::current("Access"));
    } else {
        crumbs.push(Breadcrumb::link("Access", format!("{}/access", kind.item_href(org, box_name))));
        crumbs.push(Breadcrumb::current(current));
    }
    crumbs
}

fn use_wall() -> (RwSignal<String>, RwSignal<Wall>) {
    let org = RwSignal::new(org_slug());
    Effect::new(move |_| org.set(org_slug()));
    let wall = RwSignal::new(Wall::default());
    watch_wall(org, wall);
    (org, wall)
}

fn watch_wall(org: RwSignal<String>, wall: RwSignal<Wall>) {
    Effect::new(move |_| {
        let n = org.get();
        if n.is_empty() {
            return;
        }
        spawn_local(async move {
            loop {
                if let Some(s) = get_json::<Wall>(&format!("/api/orgs/{n}/wall")).await {
                    wall.set(s);
                }
                TimeoutFuture::new(2000).await;
            }
        });
    });
}

fn use_item(key: &'static str) -> (RwSignal<String>, RwSignal<String>) {
    let params = use_params_map();
    let org = RwSignal::new(String::new());
    let item = RwSignal::new(String::new());
    Effect::new(move |_| {
        let p = params.read();
        org.set(org_slug());
        item.set(p.get(key).unwrap_or_default());
    });
    (org, item)
}

#[component]
pub fn Org() -> impl IntoView {
    let (org, wall) = use_wall();
    view! {
        <main class="wrap page" data-testid="page-org">
            {move || {
                let org = org.get();
                let w = wall.get();
                let label = space_label(&org);
                let crumbs = vec![
                    Breadcrumb::link("Home", "/"),
                    Breadcrumb::current(label.clone()),
                ];
                let boxes = Noun::BOXES;
                let people = Noun::PEOPLE;
                let company = org != "personal";
                let n_boxes = w.collections.iter().find(|c| c.id == boxes.path()).map(|c| c.records.len()).unwrap_or(0);
                let n_people = w.collections.iter().find(|c| c.id == people.path()).map(|c| c.records.len()).unwrap_or(0);
                let n_access = w.collections.iter().find(|c| c.id == "access").map(|c| c.records.len()).unwrap_or(0);
                let mut pages = vec![
                    Subpage::new(boxes.many, boxes.list_href(&org), n_boxes).with_test("org-boxes"),
                ];
                if company {
                    pages.push(Subpage::new(people.many, people.list_href(&org), n_people).with_test("org-people"));
                    pages.push(Subpage::new("Access", format!("{}/access", space_href(&org)), n_access).with_test("org-access"));
                    pages.push(Subpage::new("Settings", format!("{}/settings", space_href(&org)), "—").with_test("org-settings"));
                }
                view! {
                    <Page breadcrumbs=crumbs subpages=pages>
                        <span></span>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn NewOrg() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let flash = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-new-org">
            <Page breadcrumbs=vec![
                Breadcrumb::link("Home", "/"),
                Breadcrumb::current("Create organization"),
            ]>
                <p class="lede">"Optional. Share boxes with people. Until then, your boxes stay personal."</p>
                <p class="flash">{move || flash.get()}</p>
                <form class="stack" on:submit=move |ev| {
                    ev.prevent_default();
                    let n = name.get();
                    spawn_local(async move {
                        match post("/api/orgs", &json!({"name": n.clone()})).await {
                            Ok(_) => go(format!("{}/boxes", space_href(&n))),
                            Err(e) => flash.set(e),
                        }
                    });
                }>
                    <label>
                        <span>"Name"</span>
                        <input data-testid="org-name" placeholder="acme" prop:value=move || name.get()
                            on:input=move |ev| name.set(event_value(&ev)) />
                    </label>
                    <p class="hint">"Lowercase, like acme. You become owner."</p>
                    <button class="btn" type="submit" data-testid="org-submit">"Create organization"</button>
                </form>
            </Page>
        </main>
    }
}

#[component]
pub fn Boxes() -> impl IntoView {
    let (_org, wall) = use_wall();
    view! {
        <main class="wrap" data-testid="page-boxes">
            {move || {
                let w = wall.get();
                let org = w.org.clone();
                let kind = Noun::BOXES;
                let crumbs = kind.crumbs_list(&org);
                let nav = vec![kind.add_action(&org)];
                let col = w.collections.into_iter().find(|c| c.id == kind.path());
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        {match col {
                            Some(c) => view! { <Board col=c org=org.clone() kind=kind /> }.into_any(),
                            None => view! { <span></span> }.into_any(),
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn People() -> impl IntoView {
    let (_org, wall) = use_wall();
    view! {
        <main class="wrap" data-testid="page-people">
            {move || {
                let w = wall.get();
                let org = w.org.clone();
                let kind = Noun::PEOPLE;
                let crumbs = kind.crumbs_list(&org);
                let nav = vec![kind.add_action(&org)];
                let col = w.collections.into_iter().find(|c| c.id == kind.path());
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        {match col {
                            Some(c) => view! { <Board col=c org=org.clone() kind=kind /> }.into_any(),
                            None => view! { <span></span> }.into_any(),
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
fn Board(col: Collection, org: String, kind: Noun) -> impl IntoView {
    let records = col.records.clone();
    view! {
        <section>
            {if records.is_empty() {
                view! { <span></span> }.into_any()
            } else {
                view! {
                    <div class="dir">
                        {records.into_iter().map(|r| {
                            let name = r.cells.first().cloned().unwrap_or_default();
                            let rest = r.cells.iter().skip(1).cloned().collect::<Vec<_>>();
                            let meta = rest.join(" · ");
                            let test_id = format!("card-{name}");
                            let href = kind.item_href(&org, &name);
                            view! {
                                <a class="dir-row" href=href data-testid=test_id>
                                    <strong>{name}</strong>
                                    <span class="count">{meta}</span>
                                </a>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}

#[component]
pub fn NewBox() -> impl IntoView {
    let org = RwSignal::new(String::new());
    Effect::new(move |_| {
        org.set(org_slug());
    });
    let name = RwSignal::new(String::new());
    let flash = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-add-box">
            {move || {
                let org_n = org.get();
                let kind = Noun::BOXES;
                let crumbs = kind.crumbs_add(&org_n);
                let done = !secret.get().is_empty();
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="flash">{flash.get()}</p>
                        <p class="secret" data-testid="secret">{secret.get()}</p>
                        {if done {
                            view! {
                                <p><a class="btn" data-testid="done" href=kind.list_href(&org_n)>"Done"</a></p>
                            }.into_any()
                        } else {
                            view! {
                                <form class="stack" on:submit=move |ev| {
                                    ev.prevent_default();
                                    let n = org.get();
                                    let who = name.get();
                                    spawn_local(async move {
                                        match post(
                                            &format!("/api/orgs/{n}/agents"),
                                            &json!({"name": who, "client": false}),
                                        ).await {
                                            Ok(v) => {
                                                if let Some(t) = v.get("token").and_then(|x| x.as_str()) {
                                                    secret.set(format!("Token — copy now, shown once\n{t}"));
                                                } else {
                                                    flash.set("no token".into());
                                                }
                                            }
                                            Err(e) => flash.set(e),
                                        }
                                    });
                                }>
                                    <label>
                                        <span>"Name"</span>
                                        <input data-testid="box-name" placeholder="box-1" prop:value=move || name.get()
                                            on:input=move |ev| name.set(event_value(&ev)) />
                                    </label>
                                    <button class="btn" type="submit" data-testid="add-submit">{kind.add()}</button>
                                </form>
                            }.into_any()
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn BoxPage() -> impl IntoView {
    let (org, box_name) = use_item("box");
    let flash = RwSignal::new(String::new());
    let access = RwSignal::new((true, Vec::<String>::new()));
    Effect::new(move |_| {
        let n = org.get();
        let b = box_name.get();
        if n.is_empty() || b.is_empty() {
            return;
        }
        spawn_local(async move {
            if let Some(v) = get_json::<serde_json::Value>(&format!("/api/orgs/{n}/agents/{b}/access")).await {
                let org_wide = v.get("org").and_then(|x| x.as_bool()).unwrap_or(true);
                let people = v
                    .get("people")
                    .and_then(|x| x.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                access.set((org_wide, people));
            }
        });
    });
    view! {
        <main class="wrap" data-testid="page-box">
            {move || {
                let org_n = org.get();
                let nm = box_name.get();
                let kind = Noun::BOXES;
                let crumbs = kind.crumbs_item(&org_n, nm.clone());
                let (org_wide, people) = access.get();
                let who_count: String = if org_wide {
                    "all".into()
                } else {
                    people.len().to_string()
                };
                let pages = vec![
                    Subpage::new("Access", format!("{}/access", kind.item_href(&org_n, &nm)), who_count).with_test("box-access"),
                ];
                let nav = vec![kind.remove_action(&org_n, &nm)];
                view! {
                    <Page breadcrumbs=crumbs action_links=nav subpages=pages>
                        <p class="flash">{flash.get()}</p>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn BoxAccess() -> impl IntoView {
    let (org, box_name) = use_item("box");
    let access = RwSignal::new((true, Vec::<String>::new()));
    Effect::new(move |_| {
        let n = org.get();
        let b = box_name.get();
        if n.is_empty() || b.is_empty() {
            return;
        }
        spawn_local(async move {
            loop {
                if let Some(v) = get_json::<serde_json::Value>(&format!("/api/orgs/{n}/agents/{b}/access")).await {
                    let org_wide = v.get("org").and_then(|x| x.as_bool()).unwrap_or(true);
                    let people = v
                        .get("people")
                        .and_then(|x| x.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    access.set((org_wide, people));
                }
                TimeoutFuture::new(2000).await;
            }
        });
    });
    view! {
        <main class="wrap" data-testid="page-box-access">
            {move || {
                let org_n = org.get();
                let nm = box_name.get();
                let crumbs = crumbs_box_access(&org_n, &nm, "Access");
                let (org_wide, people) = access.get();
                let nav = if org_n == "personal" {
                    Vec::new()
                } else {
                    vec![ActionLink::new("Grant access", format!("{}/access/new", Noun::BOXES.item_href(&org_n, &nm))).with_test("box-grant")]
                };
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        {if org_wide {
                            view! { <p class="lede" data-testid="box-access-list">"Everyone in the organization."</p> }.into_any()
                        } else if people.is_empty() {
                            view! { <p class="lede" data-testid="box-access-list">"No one yet."</p> }.into_any()
                        } else {
                            view! {
                                <div class="dir" data-testid="box-access-list">
                                    {people.into_iter().map(|p| {
                                        view! {
                                            <div class="dir-row">
                                                <strong>{p}</strong>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn BoxGrant() -> impl IntoView {
    let (org, box_name) = use_item("box");
    let wall = RwSignal::new(Wall::default());
    watch_wall(org, wall);
    let flash = RwSignal::new(String::new());
    let person = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-box-grant">
            {move || {
                let org_n = org.get();
                let nm = box_name.get();
                let people: Vec<String> = wall
                    .get()
                    .collections
                    .iter()
                    .find(|c| c.id == "people")
                    .map(|c| c.records.iter().filter_map(|r| r.cells.first().cloned()).collect())
                    .unwrap_or_default();
                let p0 = people.first().cloned().unwrap_or_default();
                let crumbs = crumbs_box_access(&org_n, &nm, "Grant access");
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="lede">"Allow a person to App to this box."</p>
                        <p class="flash">{flash.get()}</p>
                        <form class="stack" on:submit=move |ev| {
                            ev.prevent_default();
                            let n = org.get();
                            let b = box_name.get();
                            let who = {
                                let p = person.get();
                                if p.is_empty() { p0.clone() } else { p }
                            };
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/acl/grant"), &json!({"client": who, "box": b})).await {
                                    Ok(_) => go(format!("{}/access", Noun::BOXES.item_href(&n, &b))),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>
                            <label>
                                <span>"Person"</span>
                                <select data-testid="grant-person" on:change=move |ev| person.set(event_value(&ev))>
                                    {people.clone().into_iter().map(|p| {
                                        let label = p.clone();
                                        view! { <option value=p>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </label>
                            <button class="btn" type="submit" data-testid="grant-submit">"Allow"</button>
                        </form>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn BoxRemove() -> impl IntoView {
    let (org, box_name) = use_item("box");
    let flash = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-box-remove">
            {move || {
                let kind = Noun::BOXES;
                let org_n = org.get();
                let nm = box_name.get();
                let crumbs = kind.crumbs_remove(&org_n, nm.clone());
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="lede">"Revokes the box token. It cannot dial until you add it again."</p>
                        <p class="flash">{flash.get()}</p>
                        <button class="btn-ink" type="button" data-testid="box-remove-confirm" on:click=move |_| {
                            let n = org.get();
                            let b = box_name.get();
                            let list = kind.list_href(&n);
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/agents/{b}/revoke"), &json!({})).await {
                                    Ok(_) => go(list),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>{kind.remove()}</button>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn NewPerson() -> impl IntoView {
    let org = RwSignal::new(String::new());
    Effect::new(move |_| {
        org.set(org_slug());
    });
    let name = RwSignal::new(String::new());
    let flash = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-add-person">
            {move || {
                let org_n = org.get();
                let kind = Noun::PEOPLE;
                let crumbs = kind.crumbs_add(&org_n);
                let done = !secret.get().is_empty();
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="flash">{flash.get()}</p>
                        <p class="secret" data-testid="secret">{secret.get()}</p>
                        {if done {
                            view! {
                                <p><a class="btn" data-testid="done" href=kind.list_href(&org_n)>"Done"</a></p>
                            }.into_any()
                        } else {
                            view! {
                                <form class="stack" on:submit=move |ev| {
                                    ev.prevent_default();
                                    let n = org.get();
                                    let who = name.get();
                                    spawn_local(async move {
                                        match post(
                                            &format!("/api/orgs/{n}/agents"),
                                            &json!({"email": who, "client": true}),
                                        ).await {
                                            Ok(v) => {
                                                if let Some(t) = v.get("token").and_then(|x| x.as_str()) {
                                                    secret.set(format!("Token — copy now, shown once\n{t}"));
                                                } else {
                                                    flash.set(String::new());
                                                    go(Noun::PEOPLE.list_href(&n));
                                                }
                                            }
                                            Err(e) => flash.set(e),
                                        }
                                    });
                                }>
                                    <label>
                                        <span>"Email"</span>
                                        <input data-testid="person-email" type="email" placeholder="alice@acme.test" prop:value=move || name.get()
                                            on:input=move |ev| name.set(event_value(&ev)) />
                                    </label>
                                    <button class="btn" type="submit" data-testid="add-submit">{kind.add()}</button>
                                </form>
                            }.into_any()
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn PersonPage() -> impl IntoView {
    let (org, person) = use_item("person");
    let wall = RwSignal::new(Wall::default());
    watch_wall(org, wall);
    let flash = RwSignal::new(String::new());
    let role = RwSignal::new(String::from("member"));
    Effect::new(move |_| {
        let nm = person.get();
        if let Some(r) = wall
            .get()
            .collections
            .iter()
            .find(|c| c.id == "people")
            .and_then(|c| c.records.iter().find(|r| r.cells.first().map(|x| x.as_str()) == Some(nm.as_str())))
        {
            role.set(r.cells.get(1).cloned().unwrap_or_else(|| "member".into()));
        }
    });
    view! {
        <main class="wrap" data-testid="page-person">
            {move || {
                let org_n = org.get();
                let nm = person.get();
                let kind = Noun::PEOPLE;
                let crumbs = kind.crumbs_item(&org_n, nm.clone());
                let nav = vec![kind.remove_action(&org_n, &nm)];
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        <p class="flash">{flash.get()}</p>
                        <form class="stack" on:submit=move |ev| {
                            ev.prevent_default();
                            let n = org.get();
                            let who = person.get();
                            let r = role.get();
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/agents/{who}/role"), &json!({"role": r})).await {
                                    Ok(_) => flash.set(String::new()),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>
                            <label>
                                <span>"Role"</span>
                                <select data-testid="person-role" on:change=move |ev| role.set(event_value(&ev))>
                                    <option value="member">"member"</option>
                                    <option value="admin">"admin"</option>
                                    <option value="owner">"owner"</option>
                                </select>
                            </label>
                            <button class="btn" type="submit">"Save role"</button>
                        </form>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn PersonRemove() -> impl IntoView {
    let (org, person) = use_item("person");
    let flash = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-person-remove">
            {move || {
                let kind = Noun::PEOPLE;
                let org_n = org.get();
                let nm = person.get();
                let crumbs = kind.crumbs_remove(&org_n, nm.clone());
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="lede">"They leave the organization and cannot open the console."</p>
                        <p class="flash">{flash.get()}</p>
                        <button class="btn-ink" type="button" data-testid="person-remove-confirm" on:click=move |_| {
                            let n = org.get();
                            let who = person.get();
                            let list = kind.list_href(&n);
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/agents/{who}/revoke"), &json!({})).await {
                                    Ok(_) => go(list),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>{kind.remove()}</button>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn AccessOpen() -> impl IntoView {
    let (org, person) = use_item("person");
    let flash = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-access-open">
            {move || {
                let org_n = org.get();
                let nm = person.get();
                let crumbs = vec![
                    Breadcrumb::link("Home", "/"),
                    Breadcrumb::link(space_label(&org_n), space_href(&org_n)),
                    Breadcrumb::link("Access", format!("{}/access", space_href(&org_n))),
                    Breadcrumb::link(nm.clone(), format!("{}/access/{nm}", space_href(&org_n))),
                    Breadcrumb::current("Allow all boxes"),
                ];
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="lede">"They may App to every box in the organization."</p>
                        <p class="flash">{flash.get()}</p>
                        <button class="btn-ink" type="button" data-testid="access-open-confirm" on:click=move |_| {
                            let n = org.get();
                            let who = person.get();
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/acl/open"), &json!({"client": who})).await {
                                    Ok(_) => go(format!("{}/access/{who}", space_href(&n))),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>"Allow all boxes"</button>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn Access() -> impl IntoView {
    let (_org, wall) = use_wall();
    view! {
        <main class="wrap" data-testid="page-access">
            {move || {
                let w = wall.get();
                let org = w.org.clone();
                let crumbs = crumbs_here(&org, "Access");
                let nav = vec![ActionLink::new("Grant access", format!("{}/access/new", space_href(&org))).with_test("grant-access")];
                let people = names(&w, "access");
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        <div class="dir" data-testid="access-table">
                            {people.into_iter().map(|p| {
                                let href = format!("{}/access/{p}", space_href(&org));
                                view! {
                                    <a class="dir-row" href=href>
                                        <strong>{p}</strong>
                                    </a>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn GrantAccess() -> impl IntoView {
    let params = use_params_map();
    let (_org, wall) = use_wall();
    let flash = RwSignal::new(String::new());
    let person = RwSignal::new(String::new());
    let box_name = RwSignal::new(String::new());
    Effect::new(move |_| {
        let from = params.read().get("person").unwrap_or_default();
        if !from.is_empty() && person.get_untracked().is_empty() {
            person.set(from);
        }
    });
    view! {
        <main class="wrap" data-testid="page-grant">
            {move || {
                let w = wall.get();
                let org = w.org.clone();
                let from = params.read().get("person").unwrap_or_default();
                let people = names(&w, "people");
                let boxes = names(&w, "boxes");
                let p0 = if from.is_empty() {
                    people.first().cloned().unwrap_or_default()
                } else {
                    from.clone()
                };
                let b0 = boxes.first().cloned().unwrap_or_default();
                let crumbs = if from.is_empty() {
                    crumbs_into(&org, "Access", "access", "Grant access")
                } else {
                    let mut c = crumbs_into(&org, "Access", "access", from.clone());
                    c.pop();
                    c.push(Breadcrumb::link(from.clone(), format!("{}/access/{from}", space_href(&org))));
                    c.push(Breadcrumb::current("Grant access"));
                    c
                };
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="flash">{flash.get()}</p>
                        <form class="stack" on:submit=move |ev| {
                            ev.prevent_default();
                            let n = org.clone();
                            let back = from.clone();
                            let who = pick(&person, &p0);
                            let b = pick(&box_name, &b0);
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/acl/grant"), &json!({"client": who.clone(), "box": b})).await {
                                    Ok(_) => go(if back.is_empty() {
                                        format!("{}/access", space_href(&n))
                                    } else {
                                        format!("{}/access/{who}", space_href(&n))
                                    }),
                                    Err(e) => flash.set(e),
                                }
                            });
                        }>
                            <label>
                                <span>"Person"</span>
                                <select data-testid="grant-person" prop:value=pick(&person, &p0) on:change=move |ev| person.set(event_value(&ev))>
                                    {people.into_iter().map(|p| {
                                        let label = p.clone();
                                        view! { <option value=p>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </label>
                            <label>
                                <span>"Box"</span>
                                <select data-testid="grant-box" on:change=move |ev| box_name.set(event_value(&ev))>
                                    {boxes.into_iter().map(|b| {
                                        let label = b.clone();
                                        view! { <option value=b>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </label>
                            <button class="btn" type="submit" data-testid="grant-submit">"Allow"</button>
                        </form>
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn AccessPerson() -> impl IntoView {
    let (org, person) = use_item("person");
    let wall = RwSignal::new(Wall::default());
    watch_wall(org, wall);
    let flash = RwSignal::new(String::new());
    view! {
        <main class="wrap" data-testid="page-access-person">
            {move || {
                let org_n = org.get();
                let nm = person.get();
                let boxes = wall
                    .get()
                    .collections
                    .iter()
                    .find(|c| c.id == "access")
                    .and_then(|c| {
                        c.records.iter().find(|r| r.cells.first().map(|x| x.as_str()) == Some(nm.as_str()))
                    })
                    .and_then(|r| r.cells.get(1).cloned())
                    .unwrap_or_else(|| "none".into());
                let crumbs = crumbs_into(&org_n, "Access", "access", nm.clone());
                let nav = vec![
                    ActionLink::new("Grant access", format!("{}/access/{nm}/new", space_href(&org_n))).with_test("grant-access"),
                    ActionLink::new("Allow all boxes", format!("{}/access/{nm}/open", space_href(&org_n))).with_test("access-open"),
                ];
                view! {
                    <Page breadcrumbs=crumbs action_links=nav>
                        <p class="flash">{flash.get()}</p>
                        {if boxes == "none" || boxes == "all boxes" || boxes.is_empty() {
                            view! { <p class="lede">{boxes.clone()}</p> }.into_any()
                        } else {
                            view! {
                                <div class="dir">
                                    {boxes.split(", ").filter(|s| !s.is_empty()).map(|b| {
                                        let b = b.to_string();
                                        view! {
                                            <div class="dir-row">
                                                <strong>{b}</strong>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }}
                    </Page>
                }
            }}
        </main>
    }
}

#[component]
pub fn Profile() -> impl IntoView {
    let me = expect_context::<RwSignal<Option<Me>>>();
    let display = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let flash = RwSignal::new(String::new());
    let saved = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(m) = me.get() {
            let d = m
                .display
                .clone()
                .filter(|s| !s.is_empty())
                .or(m.name.clone())
                .unwrap_or_default();
            if display.get_untracked().is_empty() {
                display.set(d);
            }
        }
    });
    view! {
        <main class="wrap" data-testid="page-profile">
            <Page breadcrumbs=vec![
                Breadcrumb::link("Home", "/"),
                Breadcrumb::current("Profile"),
            ]>
                <p class="lede">{move || me.get().and_then(|m| m.email).unwrap_or_default()}</p>
                <p class="flash">{move || flash.get()}</p>
                <form class="stack" on:submit=move |ev| {
                    ev.prevent_default();
                    let d = display.get();
                    let p = password.get();
                    spawn_local(async move {
                        match post("/api/auth/profile", &json!({"display": d, "password": p})).await {
                            Ok(_) => {
                                flash.set(String::new());
                                saved.set(true);
                                password.set(String::new());
                                if let Some(m) = get_json::<Me>("/api/auth/me").await {
                                    me.set(Some(m));
                                }
                            }
                            Err(e) => {
                                saved.set(false);
                                flash.set(e);
                            }
                        }
                    });
                }>
                    <label>
                        <span>"Name"</span>
                        <input data-testid="profile-name" prop:value=move || display.get()
                            on:input=move |ev| display.set(event_value(&ev)) />
                    </label>
                    <label>
                        <span>"New password"</span>
                        <input data-testid="profile-password" type="password" autocomplete="new-password"
                            placeholder="leave blank to keep"
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_value(&ev)) />
                    </label>
                    <button class="btn-ink" type="submit" data-testid="profile-save">"Save"</button>
                </form>
                {move || if saved.get() {
                    view! { <p class="hint" data-testid="profile-saved">"Saved."</p> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </Page>
        </main>
    }
}

#[component]
pub fn OrgSettings() -> impl IntoView {
    let org = RwSignal::new(org_slug());
    Effect::new(move |_| {
        org.set(org_slug());
    });
    let open = RwSignal::new(true);
    let loaded = RwSignal::new(false);
    let flash = RwSignal::new(String::new());
    let saved = RwSignal::new(false);
    Effect::new(move |_| {
        let n = org.get();
        if n.is_empty() || loaded.get_untracked() {
            return;
        }
        spawn_local(async move {
            if let Some(v) = get_json::<serde_json::Value>(&format!("/api/orgs/{n}/settings")).await {
                if let Some(o) = v.get("open").and_then(|x| x.as_bool()) {
                    open.set(o);
                }
                loaded.set(true);
            }
        });
    });
    view! {
        <main class="wrap" data-testid="page-settings">
            {move || {
                let org_n = org.get();
                let crumbs = crumbs_here(&org_n, "Settings");
                view! {
                    <Page breadcrumbs=crumbs>
                        <p class="flash">{move || flash.get()}</p>
                        <form class="stack" on:submit=move |ev| {
                            ev.prevent_default();
                            let n = org.get();
                            let o = open.get();
                            spawn_local(async move {
                                match post(&format!("/api/orgs/{n}/settings"), &json!({"open": o})).await {
                                    Ok(_) => {
                                        flash.set(String::new());
                                        saved.set(true);
                                    }
                                    Err(e) => {
                                        saved.set(false);
                                        flash.set(e);
                                    }
                                }
                            });
                        }>
                            <label>
                                <span>"Default access"</span>
                                <select data-testid="org-open" prop:value=move || if open.get() { "open" } else { "lock" }
                                    on:change=move |ev| {
                                        saved.set(false);
                                        open.set(event_value(&ev) == "open");
                                    }>
                                    <option value="open" selected=open.get()>"Full access"</option>
                                    <option value="lock" selected=!open.get()>"No access"</option>
                                </select>
                            </label>
                            <button class="btn-ink" type="submit" data-testid="settings-save">"Save"</button>
                        </form>
                        {move || if saved.get() {
                            view! { <p class="hint" data-testid="settings-saved">"Saved."</p> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </Page>
                }
            }}
        </main>
    }
}
