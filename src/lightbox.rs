//! Lightbox: a full-screen viewer of a set of images, one at a time, with
//! previous and next, zoom and a pan by scrolling (`style/_lightbox.scss`).

use std::cell::Cell;
use std::rc::Rc;

use phrasebook02::Args;
use phrasebook02::yew023::use_phrases_with;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::layer::ScrollLock;
use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, class, use_overlay, use_overlays};

/// One image of a [`Lightbox`].
#[derive(Clone, Debug, PartialEq)]
pub struct LightboxItem {
    pub src: AttrValue,
    /// What it shows, for assistive technology.
    pub alt: AttrValue,
    /// A line under it.
    pub caption: Option<AttrValue>,
    /// The original, linked from the head (opens in a new tab).
    pub href: Option<AttrValue>,
}

impl LightboxItem {
    pub fn new(src: impl Into<AttrValue>, alt: impl Into<AttrValue>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            caption: None,
            href: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<AttrValue>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn href(mut self, href: impl Into<AttrValue>) -> Self {
        self.href = Some(href.into());
        self
    }
}

/// The zoom steps: the image fitted to the screen, then these times as
/// wide.
pub const LIGHTBOX_ZOOMS: [f64; 5] = [1.0, 1.5, 2.0, 3.0, 4.0];

/// What a key does in a lightbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightboxKey {
    Previous,
    Next,
    ZoomIn,
    ZoomOut,
    Fit,
}

/// The key's action: the arrows step (only while fitted, as they pan a
/// zoomed image), `+` and `-` zoom, `0` fits.
pub fn lightbox_key(key: &str, zoomed: bool) -> Option<LightboxKey> {
    match key {
        "ArrowLeft" if !zoomed => Some(LightboxKey::Previous),
        "ArrowRight" if !zoomed => Some(LightboxKey::Next),
        "+" | "=" => Some(LightboxKey::ZoomIn),
        "-" | "_" => Some(LightboxKey::ZoomOut),
        "0" => Some(LightboxKey::Fit),
        _ => None,
    }
}

/// The step a swipe of `dx` by `dy` pixels makes: one image back or on for
/// a mostly sideways swipe of at least 48 pixels.
pub fn swipe_step(dx: f64, dy: f64) -> Option<isize> {
    (dx.abs() >= 48.0 && dx.abs() > dy.abs() * 1.5).then_some(if dx < 0.0 { 1 } else { -1 })
}

/// A lightbox's classes: `lightbox`, `zoomed` while larger than the screen.
pub fn lightbox(zoomed: bool) -> Classes {
    classes!(class::LIGHTBOX, zoomed.then_some(class::ZOOMED))
}

#[derive(Properties, PartialEq)]
pub struct LightboxProps {
    pub items: Vec<LightboxItem>,
    /// The image shown; closed while `None`.
    pub index: Option<usize>,
    /// Asked to show another image (the owner sets `index`).
    pub on_index: Callback<usize>,
    /// Asked to close it (the owner sets `index` to `None`).
    pub on_close: Callback<()>,
    /// Its accessible name, else the catalog's.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A full-screen viewer over a scrim: the image fitted to the screen, its
/// place in the set and its caption, previous and next (the arrow keys, a
/// swipe), zoom (`+`, `-`, `0`, a double-click), panned by scrolling once
/// zoomed, and Escape or Close to leave. The page behind holds still, and
/// overlays opened over it close first.
#[function_component]
pub fn Lightbox(props: &LightboxProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let overlays = use_overlays();
    let open = props.index.is_some_and(|i| i < props.items.len());
    let zoom = use_state(|| 0usize);
    let fitted = use_memo((), |()| Rc::new(Cell::new(0.0f64)));
    let press = use_memo((), |()| Rc::new(Cell::new(None::<(f64, f64)>)));
    let root = use_node_ref();
    let image = use_node_ref();

    use_overlay(open, props.on_close.clone());
    {
        let root = root.clone();
        use_effect_with(open, move |&open| {
            let lock = open.then(ScrollLock::take);
            let before = open
                .then(|| web_sys::window()?.document()?.active_element())
                .flatten();
            if let Some(root) = root.cast::<HtmlElement>() {
                _ = root.focus();
            }
            move || {
                drop(lock);
                if let Some(before) =
                    before.and_then(|e| wasm_bindgen::JsCast::dyn_into::<HtmlElement>(e).ok())
                {
                    _ = before.focus();
                }
            }
        });
    }
    {
        let zoom = zoom.clone();
        use_effect_with(props.index, move |_| zoom.set(0));
    }

    if !open {
        return Html::default();
    }
    let index = props.index.unwrap_or_default();
    let item = props.items[index].clone();
    let count = props.items.len();
    let zoomed = *zoom > 0;

    let step = {
        let on_index = props.on_index.clone();
        move |by: isize| {
            let next = index as isize + by;
            if (0..count as isize).contains(&next) {
                on_index.emit(next as usize);
            }
        }
    };
    let set_zoom = {
        let (zoom, fitted, image) = (zoom.clone(), fitted.clone(), image.clone());
        Callback::from(move |to: usize| {
            if *zoom == 0
                && let Some(img) = image.cast::<Element>()
            {
                fitted.set(f64::from(img.client_width()));
            }
            zoom.set(to.min(LIGHTBOX_ZOOMS.len() - 1));
        })
    };

    let onkeydown = {
        let (step, set_zoom, at) = (step.clone(), set_zoom.clone(), *zoom);
        Callback::from(move |e: KeyboardEvent| {
            if e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            let Some(action) = lightbox_key(&e.key(), at > 0) else {
                return;
            };
            e.prevent_default();
            match action {
                LightboxKey::Previous => step(-1),
                LightboxKey::Next => step(1),
                LightboxKey::ZoomIn => set_zoom.emit(at + 1),
                LightboxKey::ZoomOut => set_zoom.emit(at.saturating_sub(1)),
                LightboxKey::Fit => set_zoom.emit(0),
            }
        })
    };
    let onpointerdown = {
        let press = press.clone();
        Callback::from(move |e: PointerEvent| {
            press.set(Some((f64::from(e.client_x()), f64::from(e.client_y()))));
        })
    };
    let onpointerup = {
        let (press, step) = (press.clone(), step.clone());
        Callback::from(move |e: PointerEvent| {
            if let Some((x, y)) = press.take()
                && !zoomed
                && let Some(by) =
                    swipe_step(f64::from(e.client_x()) - x, f64::from(e.client_y()) - y)
            {
                step(by);
            }
        })
    };
    let ondblclick = {
        let set_zoom = set_zoom.clone();
        Callback::from(move |_: MouseEvent| set_zoom.emit(if zoomed { 0 } else { 2 }))
    };
    let close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let previous = {
        let step = step.clone();
        Callback::from(move |_: MouseEvent| step(-1))
    };
    let next = Callback::from(move |_: MouseEvent| step(1));
    let zoom_in = {
        let (set_zoom, at) = (set_zoom.clone(), *zoom);
        Callback::from(move |_: MouseEvent| set_zoom.emit(at + 1))
    };
    let zoom_out = {
        let at = *zoom;
        Callback::from(move |_: MouseEvent| set_zoom.emit(at.saturating_sub(1)))
    };

    let mut args = Args::new();
    args.set("position", index + 1).set("count", count);
    let position = phrases.msg("sockel-lightbox-place", &args);
    let width = zoomed.then(|| {
        format!(
            "inline-size: {}px",
            (fitted.get() * LIGHTBOX_ZOOMS[*zoom]).round()
        )
    });
    let tool = |glyph: Glyph,
                id: &str,
                onclick: Callback<MouseEvent>,
                disabled: bool,
                extra: Option<&'static str>| {
        html! {
            <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                icon={html! { <Icon {glyph} /> }} label={phrases.text(id).to_string()}
                class={classes!(extra)} {onclick} {disabled} />
        }
    };

