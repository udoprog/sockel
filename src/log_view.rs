//! LogView and LogGroup: an append-only scroll that follows its end, with a
//! jump back to the latest and groups folded to one row; the app renders
//! the rows (`style/_log-view.scss`).

use std::cell::Cell;
use std::rc::Rc;

use phrasebook02::Args;
use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, ResizeObserver};
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, class};

/// How near its end, in pixels, a log still counts as at it: a row's
/// rounding or a trackpad's last pixels do not stop it following.
pub const LOG_END_SLOP: f64 = 24.0;

/// Whether a scroll at `top` of `height`, shown `client` tall, is at its
/// end.
pub fn at_end(top: f64, height: f64, client: f64) -> bool {
    height - top - client <= LOG_END_SLOP
}

/// A log view's classes: `log-view`, `following` while it keeps to the
/// end.
pub fn log_view(following: bool) -> Classes {
    classes!(class::LOG_VIEW, following.then_some(class::FOLLOWING))
}

#[derive(Properties, PartialEq)]
pub struct LogViewProps {
    /// How many rows it holds (or any count that grows as rows come), so
    /// the rows that come while it is away from its end are counted.
    pub count: usize,
    /// Told when it starts or stops following its end (a scroll away, a
    /// jump back).
    #[prop_or_default]
    pub on_follow: Callback<bool>,
    /// Its accessible name.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The jump button's words, else the catalog's.
    #[prop_or_default]
    pub jump_label: Option<AttrValue>,
    /// Announces each new row (`aria-live="polite"`); off by default, as a
    /// busy log would talk over everything.
    #[prop_or_default]
    pub live: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The rows, oldest first.
    #[prop_or_default]
    pub children: Html,
}

fn to_end(scroll: &NodeRef) {
    if let Some(scroll) = scroll.cast::<Element>() {
        scroll.set_scroll_top(scroll.scroll_height());
    }
}

