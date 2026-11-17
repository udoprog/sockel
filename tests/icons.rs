//! The icon set: the built-in one without a provider, the app's with one.

use plinth::*;
use yew::LocalServerRenderer;
use yew::prelude::*;

#[function_component]
fn Bare() -> Html {
    html! { <Icon glyph={Glyph::Close} /> }
}

#[function_component]
fn Provided() -> Html {
    let set = IconSet {
        close: html! { <b>{ "custom close" }</b> },
        ..IconSet::default()
    };
    html! {
        <ContextProvider<IconSet> context={set}>
            <Icon glyph={Glyph::Close} />
            <Icon glyph={Glyph::Check} />
        </ContextProvider<IconSet>>
    }
}

fn render<C: BaseComponent<Properties = ()>>() -> String {
    futures::executor::block_on(LocalServerRenderer::<C>::new().hydratable(false).render())
}

#[test]
fn without_a_provider_the_built_in_glyph_is_used() {
    let html = render::<Bare>();
    assert!(html.contains("<svg"), "{html}");
    assert!(html.contains("currentColor"), "{html}");
    assert!(html.contains(class::GLYPH), "{html}");
}

#[test]
fn a_provider_replaces_only_the_glyphs_it_sets() {
    let html = render::<Provided>();
    assert!(html.contains("custom close"), "{html}");
    assert!(!html.contains("M4 4l8 8"), "{html}");
    assert!(
        html.contains("M3.5 8.5"),
        "the built-in check remains: {html}"
    );
}

#[test]
fn the_box_is_the_same_whichever_set_fills_it() {
    let box_of = |html: &str| {
        html.split("</span>")
            .next()
            .unwrap()
            .split('>')
            .next()
            .unwrap()
            .to_owned()
    };
    assert_eq!(box_of(&render::<Bare>()), box_of(&render::<Provided>()));
}
