//! Stack, Cluster, Toolbar and Card (`style/_layout.scss`).

use crate::class;
use yew::prelude::*;

/// The space between a stack's or a cluster's items: one scale for both, a
/// step of the spacing tokens each, so `.tight` is the same gap in a row
/// and a column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gap {
    /// The primitive's own: a stack's `--space-3`, a cluster's `--space-2`.
    #[default]
    Normal,
    /// No gap.
    None,
    /// `--space-0` (2px): the buttons of a tight group, a board toolbar's.
    Hair,
    /// `--space-1`: an icon and its text, chips in a row.
    Snug,
    /// `--space-2`: a list of small things.
    Tight,
    /// `--space-3`.
    Medium,
    /// `--space-4`: a page's or a section's blocks, groups of controls.
    Loose,
    /// `--space-6`: sections apart.
    Wide,
}

impl Gap {
    fn class(self) -> Option<&'static str> {
        match self {
            Gap::Normal => None,
            Gap::None => Some(class::NONE),
            Gap::Hair => Some(class::HAIR),
            Gap::Snug => Some(class::SNUG),
            Gap::Tight => Some(class::TIGHT),
            Gap::Medium => Some(class::MEDIUM),
            Gap::Loose => Some(class::LOOSE),
            Gap::Wide => Some(class::WIDE),
        }
    }
}

/// Where a cluster's items sit along its row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    #[default]
    Start,
    /// Centred in the row.
    Center,
    /// The first and last at the ends, the rest spread between.
    Between,
    /// At the row's end: a row of actions.
    End,
}

impl Justify {
    fn class(self) -> Option<&'static str> {
        match self {
            Justify::Start => None,
            Justify::Center => Some(class::CENTER),
            Justify::Between => Some(class::BETWEEN),
            Justify::End => Some(class::END),
        }
    }
}

/// How a cluster's items line up across its row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Center,
    /// Their tops level: items of different heights that wrap text.
    Top,
    /// Their text's baselines level: a label beside larger text.
    Baseline,
    /// Their bottoms level: fields of different heights beside each other.
    Bottom,
    /// Each as tall as the row: a button beside a text field.
    Stretch,
}

impl Align {
    fn class(self) -> Option<&'static str> {
        match self {
            Align::Center => None,
            Align::Top => Some(class::TOP),
            Align::Baseline => Some(class::BASELINE),
            Align::Bottom => Some(class::BOTTOM),
            Align::Stretch => Some(class::STRETCH),
        }
    }
}

/// A stack's classes.
pub fn stack(gap: Gap) -> Classes {
    stack_with(gap, false)
}

/// A stack's classes; `flush` zeroes the block margins of its direct children.
pub fn stack_with(gap: Gap, flush: bool) -> Classes {
    classes!(class::STACK, gap.class(), flush.then_some(class::FLUSH))
}

/// A cluster's classes; `wrap` false keeps it on one line, `inline` sets
/// it in the text's flow (an icon and its label).
pub fn cluster(gap: Gap, justify: Justify, align: Align, wrap: bool, inline: bool) -> Classes {
    cluster_with(gap, justify, align, wrap, inline, false, false)
}

/// A cluster's classes with the opt-in modifiers: `lines` draws wrapped lines
/// closer than the column gap, `flush` zeroes the block margins of its
/// direct children.
pub fn cluster_with(
    gap: Gap,
    justify: Justify,
    align: Align,
    wrap: bool,
    inline: bool,
    lines: bool,
    flush: bool,
) -> Classes {
    classes!(
        class::CLUSTER,
        gap.class(),
        justify.class(),
        align.class(),
        (!wrap).then_some(class::NOWRAP),
        inline.then_some(class::INLINE),
        lines.then_some(class::LINES),
        flush.then_some(class::FLUSH)
    )
}

/// A toolbar's classes; `boxed` draws it as a bar of its own.
pub fn toolbar(boxed: bool) -> Classes {
    classes!(class::TOOLBAR, boxed.then_some(class::BOXED))
}

/// How much room a card leaves inside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Inset {
    #[default]
    Normal,
    /// Less, for a dense card in a grid or list.
    Compact,
    /// None: what it holds runs to its edges (a list, a table, an image).
    Flush,
}

/// A card's classes; `link` for a card that is itself a link or button.
pub fn card(inset: Inset, link: bool) -> Classes {
    let inset = match inset {
        Inset::Normal => None,
        Inset::Compact => Some(class::COMPACT),
        Inset::Flush => Some(class::FLUSH),
    };

    classes!(class::CARD, inset, link.then_some(class::LINK))
}

#[derive(Properties, PartialEq)]
pub struct StackProps {
    #[prop_or_default]
    pub gap: Gap,
    /// Zeroes the block margins of the direct children.
    #[prop_or_default]
    pub flush: bool,
    #[prop_or("div")]
    pub tag: &'static str,
    /// The screen's own class, for what only it needs.
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A list tag drops the browser's bullets, margin and padding.
fn list_reset(tag: &str) -> Option<&'static str> {
    matches!(tag, "ul" | "ol").then_some(class::PLAIN_LIST)
}

