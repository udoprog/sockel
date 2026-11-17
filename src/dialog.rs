//! Dialog (`style/_dialog.scss`).

use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::layer::Layer;
use crate::popover::{Listener, focus_lost};
use crate::{
    ActionGroup, Button, ButtonSize, ButtonVariant, Dismiss, Glyph, Icon, SectionHead, class,
    use_overlays,
};

/// A dialog's size: its width, and whether its height is fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// `--dialog-width`, as tall as its content from `--dialog-min-height`.
    #[default]
    Default,
    /// `--dialog-width-wide`.
    Wide,
    /// `--dialog-width`, and `--dialog-height-tall` whatever it holds.
    Tall,
}

/// What may close a dialog besides its own actions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogDismiss {
    /// Escape, its close button and a press on the scrim.
    #[default]
    Any,
    /// Escape and its close button, not a press on the scrim (a form a
    /// stray press should not throw away).
    NoOutside,
    /// Only its actions: no close button, and Escape does nothing.
    Actions,
}

impl DialogDismiss {
    /// Whether this mode lets `why` close the dialog.
    pub fn allows(self, why: Dismiss) -> bool {
        match self {
            DialogDismiss::Any => true,
            DialogDismiss::NoOutside => why != Dismiss::Outside,
            DialogDismiss::Actions => false,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    /// Asked to close it (the owner sets `open` to false).
    pub on_dismiss: Callback<Dismiss>,
    /// Its heading and accessible name.
    pub title: AttrValue,
    /// Before the title, in the accent.
    #[prop_or_default]
    pub icon: Option<Html>,
    #[prop_or_default]
    pub size: DialogSize,
    #[prop_or_default]
    pub dismiss: DialogDismiss,
    /// A sheet from the bottom below the phone breakpoint; on by default.
    #[prop_or(true)]
    pub sheet: bool,
    #[prop_or(AttrValue::Static("Close"))]
    pub close_label: AttrValue,
    /// The close button's `data-testid`.
    #[prop_or_default]
    pub close_testid: Option<AttrValue>,
    /// The foot's actions at its end, the primary last.
    #[prop_or_default]
    pub actions: Html,
    /// The foot's actions at its start, apart from the rest (a Delete).
    #[prop_or_default]
    pub actions_start: Html,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A dialog's classes: `dialog`, `wide` or `tall`, and `sheet` when it
/// becomes one on a phone.
pub fn dialog(size: DialogSize, sheet: bool) -> Classes {
    classes!(
        class::DIALOG,
        match size {
            DialogSize::Default => None,
            DialogSize::Wide => Some(class::WIDE),
            DialogSize::Tall => Some(class::TALL),
        },
        sheet.then_some(class::SHEET)
    )
}

/// What Tab reaches: not disabled, and not a roving option held at `-1`.
const FOCUSABLE: &str = "button:not(:disabled):not([tabindex='-1']), \
         input:not(:disabled):not([type=hidden]):not([tabindex='-1']), \
         select:not(:disabled):not([tabindex='-1']), textarea:not(:disabled):not([tabindex='-1']), \
         a[href]:not([tabindex='-1']), [tabindex]:not([tabindex='-1'])";

/// The enabled controls in `root` that Tab reaches, in order.
fn focusable(root: &Element) -> Vec<HtmlElement> {
    let Ok(list) = root.query_selector_all(FOCUSABLE) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into::<HtmlElement>().ok())
        .filter(|e| e.offset_parent().is_some())
        .collect()
}

fn focus(element: &HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    _ = element.focus_with_options(&options);
}

fn has(element: &Element, class: &str) -> bool {
    element.class_list().contains(class)
}

/// Everything outside `layer` it puts out of reach that was not already:
/// the page around the path from `<body>` to it, the overlays open before
/// it in its own host, and those open in any other host. Toasts stay live.
fn outside(layer: &Element) -> Vec<Element> {
    let mut out = Vec::new();
    let mut on_path = layer.clone();
    while let Some(parent) = on_path.parent_element() {
        let own_host = has(&parent, class::OVERLAY_HOST);
        let children = parent.children();
        for element in (0..children.length()).filter_map(|i| children.item(i)) {
            if element.is_same_node(Some(&on_path)) {
                if own_host {
                    break;
                }
            } else if has(&element, class::OVERLAY_HOST) {
                let overlays = element.children();
                out.extend((0..overlays.length()).filter_map(|i| overlays.item(i)));
            } else {
                out.push(element);
            }
        }
        if parent.tag_name() == "BODY" {
            break;
        }
        on_path = parent;
    }
    out.retain(|e| !e.has_attribute("inert") && !has(e, class::TOAST_HOST));
    out
}

/// A modal box over a scrim in the overlay layer, while `open`: its head
/// (icon, title, close), a body that scrolls, and a foot of actions pinned
/// below it. The page behind is out of reach and does not scroll or move;
/// Tab stays inside; Escape and a press on the scrim close the innermost
/// overlay only, as `dismiss` allows; focus returns where it was.
#[function_component]
pub fn Dialog(props: &DialogProps) -> Html {
    let layer_ref = use_node_ref();
    let panel_ref = use_node_ref();
    let overlays = use_overlays();
    let latest = use_mut_ref(Callback::noop);
    *latest.borrow_mut() = props.on_dismiss.clone();
    let mode = use_mut_ref(DialogDismiss::default);
    *mode.borrow_mut() = props.dismiss;
    // Its place on the stack while open, and whether a press began on the scrim.
    let layer = use_mut_ref(|| None::<Rc<Layer>>);
    let pressed = use_mut_ref(|| false);

    {
        let (layer_ref, panel_ref, layer) = (layer_ref.clone(), panel_ref.clone(), layer.clone());
        let (latest, mode, overlays) = (latest.clone(), mode.clone(), overlays.clone());
        use_effect_with(props.open, move |&open| {
            let mut held = None;
            let mut kept = Vec::new();
            let mut before = None;
            let mut lock = None;

            if open {
                let top = Rc::new(overlays.layer());
                *layer.borrow_mut() = Some(top.clone());
                lock = Some(overlays.lock_scroll());
                let document = web_sys::window().and_then(|w| w.document());
                before = document
                    .as_ref()
                    .and_then(|d| d.active_element())
                    .and_then(|e| e.dyn_into::<HtmlElement>().ok());

                if let Some(root) = layer_ref.cast::<Element>() {
                    kept = outside(&root);
                    for element in &kept {
                        _ = element.set_attribute("inert", "");
                    }
                }
                if let Some(panel) = panel_ref.cast::<HtmlElement>() {
                    let first = panel
                        .query_selector("[autofocus], [data-autofocus]")
                        .ok()
                        .flatten()
                        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
                        .or_else(|| {
                            panel
                                .query_selector(&format!(".{}", class::DIALOG_BODY))
                                .ok()
                                .flatten()
                                .and_then(|body| focusable(&body).into_iter().next())
                        });
                    focus(first.as_ref().unwrap_or(&panel));
                }

                if let Some(document) = document {
                    let keyed = move |e: web_sys::Event| {
                        if !top.is_top() {
                            return;
                        }
                        let e = e.unchecked_ref::<web_sys::KeyboardEvent>();
                        match e.key().as_str() {
                            "Escape" => {
                                if mode.borrow().allows(Dismiss::Escape) {
                                    latest.borrow().emit(Dismiss::Escape);
                                }
                            }
                            "Tab" => {
                                let Some(panel) = panel_ref.cast::<Element>() else {
                                    return;
                                };
                                let all = focusable(&panel);
                                let (Some(first), Some(last)) = (all.first(), all.last()) else {
                                    e.prevent_default();
                                    return;
                                };
                                let active =
                                    panel.owner_document().and_then(|d| d.active_element());
                                let at = |el: &HtmlElement| {
                                    active.as_ref().is_some_and(|a| el.is_same_node(Some(a)))
                                };
                                let inside =
                                    active.as_ref().is_some_and(|a| panel.contains(Some(a)));
                                if e.shift_key() && (at(first) || !inside) {
                                    e.prevent_default();
                                    focus(last);
                                } else if !e.shift_key() && (at(last) || !inside) {
                                    e.prevent_default();
                                    focus(first);
                                }
                            }
                            _ => {}
                        }
                    };
                    held = Some(Listener::new(document.into(), "keydown", false, keyed));
                }
            }

            move || {
                drop(held);
                *layer.borrow_mut() = None;
                for element in &kept {
                    _ = element.remove_attribute("inert");
                }
                drop(lock);
                let within = layer_ref.cast::<Element>().is_some_and(|root| {
                    root.owner_document()
                        .and_then(|d| d.active_element())
                        .is_some_and(|a| root.contains(Some(&a)))
                });
                if let Some(before) = before.filter(|b| b.is_connected())
                    && (within || focus_lost())
                {
                    focus(&before);
                }
            }
        });
    }

    if !props.open {
        return Html::default();
    }

    let on_scrim = |layer_ref: &NodeRef, target: Option<web_sys::EventTarget>| {
        let target = target.and_then(|t| t.dyn_into::<web_sys::Node>().ok());
        layer_ref
            .get()
            .is_some_and(|l| l.is_same_node(target.as_ref()))
    };
    let onpointerdown = {
        let (layer_ref, layer, pressed) = (layer_ref.clone(), layer.clone(), pressed.clone());
        Callback::from(move |e: PointerEvent| {
            let top = layer.borrow().as_ref().is_some_and(|l| l.is_top());
            *pressed.borrow_mut() = top && on_scrim(&layer_ref, e.target());
        })
    };
    let onclick = {
        let (layer_ref, latest) = (layer_ref.clone(), latest.clone());
        let allowed = props.dismiss.allows(Dismiss::Outside);
        Callback::from(move |e: MouseEvent| {
            // Pressed and let go on the scrim, so a drag out of a field does not close it.
            if pressed.replace(false) && allowed && on_scrim(&layer_ref, e.target()) {
                latest.borrow().emit(Dismiss::Outside);
            }
        })
    };

    let close = props.dismiss.allows(Dismiss::Close).then(|| {
        let onclick = props.on_dismiss.reform(|_: MouseEvent| Dismiss::Close);
        html! {
            <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                icon={html! { <Icon glyph={Glyph::Close} /> }}
                label={props.close_label.clone()} testid={props.close_testid.clone()} {onclick} />
        }
    });
    let foot =
        (props.actions != Html::default() || props.actions_start != Html::default()).then(|| {
            html! {
                <div class={class::DIALOG_FOOT}>
                    if props.actions_start != Html::default() {
                        <ActionGroup>{ props.actions_start.clone() }</ActionGroup>
                    }
                    <ActionGroup end=true>{ props.actions.clone() }</ActionGroup>
                </div>
            }
        });

    overlays.portal(html! {
        <div ref={layer_ref}
            class={classes!(class::DIALOG_LAYER, props.sheet.then_some(class::SHEET))}
            {onpointerdown} {onclick}>
            <div ref={panel_ref}
                class={classes!(dialog(props.size, props.sheet), props.class.clone())}
                role="dialog" aria-modal="true" aria-label={props.title.clone()} tabindex="-1"
                data-testid={props.testid.clone()}>
                <SectionHead class={class::DIALOG_HEAD} title={html! { props.title.clone() }}
                    icon={props.icon.clone()}>
                    { for close }
                </SectionHead>
                <div class={class::DIALOG_BODY}>{ props.children.clone() }</div>
                { for foot }
            </div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn every_focusable_clause_skips_a_roving_tabindex() {
        assert!(
            FOCUSABLE.split(", ").all(|c| c.contains("[tabindex='-1']")),
            "{FOCUSABLE}"
        );
    }

    #[test]
    fn size_and_sheet_are_modifiers() {
        assert_eq!(names(dialog(DialogSize::Default, false)), [class::DIALOG]);
        assert_eq!(
            names(dialog(DialogSize::Wide, false)),
            [class::DIALOG, class::WIDE]
        );
        assert_eq!(
            names(dialog(DialogSize::Tall, true)),
            [class::DIALOG, class::TALL, class::SHEET]
        );
    }

    #[test]
    fn dismiss_modes_allow_what_they_say() {
        use Dismiss::*;
        let all = [Outside, Escape, Close];
        assert!(all.iter().all(|&w| DialogDismiss::Any.allows(w)));
        assert_eq!(
            all.map(|w| DialogDismiss::NoOutside.allows(w)),
            [false, true, true]
        );
        assert!(!all.iter().any(|&w| DialogDismiss::Actions.allows(w)));
    }

    fn render(open: bool, dismiss: DialogDismiss, actions: Html) -> String {
        let props = DialogProps {
            open,
            on_dismiss: Callback::noop(),
            title: "Rename".into(),
            icon: None,
            size: DialogSize::Default,
            dismiss,
            sheet: true,
            close_label: "Close".into(),
            close_testid: Some("close".into()),
            actions,
            actions_start: Html::default(),
            class: Classes::new(),
            testid: None,
            children: html! { "Body" },
        };
        futures::executor::block_on(
            yew::LocalServerRenderer::<Dialog>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn it_renders_only_while_open_as_a_modal() {
        assert_eq!(render(false, DialogDismiss::Any, Html::default()), "");
        let html = render(true, DialogDismiss::Any, html! { "Save" });
        assert!(html.contains("role=\"dialog\""), "{html}");
        assert!(html.contains("aria-modal=\"true\""), "{html}");
        assert!(html.contains("aria-label=\"Rename\""), "{html}");
        assert!(html.contains("aria-label=\"Close\""), "{html}");
        assert!(html.contains(class::DIALOG_FOOT), "{html}");
        assert!(html.contains(class::DIALOG_LAYER), "{html}");
        assert!(html.contains("data-testid=\"close\""), "{html}");
    }

    #[test]
    fn only_actions_close_it_without_a_close_button_or_foot_when_none() {
        let html = render(true, DialogDismiss::Actions, Html::default());
        assert!(!html.contains("aria-label=\"Close\""), "{html}");
        assert!(!html.contains(class::DIALOG_FOOT), "{html}");
    }
}
