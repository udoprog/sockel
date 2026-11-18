//! EmptyState (`style/_notices.scss`).

use crate::{button::is_empty, class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EmptyStateProps {
    #[prop_or_default]
    pub class: Classes,
    /// What there is none of; left out when empty (the hint alone then says it).
    pub title: AttrValue,
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The icon is its own picture: no box, disc or fill round it, and no gap below.
    #[prop_or_default]
    pub bare_icon: bool,
    /// What to do about it.
    #[prop_or_default]
    pub hint: Option<Html>,
    /// One row high (`--control-height`), for inside a list or a table.
    #[prop_or_default]
    pub compact: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The actions (buttons), after the text.
    #[prop_or_default]
    pub children: Html,
}

/// An empty state's classes: `empty-state`, `compact` for one row.
pub fn empty_state(compact: bool) -> Classes {
    classes!(class::EMPTY_STATE, compact.then_some(class::COMPACT))
}

/// What stands where a list or page has nothing: an icon, a title, a hint
/// and actions. Compact, it is one row high whatever it says, its text cut.
#[function_component]
pub fn EmptyState(props: &EmptyStateProps) -> Html {
    html! {
        <div class={classes!(empty_state(props.compact), props.class.clone())}
            data-testid={props.testid.clone()}>
            if let Some(icon) = &props.icon {
                <span class={classes!(class::EMPTY_STATE_ICON, props.bare_icon.then_some(class::BARE))} aria-hidden="true">{ icon.clone() }</span>
            }
            if !props.title.is_empty() {
                <p class={class::EMPTY_STATE_TITLE}>{ props.title.clone() }</p>
            }
            if let Some(hint) = &props.hint {
                <p class={class::EMPTY_STATE_HINT}>{ hint.clone() }</p>
            }
            if !is_empty(&props.children) {
                <div class={class::EMPTY_STATE_ACTIONS}>{ props.children.clone() }</div>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn compact_is_a_modifier() {
        assert_eq!(names(empty_state(false)), [class::EMPTY_STATE]);
        assert_eq!(
            names(empty_state(true)),
            [class::EMPTY_STATE, class::COMPACT]
        );
    }

    #[test]
    fn no_children_is_empty() {
        assert!(is_empty(&html! {}));
        assert!(!is_empty(&html! { <b /> }));
    }
}
