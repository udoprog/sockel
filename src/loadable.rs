//! Loadable (`style/_skeleton.scss`).

use crate::testid::with_testid;
use phrasebook02::yew023::use_phrases_with;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::timer::Timer;
use crate::{EmptyState, ErrorBox, Skeleton, SkeletonShape, class};

/// What a [`Loadable`]'s data is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadState {
    /// Loading, or loading again: the placeholder shows.
    #[default]
    Loading,
    Ready,
    /// Loaded, and there is nothing: shown after `empty_delay`.
    Empty,
    Error,
}

/// What a [`Loadable`] shows: the empty state waits for its delay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Placeholder,
    Ready,
    Empty,
    Error,
}

fn phase(state: LoadState, waited: bool) -> Phase {
    match state {
        LoadState::Loading => Phase::Placeholder,
        LoadState::Empty if !waited => Phase::Placeholder,
        LoadState::Empty => Phase::Empty,
        LoadState::Ready => Phase::Ready,
        LoadState::Error => Phase::Error,
    }
}

/// Whether the empty state may show: `ms` have passed since `active` began.
/// A delay of zero shows at once.
#[hook]
fn use_waited(active: bool, ms: u32) -> bool {
    let waited = use_state(|| false);
    {
        let waited = waited.clone();
        use_effect_with((active, ms), move |&(active, ms)| {
            let mut timer = None;
            if active && ms > 0 {
                timer = Timer::start(ms, move || waited.set(true));
            } else {
                waited.set(false);
            }
            move || drop(timer)
        });
    }
    active && (ms == 0 || *waited)
}

#[derive(Properties, PartialEq)]
pub struct LoadableProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub state: LoadState,
    /// Rows the box is at least as high as while it loads, and how many the
    /// default placeholder has.
    #[prop_or(3)]
    pub rows: usize,
    /// Replaces the placeholder (rows of [`Skeleton`]).
    #[prop_or_default]
    pub placeholder: Option<Html>,
    /// What a screen reader hears while it loads, else the catalog's.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// Replaces the default empty state.
    #[prop_or_default]
    pub empty: Option<Html>,
    /// The default empty state's title, else the catalog's.
    #[prop_or_default]
    pub empty_title: Option<AttrValue>,
    /// Milliseconds the empty state waits, so it never flashes before data.
    #[prop_or(250)]
    pub empty_delay: u32,
    /// The error's message, else the catalog's.
    #[prop_or_default]
    pub error: Option<AttrValue>,
    #[prop_or_default]
    pub error_cause: Option<AttrValue>,
    /// Shows the error's retry button.
    #[prop_or_default]
    pub onretry: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub retrying: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The loaded content.
    #[prop_or_default]
    pub children: Html,
}

