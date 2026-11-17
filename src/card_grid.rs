//! CardGrid (`style/_card-grid.scss`).

use crate::class;
use yew::prelude::*;

/// How wide a card-grid column starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridSize {
    /// `--card-grid-min`.
    #[default]
    Normal,
    /// `--card-grid-wide`: cards that carry a few lines of text.
    Wide,
    /// `--card-grid-narrow`: small cards, two abreast on a phone.
    Narrow,
}

#[derive(Properties, PartialEq)]
pub struct CardGridProps {
    #[prop_or_default]
    pub size: GridSize,
    #[prop_or("ul")]
    pub tag: &'static str,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The cards.
    #[prop_or_default]
    pub children: Html,
}

/// A card grid's classes.
pub fn card_grid(size: GridSize) -> Classes {
    let size = match size {
        GridSize::Normal => None,
        GridSize::Wide => Some(class::WIDE),
        GridSize::Narrow => Some(class::NARROW),
    };

    classes!(class::CARD_GRID, size)
}

/// A grid of cards, as many columns as fit, each at least the size's width
/// and all sharing the row's room; a card is as tall as its row.
#[function_component]
pub fn CardGrid(props: &CardGridProps) -> Html {
    html! {
        <@{props.tag} class={classes!(card_grid(props.size), props.class.clone())}
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
    fn size_is_a_modifier() {
        assert_eq!(names(card_grid(GridSize::Normal)), [class::CARD_GRID]);
        assert_eq!(
            names(card_grid(GridSize::Wide)),
            [class::CARD_GRID, class::WIDE]
        );
        assert_eq!(
            names(card_grid(GridSize::Narrow)),
            [class::CARD_GRID, class::NARROW]
        );
    }
}
