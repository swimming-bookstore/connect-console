use leptos::prelude::*;

/// Punched square.
#[component]
pub fn Mark(#[prop(optional)] big: bool) -> impl IntoView {
    let class = if big { "mark big" } else { "mark" };
    view! {
        <svg class=class viewBox="0 0 16 16" aria-hidden="true" focusable="false">
            <path
                fill="currentColor"
                fill-rule="evenodd"
                d="M2 0h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2zm3.5 5.5h5v5h-5z"
            ></path>
        </svg>
    }
}

pub fn theme_now() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Ok(Some(s)) = w.local_storage() {
                if let Ok(Some(t)) = s.get_item("connect-theme") {
                    if t == "dark" || t == "light" {
                        return t;
                    }
                }
            }
        }
    }
    "light".into()
}

pub fn apply_theme(theme: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                if let Some(el) = doc.document_element() {
                    let _ = el.set_attribute("data-theme", theme);
                }
            }
            if let Ok(Some(s)) = w.local_storage() {
                let _ = s.set_item("connect-theme", theme);
            }
        }
    }
    let _ = theme;
}

#[component]
pub fn ThemeBtn() -> impl IntoView {
    let theme = RwSignal::new(theme_now());
    Effect::new(move |_| apply_theme(&theme.get()));
    view! {
        <button class="ghost" type="button" data-testid="theme" on:click=move |_| {
            theme.update(|t| *t = if t.as_str() == "dark" { "light" } else { "dark" }.into());
        }>{move || if theme.get() == "dark" { "Light" } else { "Dark" }}</button>
    }
}
