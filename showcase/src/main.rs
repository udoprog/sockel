//! An interactive gallery of every plinth component.

mod code;
mod controls;
mod pages;
mod probe;

/// The showcase's own classes (`style/`), from plinth-build.
mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

/// The showcase's one stylesheet: plinth's CSS, then `style/`'s.
const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

use wasm_bindgen::{JsCast, closure::Closure};
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
    {
        let current = current.clone();
        use_effect_with((), move |()| {
            let listener = Closure::<dyn Fn()>::new(move || current.set(initial_page()));
            let window = web_sys::window();
            if let Some(window) = &window {
                let _ = window.add_event_listener_with_callback(
                    "hashchange",
                    listener.as_ref().unchecked_ref(),
                );
            }
            move || {
                if let Some(window) = window {
                    let _ = window.remove_event_listener_with_callback(
                        "hashchange",
                        listener.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }
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
                <a href={format!("#{}", page.id)} class={class::NAV_LINK}
                    aria-current={(index == *current).then_some("page")} {onclick}>
                    { page.title }
                </a>
            </li>
        }
    };

    let page = &PAGES[*current];

    html! {
        <div class={class::SHELL}>
            <plinth::Styles css={STYLESHEET} />
            <nav class={class::NAV} aria-label="Components">
                <Stack gap={Gap::Loose}>
                    <strong class={class::NAME}>{ "plinth" }</strong>
                    <ul class={classes!(plinth::class::PLAIN_LIST, class::NAV_LIST)}>
                        { for PAGES.iter().enumerate().map(link) }
                    </ul>
                    <Select<Theme> label="Theme" options={THEMES} value={*theme} on_change={on_theme} />
                </Stack>
            </nav>
            <main class={class::MAIN}>
                <Stack gap={Gap::Loose} class={plinth::class::WORKSPACE_PAGE}>
                    <PageHead title={page.title} />
                    <p class={classes!(class::PROSE, class::MUTED)}>{ page.about }</p>
                    <Cluster gap={Gap::Snug}>
                        { for page.covers.iter().map(|name| html! { <code class={class::CHIP}>{ *name }</code> }) }
                    </Cluster>
                    <div key={page.id} class={plinth::class::CONTENTS}>{ (page.render)() }</div>
                </Stack>
            </main>
            if !page.holds.is_empty() {
                <probe::Probe key={page.id} holds={page.holds} />
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
