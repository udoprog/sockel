//! The overlay layer: [`OverlayHost`], the [`Overlays`] it provides, and
//! the fallback without one (`style/_popover.scss`).

use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use crate::class;
use crate::layer::{Layer, ScrollLock, Stack};
use crate::popover::Listener;

/// The id of the fallback overlay layer, the last child of `<body>`.
pub const OVERLAY_ID: &str = class::OVERLAY;

#[derive(Clone, Debug, PartialEq)]
enum Host {
    /// The fallback's `#sockel-overlay`, made on first use.
    Body,
    Element(Element),
    /// No document (server rendering): overlays render in place.
    InPlace,
}

/// Where overlays render and their stack: an [`OverlayHost`]'s, or without
/// one the fallback, a layer at the end of `<body>` with one stack per
/// thread. Get it with [`use_overlays`].
#[derive(Clone, Debug)]
pub struct Overlays {
    host: Host,
    stack: Stack,
}

impl PartialEq for Overlays {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && Rc::ptr_eq(&self.stack, &other.stack)
    }
}

thread_local! {
    static FALLBACK: Overlays = Overlays { host: Host::Body, stack: Stack::default() };
}

impl Overlays {
    /// Without an [`OverlayHost`].
    pub fn fallback() -> Self {
        FALLBACK.with(Clone::clone)
    }

    /// `content` rendered in the overlay layer, so no scrolling or clipping
    /// box around its owner cuts it; its events still bubble through its
    /// owner. In place where there is no document (server rendering).
    pub fn portal(&self, content: Html) -> Html {
        let host = match &self.host {
            Host::Body => body_host(),
            Host::Element(element) => Some(element.clone()),
            Host::InPlace => None,
        };
        match host {
            Some(host) => create_portal(content, host),
            None => content,
        }
    }

    /// Whether anything but a tip is open on this stack: a popover, menu,
    /// list or dialog, or an overlay of the app's own ([`use_overlay`]). An
    /// app's own keyboard shortcuts can stand aside while one is.
    pub fn any_open(&self) -> bool {
        self.stack.borrow().any_open()
    }

    pub(crate) fn layer(&self) -> Layer {
        Layer::open(&self.stack, false)
    }

    pub(crate) fn tip_layer(&self) -> Layer {
        Layer::open(&self.stack, true)
    }

    pub(crate) fn lock_scroll(&self) -> ScrollLock {
        ScrollLock::take()
    }
}

/// The fallback layer, made on first use.
#[cfg(target_arch = "wasm32")]
fn body_host() -> Option<Element> {
    let document = web_sys::window()?.document()?;
    if let Some(host) = document.get_element_by_id(OVERLAY_ID) {
        return Some(host);
    }
    let host = document.create_element("div").ok()?;
    host.set_id(OVERLAY_ID);
    host.set_class_name(class::OVERLAY_HOST);
    document.body()?.append_child(&host).ok()?;
    Some(host)
}

#[cfg(not(target_arch = "wasm32"))]
fn body_host() -> Option<Element> {
    None
}

