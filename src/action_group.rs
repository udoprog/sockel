//! ActionGroup (`style/_actions.scss`).

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ActionGroupProps {
    /// At the end of its row, or of its own line.
    #[prop_or_default]
    pub end: bool,
    /// Quiet icon actions, closer together.
    #[prop_or_default]
    pub quiet: bool,
    #[prop_or_default]
    pub nowrap: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// An action group's classes: `end` at its row's end, `quiet` for quiet
/// icon actions, `wrap` false on one line.
pub fn action_group(end: bool, quiet: bool, wrap: bool) -> Classes {
    classes!(
        "action-group",
        end.then_some("end"),
        quiet.then_some("quiet"),
        (!wrap).then_some("nowrap")
    )
}

/// The buttons that end a form, a panel or a section, the primary first.
#[function_component]
pub fn ActionGroup(props: &ActionGroupProps) -> Html {
    let classes = action_group(props.end, props.quiet, !props.nowrap);

    html! {
        <div class={classes!(classes, props.class.clone())} data-testid={props.testid.clone()}>
            { props.children.clone() }
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
    fn modifiers_follow_the_props() {
        assert_eq!(names(action_group(false, false, true)), ["action-group"]);
        assert_eq!(
            names(action_group(true, true, false)),
            ["action-group", "end", "quiet", "nowrap"]
        );
    }
}
