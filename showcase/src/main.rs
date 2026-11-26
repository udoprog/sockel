//! The Sockel documentation site: a page of controls for every component.

mod code;
mod controls;
mod pages;
mod probe;
mod route;

/// The showcase's own classes (`style/`), from sockel-build.
mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

/// The showcase's one stylesheet: sockel's CSS, then `style/`'s.
const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

use phrasebook02::{Lang, Phrasebook, Phrases, browser};
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

use sockel::{Cluster, Gap, OverlayHost, PageHead, SearchField, SectionHead, Stack};

use controls::{Select, heading_link};
use pages::{Group, PAGES, in_group};
use route::Route;

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

const SV_SE: Lang = Lang::new("sv-SE");

/// The languages the showcase offers, by their own names.
const LANGUAGES: &[(&str, Lang)] = &[("English", Lang::EN_US), ("Svenska", SV_SE)];

/// The showcase's phrasebook: it has no messages of its own, only sockel's.
fn phrasebook() -> Phrasebook {
    let mut builder = Phrasebook::builder();
    for &(name, lang) in LANGUAGES {
        builder = builder.language(lang, name);
    }
    builder.catalog(&sockel::phrasebookcatalog01::L10N).build()
}

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

/// The route the address names.
fn current_route() -> Route {
    let hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();
    route::parse(&hash)
}

/// Shows what a route names: its section, else the page's top; and names
/// the page in the window's title.
fn show(route: &Route) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let document = window.document();
    if let (Some(document), Some(page)) = (&document, PAGES.get(route.page)) {
        document.set_title(&format!("{} · Sockel documentation", page.title));
    }
    let target = route
        .target
        .as_deref()
        .and_then(|id| document?.get_element_by_id(id));
    match target {
        Some(target) => target.scroll_into_view(),
        None => window.scroll_to_with_x_and_y(0.0, 0.0),
    }
}

#[function_component]
fn App() -> Html {
    let route = use_state(current_route);
    {
        let route = route.clone();
        use_effect_with((), move |()| {
            let listener = Closure::<dyn Fn()>::new(move || route.set(current_route()));
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
    use_effect_with((*route).clone(), show);
    let filter = use_state(String::new);
    let theme = use_state(|| Theme::System);
    let on_theme = {
        let theme = theme.clone();
        Callback::from(move |next| {
            apply(next);
            theme.set(next);
        })
    };

    let phrases = use_state(|| browser::initial(&phrasebook()));
    let on_lang = {
        let phrases = phrases.clone();
        Callback::from(move |lang: Lang| {
            let next = phrases.with_lang(lang);
            browser::set_document_lang(lang);
            next.store();
            phrases.set(next);
        })
    };

    let link = |index: usize| {
        let page = PAGES.get(index)?;
        Some(html! {
            <li>
                <a href={format!("#{}", route::slug(page))} class={class::NAV_LINK}
                    aria-current={(index == route.page).then_some("page")}>
                    { page.title }
                </a>
            </li>
        })
    };
    let groups: Vec<(Group, Vec<usize>)> = Group::ALL
        .iter()
        .map(|&group| {
            let found: Vec<usize> = in_group(group)
                .into_iter()
                .filter(|&index| PAGES.get(index).is_some_and(|p| route::matches(p, &filter)))
                .collect();
            (group, found)
        })
        .filter(|(_, found)| !found.is_empty())
        .collect();
    let on_filter = {
        let filter = filter.clone();
        Callback::from(move |text: String| filter.set(text))
    };

    let Some(page) = PAGES.get(route.page) else {
        return Html::default();
    };

    html! {
        <ContextProvider<Phrases> context={(*phrases).clone()}>
        <OverlayHost>
        <div class={class::SHELL}>
            <sockel::Styles css={STYLESHEET} />
            <nav class={class::NAV} aria-label="Components">
                <Stack gap={Gap::Loose}>
                    <strong class={class::NAME}>{ "Sockel documentation" }</strong>
                    <SearchField value={(*filter).clone()} on_input={on_filter} label="Filter components"
                        placeholder="Filter" />
                    { for groups.into_iter().map(|(group, found)| html! {
                        <Stack gap={Gap::Snug}>
                            <SectionHead title={group.title()} class={sockel::class::LABEL} />
                            <ul class={classes!(sockel::class::PLAIN_LIST, class::NAV_LIST)}>
                                { for found.into_iter().filter_map(link) }
                            </ul>
                        </Stack>
                    }) }
                    if !PAGES.iter().any(|p| route::matches(p, &filter)) {
                        <p class={classes!(class::PROSE, class::MUTED)}>{ "No component matches." }</p>
                    }
                    <Select<Theme> label="Theme" options={THEMES} value={*theme} on_change={on_theme} />
                    <Select<Lang> label="Language" options={LANGUAGES} value={phrases.lang()} on_change={on_lang} />
                </Stack>
            </nav>
            <main class={class::MAIN}>
                <Stack gap={Gap::Loose} class={sockel::class::WORKSPACE_PAGE}>
                    <PageHead title={heading_link(route::slug(page), page.title)} />
                    <p class={classes!(class::PROSE, class::MUTED)}>{ page.about }</p>
                    if !page.covers.is_empty() {
                        <Cluster gap={Gap::Snug}>
                            { for page.covers.iter().map(|name| html! {
                                <a href={format!("#{name}")} class={class::HEADING_LINK}>
                                    <code class={class::CHIP}>{ *name }</code>
                                </a>
                            }) }
                        </Cluster>
                    }
                    <div key={page.id} class={sockel::class::CONTENTS}>{ (page.render)() }</div>
                </Stack>
            </main>
            if !page.holds.is_empty() {
                <probe::Probe key={page.id} holds={page.holds} />
            }
        </div>
        </OverlayHost>
        </ContextProvider<Phrases>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
