//! WASM port of the SSR page chrome: breadcrumbs, actions, info, content, subpages.
//!
//! Set `Noun { many, one }` once (Boxes / box). List, add, item, and remove
//! pages reuse that for titles, actions, and hrefs.

use crate::types::space_href;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}

impl Breadcrumb {
    pub fn link(label: impl ToString, href: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: Some(href.to_string()),
        }
    }

    pub fn current(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: None,
        }
    }
}

#[derive(Clone)]
pub struct ActionLink {
    pub label: String,
    pub href: String,
    pub testid: String,
}

impl ActionLink {
    pub fn new(label: impl ToString, href: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: href.to_string(),
            testid: String::new(),
        }
    }

    pub fn with_test(mut self, id: impl ToString) -> Self {
        self.testid = id.to_string();
        self
    }
}

/// Collection noun. `many` is the list title and path (`Boxes` → `/boxes`).
/// `one` is the record word (`box` → Add box, Remove box).
#[derive(Clone, Copy)]
pub struct Noun {
    pub many: &'static str,
    pub one: &'static str,
}

impl Noun {
    pub const BOXES: Noun = Noun {
        many: "Boxes",
        one: "box",
    };
    pub const PEOPLE: Noun = Noun {
        many: "People",
        one: "person",
    };

    pub fn path(&self) -> String {
        self.many.to_lowercase()
    }

    pub fn add(&self) -> String {
        format!("Add {}", self.one)
    }

    pub fn remove(&self) -> String {
        format!("Remove {}", self.one)
    }

    pub fn list_href(&self, org: &str) -> String {
        format!("{}/{}", space_href(org), self.path())
    }

    pub fn new_href(&self, org: &str) -> String {
        format!("{}/{}/new", space_href(org), self.path())
    }

    pub fn item_href(&self, org: &str, name: &str) -> String {
        format!("{}/{}/{name}", space_href(org), self.path())
    }

    pub fn remove_href(&self, org: &str, name: &str) -> String {
        format!("{}/{}/{name}/remove", space_href(org), self.path())
    }

    pub fn add_test(&self) -> String {
        format!("add-{}", self.one)
    }

    pub fn remove_test(&self) -> String {
        format!("{}-remove", self.one)
    }

    pub fn add_action(&self, org: &str) -> ActionLink {
        ActionLink::new(self.add(), self.new_href(org)).with_test(self.add_test())
    }

    pub fn remove_action(&self, org: &str, name: &str) -> ActionLink {
        ActionLink::new(self.remove(), self.remove_href(org, name)).with_test(self.remove_test())
    }

    pub fn crumbs_list(&self, org: &str) -> Vec<Breadcrumb> {
        vec![
            Breadcrumb::link("Home", "/"),
            Breadcrumb::link(space_label(org), space_href(org)),
            Breadcrumb::current(self.many),
        ]
    }

    pub fn crumbs_add(&self, org: &str) -> Vec<Breadcrumb> {
        vec![
            Breadcrumb::link("Home", "/"),
            Breadcrumb::link(space_label(org), space_href(org)),
            Breadcrumb::link(self.many, self.list_href(org)),
            Breadcrumb::current(self.add()),
        ]
    }

    pub fn crumbs_item(&self, org: &str, name: impl ToString) -> Vec<Breadcrumb> {
        vec![
            Breadcrumb::link("Home", "/"),
            Breadcrumb::link(space_label(org), space_href(org)),
            Breadcrumb::link(self.many, self.list_href(org)),
            Breadcrumb::current(name.to_string()),
        ]
    }

    pub fn crumbs_remove(&self, org: &str, name: impl ToString) -> Vec<Breadcrumb> {
        let name = name.to_string();
        vec![
            Breadcrumb::link("Home", "/"),
            Breadcrumb::link(space_label(org), space_href(org)),
            Breadcrumb::link(self.many, self.list_href(org)),
            Breadcrumb::link(name.clone(), self.item_href(org, &name)),
            Breadcrumb::current(self.remove()),
        ]
    }
}

pub fn space_label(org: &str) -> String {
    if org == "personal" {
        "Personal".into()
    } else {
        org.into()
    }
}

#[derive(Clone)]
pub struct Subpage {
    pub label: String,
    pub href: String,
    pub count: String,
    pub testid: String,
}

impl Subpage {
    pub fn new(label: impl ToString, href: impl ToString, count: impl std::fmt::Display) -> Self {
        Self {
            label: label.to_string(),
            href: href.to_string(),
            count: count.to_string(),
            testid: String::new(),
        }
    }

    pub fn with_test(mut self, id: impl ToString) -> Self {
        self.testid = id.to_string();
        self
    }
}

#[component]
pub fn Page(
    breadcrumbs: Vec<Breadcrumb>,
    #[prop(optional)] action_links: Vec<ActionLink>,
    #[prop(optional)] subpages: Vec<Subpage>,
    children: Children,
) -> impl IntoView {
    let current = breadcrumbs
        .iter()
        .rev()
        .find(|c| c.href.is_none())
        .map(|c| c.label.clone())
        .unwrap_or_default();
    view! {
        <div class="page">
            {if breadcrumbs.len() > 1 {
                let trail: Vec<_> = breadcrumbs.iter().filter(|c| c.href.is_some()).cloned().collect();
                Either::Left(view! {
                    <>
                        <nav class="crumbs" aria-label="Breadcrumb">
                            {trail.into_iter().enumerate().map(|(i, crumb)| {
                                let href = crumb.href.unwrap_or_default();
                                let testid = if i == 0 {
                                    "crumb-home"
                                } else if i == 1 {
                                    "crumb-space"
                                } else {
                                    "crumb"
                                };
                                view! {
                                    {if i > 0 {
                                        Either::Left(view! { <span class="crumb-sep" aria-hidden="true">"/"</span> })
                                    } else {
                                        Either::Right(())
                                    }}
                                    <A href={href} attr:data-testid={testid}>{crumb.label}</A>
                                }
                            }).collect::<Vec<_>>()}
                        </nav>
                        <h1 data-testid="org-title">{current.clone()}</h1>
                    </>
                })
            } else {
                Either::Right(view! { <h1 data-testid="org-title">{current}</h1> })
            }}

            {if !action_links.is_empty() {
                Either::Left(view! {
                    <div class="actions">
                        {action_links.into_iter().map(|link| {
                            view! {
                                <a class="action-link" href={link.href} data-testid={link.testid}>{link.label}</a>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                })
            } else {
                Either::Right(())
            }}

            {children()}

            {if !subpages.is_empty() {
                Either::Left(view! {
                    <div class="dir">
                        {subpages.into_iter().map(|sp| {
                            let test = sp.testid.clone();
                            view! {
                                <a class="dir-row" href={sp.href} data-testid={test}>
                                    <strong>{sp.label}</strong>
                                    <span class="count">{sp.count}</span>
                                </a>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                })
            } else {
                Either::Right(())
            }}
        </div>
    }
}