#[function_component]
pub fn Stack(props: &StackProps) -> Html {
    html! {
        <@{props.tag} class={classes!(stack_with(props.gap, props.flush), list_reset(props.tag), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </@>
    }
}

#[derive(Properties, PartialEq)]
pub struct ClusterProps {
    #[prop_or_default]
    pub gap: Gap,
    #[prop_or_default]
    pub justify: Justify,
    #[prop_or_default]
    pub align: Align,
    /// Keeps the row on one line.
    #[prop_or_default]
    pub nowrap: bool,
    /// In the text's flow, as wide as what it holds.
    #[prop_or_default]
    pub inline: bool,
    /// Wrapped lines sit closer (`--space-1`) than the column gap.
    #[prop_or_default]
    pub lines: bool,
    /// Zeroes the block margins of the direct children.
    #[prop_or_default]
    pub flush: bool,
    #[prop_or("div")]
    pub tag: &'static str,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn Cluster(props: &ClusterProps) -> Html {
    let classes = cluster_with(
        props.gap,
        props.justify,
        props.align,
        !props.nowrap,
        props.inline,
        props.lines,
        props.flush,
    );

    html! {
        <@{props.tag} class={classes!(classes, list_reset(props.tag), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </@>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    /// A bar of its own, bordered on the surface.
    #[prop_or_default]
    pub boxed: bool,
    /// The space between its controls; the normal gap is `--space-1`.
    #[prop_or_default]
    pub gap: Gap,
    /// What the toolbar is for, for assistive technology.
    pub label: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// One line of controls; a [`ToolbarSpacer`] among them pushes the rest to
/// its end.
#[function_component]
pub fn Toolbar(props: &ToolbarProps) -> Html {
    html! {
        <div role="toolbar" aria-label={props.label.clone()}
            class={classes!(toolbar(props.boxed), props.gap.class(), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarSpacerProps {
    #[prop_or_default]
    pub class: Classes,
}

/// Among a toolbar's controls, pushes those after it to the toolbar's end.
#[function_component]
pub fn ToolbarSpacer(props: &ToolbarSpacerProps) -> Html {
    html! {
        <span class={classes!(class::TOOLBAR_SPACER, props.class.clone())} aria-hidden="true" />
    }
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    #[prop_or_default]
    pub inset: Inset,
    #[prop_or("div")]
    pub tag: &'static str,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A card holding content; a card that is a link takes [`card`]'s classes
/// on its `Link`.
#[function_component]
pub fn Card(props: &CardProps) -> Html {
    html! {
        <@{props.tag} class={classes!(card(props.inset, false), list_reset(props.tag), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </@>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_stack_is_its_class_and_its_gap() {
        assert_eq!(names(stack(Gap::Normal)), [class::STACK]);
        assert_eq!(names(stack(Gap::None)), [class::STACK, class::NONE]);
        assert_eq!(names(stack(Gap::Hair)), [class::STACK, class::HAIR]);
        assert_eq!(names(stack(Gap::Snug)), [class::STACK, class::SNUG]);
        assert_eq!(names(stack(Gap::Tight)), [class::STACK, class::TIGHT]);
        assert_eq!(names(stack(Gap::Medium)), [class::STACK, class::MEDIUM]);
        assert_eq!(names(stack(Gap::Loose)), [class::STACK, class::LOOSE]);
        assert_eq!(names(stack(Gap::Wide)), [class::STACK, class::WIDE]);
    }

    #[test]
    fn a_cluster_names_only_what_differs_from_its_defaults() {
        assert_eq!(
            names(cluster(
                Gap::Normal,
                Justify::Start,
                Align::Center,
                true,
                false
            )),
            [class::CLUSTER]
        );
        assert_eq!(
            names(cluster(
                Gap::Tight,
                Justify::Between,
                Align::Top,
                false,
                false
            )),
            [
                class::CLUSTER,
                class::TIGHT,
                class::BETWEEN,
                class::TOP,
                class::NOWRAP
            ]
        );
        assert_eq!(
            names(cluster(
                Gap::Snug,
                Justify::End,
                Align::Baseline,
                true,
                true
            )),
            [
                class::CLUSTER,
                class::SNUG,
                class::END,
                class::BASELINE,
                class::INLINE
            ]
        );
        assert_eq!(
            names(cluster(
                Gap::Hair,
                Justify::Start,
                Align::Stretch,
                true,
                false
            )),
            [class::CLUSTER, class::HAIR, class::STRETCH]
        );
    }

    #[test]
    fn a_stack_and_a_cluster_take_the_flush_and_lines_modifiers() {
        assert_eq!(
            names(stack_with(Gap::None, true)),
            [class::STACK, class::NONE, class::FLUSH]
        );
        assert_eq!(names(stack_with(Gap::Normal, false)), [class::STACK]);
        assert_eq!(
            names(cluster_with(
                Gap::Normal,
                Justify::Start,
                Align::Center,
                true,
                false,
                true,
                true
            )),
            [class::CLUSTER, class::LINES, class::FLUSH]
        );
    }

    #[test]
    fn a_list_tag_takes_the_plain_list_class() {
        assert_eq!(list_reset("ul"), Some(class::PLAIN_LIST));
        assert_eq!(list_reset("ol"), Some(class::PLAIN_LIST));
        assert_eq!(list_reset("div"), None);
        assert_eq!(list_reset("section"), None);
    }

    #[test]
    fn a_toolbar_and_a_card_take_their_modifiers() {
        assert_eq!(names(toolbar(false)), [class::TOOLBAR]);
        assert_eq!(names(toolbar(true)), [class::TOOLBAR, class::BOXED]);
        assert_eq!(
            names(classes!(toolbar(false), Gap::Tight.class())),
            [class::TOOLBAR, class::TIGHT]
        );
        assert_eq!(
            names(classes!(toolbar(false), Gap::Normal.class())),
            [class::TOOLBAR]
        );
        assert_eq!(names(card(Inset::Normal, false)), [class::CARD]);
        assert_eq!(
            names(card(Inset::Compact, true)),
            [class::CARD, class::COMPACT, class::LINK]
        );
        assert_eq!(
            names(card(Inset::Flush, false)),
            [class::CARD, class::FLUSH]
        );
    }
}