#[cfg(target_arch = "wasm32")]
fn new_host() -> Option<Element> {
    web_sys::window()?.document()?.create_element("div").ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn new_host() -> Option<Element> {
    None
}

/// `content` in the fallback overlay layer, at the end of `<body>`. A
/// component should use [`use_overlays`]`().portal(content)`, which finds
/// the app's [`OverlayHost`].
pub fn overlay(content: Html) -> Html {
    Overlays::fallback().portal(content)
}

/// The nearest [`OverlayHost`]'s overlays, or the fallback without one.
#[hook]
pub fn use_overlays() -> Overlays {
    use_context::<Overlays>().unwrap_or_else(Overlays::fallback)
}

/// Puts an overlay of the app's own (a full-screen panel, a drawer) on the
/// stack while `open`, so the overlays opened over it close first: Escape
/// calls `on_escape` only while nothing opened after it is open, and not
/// when an element in it acted on the key itself (prevented its default).
#[hook]
pub fn use_overlay(open: bool, on_escape: Callback<()>) {
    let overlays = use_overlays();
    let latest = use_mut_ref(Callback::noop);
    *latest.borrow_mut() = on_escape;

    use_effect_with(open, move |&open| {
        let held = open
            .then(|| web_sys::window().and_then(|w| w.document()))
            .flatten()
            .map(|document| {
                let layer = overlays.layer();
                Listener::new(document.into(), "keydown", false, move |e| {
                    let e = e.unchecked_ref::<web_sys::KeyboardEvent>();
                    if e.key() == "Escape" && !e.default_prevented() && layer.is_top() {
                        latest.borrow().emit(());
                    }
                })
            });
        move || drop(held)
    });
}

/// A provider's overlays: its own host, and the stack of the provider it
/// is inside, if any, so that one app keeps one stack.
fn provided(outer: Option<&Overlays>, host: Option<Element>) -> Overlays {
    Overlays {
        host: host.map_or(Host::InPlace, Host::Element),
        stack: outer.map(|o| o.stack.clone()).unwrap_or_default(),
    }
}

/// A host's classes: `overlay-host` and the app's.
pub fn overlay_host(classes: Classes) -> Classes {
    classes!(class::OVERLAY_HOST, classes)
}

#[derive(Properties, Default, PartialEq)]
pub struct OverlayHostProps {
    /// On the host element.
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// Where the overlays of everything inside it render: a layer after its
/// children, so they inherit what the element it stands in sets (colours,
/// theme), with their own stack, so Escape and a press outside close the
/// innermost. Optional: without one, overlays use a layer at the end of
/// `<body>`. It adds no box of its own.
#[function_component]
pub fn OverlayHost(props: &OverlayHostProps) -> Html {
    let outer = use_context::<Overlays>();
    let element = use_memo((), |()| new_host());
    let overlays = {
        let element = (*element).clone();
        use_memo(outer, move |outer| provided(outer.as_ref(), element))
    };

    if let Some(element) = &*element {
        let classes = overlay_host(props.class.clone()).to_string();
        if element.class_name() != classes {
            element.set_class_name(&classes);
        }
        match &props.testid {
            Some(id) => _ = element.set_attribute(crate::testid::ATTR, id),
            None => _ = element.remove_attribute(crate::testid::ATTR),
        }
    }
    let host = (*element)
        .clone()
        .map(|e| Html::VRef(e.into()))
        .unwrap_or_default();

    html! {
        <ContextProvider<Overlays> context={(*overlays).clone()}>
            { props.children.clone() }
            { host }
        </ContextProvider<Overlays>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_host_has_its_class_and_the_apps() {
        assert_eq!(names(overlay_host(Classes::new())), [class::OVERLAY_HOST]);
        assert_eq!(
            names(overlay_host(classes!(class::CARD))),
            [class::OVERLAY_HOST, class::CARD]
        );
    }

    #[test]
    fn two_providers_keep_their_own_stacks() {
        let (a, b) = (provided(None, None), provided(None, None));
        assert_ne!(a, b);
        let under = a.layer();
        let other = b.layer();
        assert!(under.is_top() && other.is_top());
        let over = a.layer();
        assert!(!under.is_top() && other.is_top());
        drop(over);
        assert!(under.is_top());
    }

    #[test]
    fn a_nested_provider_shares_the_outer_stack() {
        let outer = provided(None, None);
        let inner = provided(Some(&outer), None);
        let dialog = outer.layer();
        let menu = inner.layer();
        assert!(menu.is_top() && !dialog.is_top());
    }

    #[test]
    fn the_fallback_is_one_stack() {
        assert_eq!(Overlays::fallback(), Overlays::fallback());
        let under = Overlays::fallback().layer();
        let over = Overlays::fallback().layer();
        assert!(over.is_top() && !under.is_top());
    }

    #[derive(Properties, PartialEq)]
    struct OpenProps {
        provider: bool,
    }

    #[function_component]
    fn Open(props: &OpenProps) -> Html {
        let popover = html! {
            <crate::Popover anchor={NodeRef::default()} open=true on_dismiss={Callback::noop()}>
                { "Body" }
            </crate::Popover>
        };
        if props.provider {
            html! { <OverlayHost>{ popover }</OverlayHost> }
        } else {
            popover
        }
    }

    fn render(provider: bool) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Open>::with_props(OpenProps { provider })
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn on_the_server_overlays_render_in_place() {
        for provider in [true, false] {
            let html = render(provider);
            assert!(html.contains(class::POPOVER), "{html}");
            assert!(html.contains(">Body</div>"), "{html}");
            assert!(!html.contains(class::OVERLAY_HOST), "{html}");
        }
    }
}