impl Default for LoadableProps {
    fn default() -> Self {
        Self {
            class: Default::default(),
            state: Default::default(),
            rows: ::yew::html::IntoPropValue::into_prop_value(3),
            placeholder: Default::default(),
            label: Default::default(),
            empty: Default::default(),
            empty_title: Default::default(),
            empty_delay: ::yew::html::IntoPropValue::into_prop_value(250),
            error: Default::default(),
            error_cause: Default::default(),
            onretry: Default::default(),
            retrying: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
}

/// One box for content that loads: a placeholder while loading, the content
/// when ready, an empty state or an error. While it loads again the box is
/// at least as high as when it last settled, and at least `rows` rows high
/// the first time, so the page below does not move. `aria-busy` and a status
/// for a screen reader show while it loads.
#[function_component]
pub fn Loadable(props: &LoadableProps) -> Html {
    let waited = use_waited(props.state == LoadState::Empty, props.empty_delay);
    let phase = phase(props.state, waited);
    let root = use_node_ref();
    let kept = use_mut_ref(|| 0.0_f64);
    let phrases = use_phrases_with(&L10N);

    {
        let root = root.clone();
        let kept = kept.clone();
        use_effect(move || {
            if phase != Phase::Placeholder
                && let Some(el) = root.cast::<HtmlElement>()
            {
                *kept.borrow_mut() = f64::from(el.offset_height());
            }
        });
    }

    let busy = phase == Phase::Placeholder;
    let style = busy.then(|| {
        format!(
            "--loadable-rows: {}; --loadable-kept: {}px",
            props.rows,
            *kept.borrow()
        )
    });
    let content = match phase {
        Phase::Placeholder => props.placeholder.clone().unwrap_or_else(|| {
            html! { <Skeleton shape={SkeletonShape::Row} count={props.rows.max(1)} /> }
        }),
        Phase::Ready => props.children.clone(),
        Phase::Empty => props.empty.clone().unwrap_or_else(|| {
            html! { <EmptyState title={label(&phrases, &props.empty_title, "sockel-loadable-empty")} /> }
        }),
        Phase::Error => html! {
            <ErrorBox cause={props.error_cause.clone()} onretry={props.onretry.clone()}
                retrying={props.retrying}>
                { label(&phrases, &props.error, "sockel-loadable-error") }
            </ErrorBox>
        },
    };

    with_testid(
        html! {
            <div ref={root} class={classes!(class::LOADABLE, props.class.clone())} {style}
                aria-busy={busy.then_some("true")}>
                <span class={class::VISUALLY_HIDDEN} role="status">
                    if busy { { label(&phrases, &props.label, "sockel-loadable-loading") } }
                </span>
                { content }
            </div>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use yew::LocalServerRenderer;

    #[test]
    fn the_empty_state_waits_for_its_delay() {
        assert_eq!(phase(LoadState::Empty, false), Phase::Placeholder);
        assert_eq!(phase(LoadState::Empty, true), Phase::Empty);
    }

    #[test]
    fn the_other_states_do_not_wait() {
        assert_eq!(phase(LoadState::Loading, true), Phase::Placeholder);
        assert_eq!(phase(LoadState::Ready, false), Phase::Ready);
        assert_eq!(phase(LoadState::Error, false), Phase::Error);
    }

    #[derive(Properties, PartialEq)]
    struct Shown {
        state: LoadState,
        delay: u32,
    }

    #[function_component]
    fn Show(props: &Shown) -> Html {
        html! {
            <Loadable state={props.state} empty_delay={props.delay} rows={2}>
                <b>{ "content" }</b>
            </Loadable>
        }
    }

    fn render(state: LoadState, delay: u32) -> String {
        futures::executor::block_on(
            LocalServerRenderer::<Show>::with_props(Shown { state, delay })
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn loading_is_busy_with_a_status_and_skeleton_rows() {
        let html = render(LoadState::Loading, 250);
        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(
            html.contains(&format!("class=\"{}\">Loading<", class::VISUALLY_HIDDEN)),
            "{html}"
        );
        assert_eq!(
            html.matches(&format!("{} {}", class::SKELETON, class::ROW))
                .count(),
            2,
            "{html}"
        );
        assert!(!html.contains("content"), "{html}");
    }

    #[test]
    fn ready_is_not_busy_and_its_status_is_empty() {
        let html = render(LoadState::Ready, 250);
        assert!(!html.contains("aria-busy"), "{html}");
        assert!(
            html.contains(&format!("class=\"{}\"></span>", class::VISUALLY_HIDDEN)),
            "{html}"
        );
        assert!(html.contains("<b>content</b>"), "{html}");
    }

    #[test]
    fn an_empty_state_waits_before_it_shows_and_no_wait_shows_at_once() {
        assert!(render(LoadState::Empty, 250).contains("aria-busy=\"true\""));
        let html = render(LoadState::Empty, 0);
        assert!(html.contains(class::EMPTY_STATE), "{html}");
        assert!(!html.contains("aria-busy"), "{html}");
    }

    #[test]
    fn an_error_is_an_alert_with_a_retry() {
        let html = futures::executor::block_on(
            LocalServerRenderer::<Retry>::new()
                .hydratable(false)
                .render(),
        );
        assert!(html.contains("role=\"alert\""), "{html}");
        assert!(html.contains("Try again"), "{html}");
    }

    #[function_component]
    fn Retry() -> Html {
        html! { <Loadable state={LoadState::Error} onretry={Callback::noop()} /> }
    }
}
