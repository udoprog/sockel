//! Stack, Cluster, Toolbar and Card (`style/_layout.scss`).

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
            Gap::None => Some("none"),
            Gap::Hair => Some("hair"),
            Gap::Snug => Some("snug"),
            Gap::Tight => Some("tight"),
            Gap::Medium => Some("medium"),
            Gap::Loose => Some("loose"),
            Gap::Wide => Some("wide"),
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
            Justify::Center => Some("center"),
            Justify::Between => Some("between"),
            Justify::End => Some("end"),
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
            Align::Top => Some("top"),
            Align::Baseline => Some("baseline"),
            Align::Bottom => Some("bottom"),
            Align::Stretch => Some("stretch"),
        }
    }
}

/// A stack's classes.
pub fn stack(gap: Gap) -> Classes {
    stack_with(gap, false)
}

/// A stack's classes; `flush` zeroes the block margins of its direct children.
pub fn stack_with(gap: Gap, flush: bool) -> Classes {
    classes!("stack", gap.class(), flush.then_some("flush"))
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
        "cluster",
        gap.class(),
        justify.class(),
        align.class(),
        (!wrap).then_some("nowrap"),
        inline.then_some("inline"),
        lines.then_some("lines"),
        flush.then_some("flush")
    )
}

/// A toolbar's classes; `boxed` draws it as a bar of its own.
pub fn toolbar(boxed: bool) -> Classes {
    classes!("toolbar", boxed.then_some("boxed"))
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
        Inset::Compact => Some("compact"),
        Inset::Flush => Some("flush"),
    };

    classes!("card", inset, link.then_some("link"))
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

#[function_component]
pub fn Stack(props: &StackProps) -> Html {
    html! {
        <@{props.tag} class={classes!(stack_with(props.gap, props.flush), props.class.clone())}
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
        <@{props.tag} class={classes!(classes, props.class.clone())}
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

/// Among a toolbar's controls, pushes those after it to the toolbar's end.
#[function_component]
pub fn ToolbarSpacer() -> Html {
    html! { <span class="toolbar-spacer" aria-hidden="true" /> }
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
        <@{props.tag} class={classes!(card(props.inset, false), props.class.clone())}
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
        assert_eq!(names(stack(Gap::Normal)), ["stack"]);
        assert_eq!(names(stack(Gap::None)), ["stack", "none"]);
        assert_eq!(names(stack(Gap::Hair)), ["stack", "hair"]);
        assert_eq!(names(stack(Gap::Snug)), ["stack", "snug"]);
        assert_eq!(names(stack(Gap::Tight)), ["stack", "tight"]);
        assert_eq!(names(stack(Gap::Medium)), ["stack", "medium"]);
        assert_eq!(names(stack(Gap::Loose)), ["stack", "loose"]);
        assert_eq!(names(stack(Gap::Wide)), ["stack", "wide"]);
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
            ["cluster"]
        );
        assert_eq!(
            names(cluster(
                Gap::Tight,
                Justify::Between,
                Align::Top,
                false,
                false
            )),
            ["cluster", "tight", "between", "top", "nowrap"]
        );
        assert_eq!(
            names(cluster(
                Gap::Snug,
                Justify::End,
                Align::Baseline,
                true,
                true
            )),
            ["cluster", "snug", "end", "baseline", "inline"]
        );
        assert_eq!(
            names(cluster(
                Gap::Hair,
                Justify::Start,
                Align::Stretch,
                true,
                false
            )),
            ["cluster", "hair", "stretch"]
        );
    }

    #[test]
    fn a_stack_and_a_cluster_take_the_flush_and_lines_modifiers() {
        assert_eq!(
            names(stack_with(Gap::None, true)),
            ["stack", "none", "flush"]
        );
        assert_eq!(names(stack_with(Gap::Normal, false)), ["stack"]);
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
            ["cluster", "lines", "flush"]
        );
    }

    #[test]
    fn a_toolbar_and_a_card_take_their_modifiers() {
        assert_eq!(names(toolbar(false)), ["toolbar"]);
        assert_eq!(names(toolbar(true)), ["toolbar", "boxed"]);
        assert_eq!(
            names(classes!(toolbar(false), Gap::Tight.class())),
            ["toolbar", "tight"]
        );
        assert_eq!(
            names(classes!(toolbar(false), Gap::Normal.class())),
            ["toolbar"]
        );
        assert_eq!(names(card(Inset::Normal, false)), ["card"]);
        assert_eq!(
            names(card(Inset::Compact, true)),
            ["card", "compact", "link"]
        );
        assert_eq!(names(card(Inset::Flush, false)), ["card", "flush"]);
    }
}
