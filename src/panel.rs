//! Panel: a bordered ground over a list or a view, holding its filters or a
//! short form (`style/_panel.scss`).

use crate::class;
use yew::prelude::*;

/// What a panel stands on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PanelGround {
    /// A sunken ground inside a ring.
    #[default]
    Sunk,
    /// A rounded accent tint: one line of a form, as a chip.
    Tint,
    /// No ground or ring, only the spacing.
    Bare,
}

/// A panel's classes; `line` lays it out as one row, `centered` centres it
/// in its column.
pub fn panel(ground: PanelGround, line: bool, centered: bool) -> Classes {
    let ground = match ground {
        PanelGround::Sunk => None,
        PanelGround::Tint => Some(class::TINT),
        PanelGround::Bare => Some(class::BARE),
    };

    classes!(
        class::PANEL_BOX,
        line.then_some(class::LINE),
        ground,
        centered.then_some(class::CENTER)
    )
}

#[derive(Properties, PartialEq)]
pub struct PanelProps {
    #[prop_or_default]
    pub ground: PanelGround,
    /// One row that wraps, rather than a column.
    #[prop_or_default]
    pub line: bool,
    /// Centred in its column.
    #[prop_or_default]
    pub centered: bool,
    /// In `--text-sm` type (`class::SMALL`).
    #[prop_or_default]
    pub small: bool,
    /// What the panel holds, for assistive technology (a `group`'s name).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// As wide as what it holds, never wider than its column.
#[function_component]
pub fn Panel(props: &PanelProps) -> Html {
    let role = props.label.is_some().then_some("group");

    html! {
        <div {role} aria-label={props.label.clone()}
            class={classes!(
                panel(props.ground, props.line, props.centered),
                props.small.then_some(class::SMALL),
                props.class.clone()
            )}
            data-testid={props.testid.clone()}>
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
    fn a_panel_names_only_what_differs_from_its_defaults() {
        assert_eq!(
            names(panel(PanelGround::Sunk, false, false)),
            [class::PANEL_BOX]
        );
        assert_eq!(
            names(panel(PanelGround::Sunk, true, false)),
            [class::PANEL_BOX, class::LINE]
        );
        assert_eq!(
            names(panel(PanelGround::Tint, true, false)),
            [class::PANEL_BOX, class::LINE, class::TINT]
        );
        assert_eq!(
            names(panel(PanelGround::Bare, false, true)),
            [class::PANEL_BOX, class::BARE, class::CENTER]
        );
    }

    #[test]
    fn a_panel_reads_its_ring_type_and_tint_from_tokens() {
        let css = crate::STYLESHEET;
        let from = css.find(".plinth-panel-box {").expect("the rule exists");
        let rule = &css[from..from + css[from..].find('}').expect("the rule ends")];
        assert!(rule.contains("border: var(--space-px) solid var(--panel-border)"));
        assert!(rule.contains("font-size: var(--panel-text)"));
        assert!(rule.contains("margin-block-end: var(--panel-space-after)"));
        let from = css
            .find(".plinth-panel-box.plinth-tint {")
            .expect("the rule exists");
        let rule = &css[from..from + css[from..].find('}').expect("the rule ends")];
        assert!(rule.contains("border-radius: var(--panel-tint-radius)"));
        assert!(rule.contains("color: var(--panel-tint-text)"));
        assert!(rule.contains("margin-block-end: var(--panel-tint-space-after)"));
        assert!(css.contains(".plinth-panel-box.plinth-small {"));
    }
}