    overlays.portal(html! {
        <div ref={root} class={classes!(lightbox(zoomed), props.class.clone())}
            role="dialog" aria-modal="true" tabindex="-1"
            aria-label={label(&phrases, &props.label, "sockel-lightbox-label")}
            data-testid={props.testid.clone()} {onkeydown}>
            <div class={class::LIGHTBOX_HEAD}>
                <span class={class::LIGHTBOX_POSITION} aria-live="polite">{ position }</span>
                <span class={class::LIGHTBOX_TOOLS}>
                    { tool(Glyph::Minus, "sockel-lightbox-zoom-out", zoom_out, !zoomed, None) }
                    { tool(Glyph::Plus, "sockel-lightbox-zoom-in", zoom_in, *zoom + 1 >= LIGHTBOX_ZOOMS.len(), None) }
                    if let Some(href) = item.href.clone() {
                        <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small} {href}
                            icon={html! { <Icon glyph={Glyph::Eye} /> }}
                            label={phrases.text("sockel-lightbox-open").to_string()} />
                    }
                    { tool(Glyph::Close, "sockel-lightbox-close", close, false, None) }
                </span>
            </div>
            <div class={class::LIGHTBOX_STAGE} {onpointerdown} {onpointerup} {ondblclick}>
                <img ref={image} key={item.src.to_string()} class={class::LIGHTBOX_IMAGE}
                    src={item.src.clone()} alt={item.alt.clone()} style={width} draggable="false" />
            </div>
            <div class={class::LIGHTBOX_FOOT}>
                { tool(Glyph::ChevronRight, "sockel-lightbox-previous", previous, index == 0, Some(class::LIGHTBOX_BACK)) }
                <span class={class::LIGHTBOX_CAPTION}>{ for item.caption.clone() }</span>
                { tool(Glyph::ChevronRight, "sockel-lightbox-next", next, index + 1 >= count, None) }
            </div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrows_step_only_while_fitted() {
        assert_eq!(
            lightbox_key("ArrowLeft", false),
            Some(LightboxKey::Previous)
        );
        assert_eq!(lightbox_key("ArrowRight", false), Some(LightboxKey::Next));
        assert_eq!(lightbox_key("ArrowRight", true), None, "it pans");
        assert_eq!(lightbox_key("+", true), Some(LightboxKey::ZoomIn));
        assert_eq!(lightbox_key("0", true), Some(LightboxKey::Fit));
        assert_eq!(lightbox_key("x", false), None);
    }

    #[test]
    fn a_sideways_swipe_steps() {
        assert_eq!(swipe_step(-80.0, 10.0), Some(1));
        assert_eq!(swipe_step(80.0, 10.0), Some(-1));
        assert_eq!(swipe_step(20.0, 0.0), None, "too short");
        assert_eq!(swipe_step(60.0, 60.0), None, "not sideways");
    }

    #[test]
    fn zoomed_is_a_modifier() {
        let names = |c: Classes| c.into_iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(names(lightbox(false)), [class::LIGHTBOX]);
        assert_eq!(names(lightbox(true)), [class::LIGHTBOX, class::ZOOMED]);
    }
}