/// A scroll of rows (`role="log"`) that keeps to its end as rows come and
/// as they grow (an image loading, a group opening), until the reader
/// scrolls away; then it holds still, and a "Jump to latest" button,
/// counting the rows since, floats over its foot without moving them. It
/// fills its container's height.
#[function_component]
pub fn LogView(props: &LogViewProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let scroll = use_node_ref();
    let content = use_node_ref();
    let following = use_state(|| true);
    // The state the scroll and resize handlers read without a render.
    let stuck = use_memo((), |()| Rc::new(Cell::new(true)));
    let seen = use_mut_ref(|| props.count);
    let unseen = use_state(|| 0usize);

    // After every render: keep to the end while following.
    {
        let (scroll, stuck) = (scroll.clone(), stuck.clone());
        use_effect(move || {
            if stuck.get() {
                to_end(&scroll);
            }
        });
    }
    // Rows growing without a render (an image loading) keep it at the end.
    {
        let (scroll, content, stuck) = (scroll.clone(), content.clone(), stuck.clone());
        use_effect_with((), move |()| {
            let observer = content.cast::<Element>().and_then(|inner| {
                let callback = Closure::<dyn FnMut()>::new(move || {
                    if stuck.get() {
                        to_end(&scroll);
                    }
                });
                let observer = ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()?;
                observer.observe(&inner);
                Some((observer, callback))
            });
            move || {
                if let Some((observer, _)) = observer {
                    observer.disconnect();
                }
            }
        });
    }
    // Rows that come while away are counted.
    {
        let (seen, unseen, stuck) = (seen.clone(), unseen.clone(), stuck.clone());
        use_effect_with(props.count, move |&count| {
            let before = std::mem::replace(&mut *seen.borrow_mut(), count);
            if !stuck.get() && count > before {
                unseen.set(*unseen + (count - before));
            }
        });
    }

    let follow = {
        let (following, stuck, unseen) = (following.clone(), stuck.clone(), unseen.clone());
        let on_follow = props.on_follow.clone();
        move |on: bool| {
            if stuck.replace(on) != on {
                following.set(on);
                on_follow.emit(on);
            }
            if on {
                unseen.set(0);
            }
        }
    };
    let onscroll = {
        let (scroll, follow) = (scroll.clone(), follow.clone());
        Callback::from(move |_: Event| {
            if let Some(s) = scroll.cast::<Element>() {
                follow(at_end(
                    f64::from(s.scroll_top()),
                    f64::from(s.scroll_height()),
                    f64::from(s.client_height()),
                ));
            }
        })
    };
    let jump = {
        let scroll = scroll.clone();
        Callback::from(move |_: MouseEvent| {
            follow(true);
            to_end(&scroll);
        })
    };

    let words = if *unseen > 0 {
        let mut args = Args::new();
        args.set("count", *unseen);
        AttrValue::from(phrases.msg("sockel-log-view-new", &args))
    } else {
        label(&phrases, &props.jump_label, "sockel-log-view-to-latest")
    };

    html! {
        <div class={classes!(log_view(*following), props.class.clone())} data-testid={props.testid.clone()}>
            <div ref={scroll} class={class::LOG_VIEW_SCROLL} role="log"
                aria-label={props.label.clone()}
                aria-live={if props.live { "polite" } else { "off" }} tabindex="0" {onscroll}>
                <div ref={content} class={class::LOG_VIEW_ROWS}>{ props.children.clone() }</div>
            </div>
            <div class={class::LOG_VIEW_JUMP} hidden={*following}>
                <Button size={ButtonSize::Small} variant={ButtonVariant::Primary}
                    icon={html! { <Icon glyph={Glyph::ChevronDown} /> }} onclick={jump}>
                    { words }
                </Button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct LogGroupProps {
    /// Its row while folded and its head while open.
    pub summary: Html,
    /// How many rows it holds, shown after the summary.
    #[prop_or_default]
    pub count: Option<usize>,
    /// Open or folded, set by the owner; its own state while `None`.
    #[prop_or_default]
    pub open: Option<bool>,
    /// Open at first, while the owner does not set `open`.
    #[prop_or_default]
    pub default_open: bool,
    /// Told the new state on a toggle.
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Its rows, rendered only while open.
    #[prop_or_default]
    pub children: Html,
}

/// A log group's classes: `log-group`, `open` while open.
pub fn log_group(open: bool) -> Classes {
    classes!(class::LOG_GROUP, open.then_some(class::OPEN))
}

/// A run of rows folded to one: a head (a button, `aria-expanded`) with its
/// mark, summary and count, and its rows under it while open.
#[function_component]
pub fn LogGroup(props: &LogGroupProps) -> Html {
    let own = use_state(|| props.default_open);
    let open = props.open.unwrap_or(*own);
    let onclick = {
        let (own, on_toggle) = (own.clone(), props.on_toggle.clone());
        Callback::from(move |_: MouseEvent| {
            own.set(!open);
            on_toggle.emit(!open);
        })
    };
    html! {
        <div class={classes!(log_group(open), props.class.clone())} data-testid={props.testid.clone()}>
            <button type="button" class={class::LOG_GROUP_HEAD} aria-expanded={open.to_string()} {onclick}>
                <span class={class::LOG_GROUP_MARK} aria-hidden="true"><Icon glyph={Glyph::ChevronRight} /></span>
                <span class={class::LOG_GROUP_SUMMARY}>{ props.summary.clone() }</span>
                if let Some(count) = props.count {
                    <span class={class::LOG_GROUP_COUNT}>{ count }</span>
                }
            </button>
            if open {
                <div class={class::LOG_GROUP_ROWS}>{ props.children.clone() }</div>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_the_end_counts_as_at_it() {
        assert!(at_end(976.0, 1000.0, 0.0));
        assert!(at_end(500.0, 1000.0, 500.0));
        assert!(!at_end(400.0, 1000.0, 500.0));
    }

    #[test]
    fn a_folded_group_renders_its_head_alone() {
        #[derive(Properties, PartialEq)]
        struct Shown {
            open: bool,
        }
        #[function_component]
        fn Show(props: &Shown) -> Html {
            html! {
                <LogView count={3} label="Transcript">
                    <LogGroup summary={html! { "3 tool calls" }} count={Some(3)} open={Some(props.open)}>
                        <p>{ "hidden row" }</p>
                    </LogGroup>
                </LogView>
            }
        }
        let render = |open| {
            futures::executor::block_on(
                yew::LocalServerRenderer::<Show>::with_props(Shown { open }).render(),
            )
        };
        let folded = render(false);
        assert!(folded.contains("aria-expanded=\"false\""), "{folded}");
        assert!(!folded.contains("hidden row"));
        assert!(folded.contains("role=\"log\""));
        assert!(folded.contains(class::FOLLOWING), "it follows at first");
        let open = render(true);
        assert!(open.contains("hidden row"));
        assert!(open.contains(&format!("{} {}", class::LOG_GROUP, class::OPEN)));
    }
}
