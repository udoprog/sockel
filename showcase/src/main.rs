//! An interactive gallery of every plinth component.

mod controls;
mod pages;

use yew::prelude::*;

use plinth::{Cluster, Gap, PageHead, Stack};

use controls::Select;
use pages::{PAGES, Page};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    System,
    Light,
    Dark,
}

const THEMES: &[(&str, Theme)] = &[
    ("System", Theme::System),
    ("Light", Theme::Light),
    ("Dark", Theme::Dark),
];

fn apply(theme: Theme) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    else {
        return;
    };

    let _ = match theme {
        Theme::System => root.remove_attribute("data-theme"),
        Theme::Light => root.set_attribute("data-theme", "light"),
        Theme::Dark => root.set_attribute("data-theme", "dark"),
    };
}

/// The page the address names, else the first.
fn initial_page() -> usize {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    let id = hash.trim_start_matches('#');
    PAGES.iter().position(|p| p.id == id).unwrap_or_default()
}

#[function_component]
fn App() -> Html {
    let current = use_state(initial_page);
    let theme = use_state(|| Theme::System);
    let on_theme = {
        let theme = theme.clone();
        Callback::from(move |next| {
            apply(next);
            theme.set(next);
        })
    };

    let link = |(index, page): (usize, &Page)| {
        let onclick = {
            let current = current.clone();
            Callback::from(move |_: MouseEvent| current.set(index))
        };

        html! {
            <li>
                <a href={format!("#{}", page.id)} class="nav-link"
                    aria-current={(index == *current).then_some("page")} {onclick}>
                    { page.title }
                </a>
            </li>
        }
    };

    let page = &PAGES[*current];

    html! {
        <div class="showcase">
            <nav class="showcase-nav" aria-label="Components">
                <Stack gap={Gap::Loose}>
                    <strong class="showcase-name">{ "plinth" }</strong>
                    <ul class="plain-list nav-list">
                        { for PAGES.iter().enumerate().map(link) }
                    </ul>
                    <Select<Theme> label="Theme" options={THEMES} value={*theme} on_change={on_theme} />
                </Stack>
            </nav>
            <main class="showcase-main">
                <Stack gap={Gap::Loose} class="workspace-page">
                    <PageHead title={page.title} />
                    <p class="prose muted">{ page.about }</p>
                    <Cluster gap={Gap::Snug}>
                        { for page.covers.iter().map(|name| html! { <code class="chip">{ *name }</code> }) }
                    </Cluster>
                    <div key={page.id} class="contents">{ (page.render)() }</div>
                </Stack>
            </main>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
