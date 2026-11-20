//! ChipSelect: removable chips of what is picked, over a [`SearchSelect`]
//! of what is not.

use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{Chip, ChipList, Gap, InputSize, SearchOption, SearchSelect, stack};

/// A chip select's classes: a tight stack of the chips and the search.
pub fn chip_select() -> Classes {
    stack(Gap::Tight)
}

/// The picks after `picked` joins `value`: last when `ordered`, else in the
/// options' order.
fn added<T: Clone + PartialEq>(
    options: &[SearchOption<T>],
    value: &[T],
    picked: T,
    ordered: bool,
) -> Vec<T> {
    let mut next = value.to_vec();
    next.push(picked);
    if !ordered {
        let rank = |v: &T| options.iter().position(|o| &o.value == v);
        next.sort_by_key(rank);
    }
    next
}

#[derive(Properties, PartialEq)]
pub struct ChipSelectProps<T: Clone + PartialEq + 'static> {
    /// Everything that may be picked.
    pub options: Vec<SearchOption<T>>,
    /// The picks. A pick that is none of the options shows no chip but is
    /// kept.
    pub value: Vec<T>,
    pub on_change: Callback<Vec<T>>,
    /// The chips' group name, and the search's placeholder and name.
    pub label: AttrValue,
    /// The picks keep the order they were picked in; else they follow the
    /// options'.
    #[prop_or_default]
    pub ordered: bool,
    /// The fewest picks: at it, no chip can be removed.
    #[prop_or_default]
    pub min: usize,
    /// Said when no option is left to pick.
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A [`ChipList`] of the picked options, each removable by its cross, and
/// a [`SearchSelect`] of the others. The chip row keeps its height and the
/// search its place as picks come and go. Backspace in the empty search
/// removes the last chip.
#[function_component]
pub fn ChipSelect<T>(props: &ChipSelectProps<T>) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let options = &props.options;
    let shown: Vec<&SearchOption<T>> = if props.ordered {
        props
            .value
            .iter()
            .filter_map(|v| options.iter().find(|o| &o.value == v))
            .collect()
    } else {
        options
            .iter()
            .filter(|o| props.value.contains(&o.value))
            .collect()
    };
    let removable = !props.disabled && props.value.len() > props.min;
    let remove = |value: &T| {
        let (on_change, value) = (props.on_change.clone(), value.clone());
        let rest: Vec<T> = props
            .value
            .iter()
            .filter(|v| **v != value)
            .cloned()
            .collect();
        Callback::from(move |_: MouseEvent| on_change.emit(rest.clone()))
    };

    let left: Vec<SearchOption<T>> = options
        .iter()
        .filter(|o| !props.value.contains(&o.value))
        .cloned()
        .collect();
    let on_pick = {
        let (options, value, on_change) = (
            props.options.clone(),
            props.value.clone(),
            props.on_change.clone(),
        );
        let ordered = props.ordered;
        Callback::from(move |picked: T| on_change.emit(added(&options, &value, picked, ordered)))
    };

    let onkeydown = {
        let last = shown.last().map(|o| o.value.clone());
        let (value, on_change) = (props.value.clone(), props.on_change.clone());
        Callback::from(move |e: KeyboardEvent| {
            let empty = e
                .target_dyn_into::<HtmlInputElement>()
                .is_some_and(|input| input.value().is_empty());
            if e.key() == "Backspace"
                && removable
                && empty
                && let Some(last) = &last
            {
                on_change.emit(value.iter().filter(|v| *v != last).cloned().collect());
            }
        })
    };

    html! {
        <div class={chip_select()} {onkeydown}>
            <ChipList label={props.label.clone()}>
                { for shown.iter().map(|o| html! {
                    <Chip on_remove={removable.then(|| remove(&o.value))}>{ o.label.clone() }</Chip>
                }) }
            </ChipList>
            <SearchSelect<T>
                options={left}
                label={props.label.clone()}
                on_change={on_pick}
                empty_label={props.empty_label.clone()}
                id={props.id.clone()}
                size={props.size}
                invalid={props.invalid}
                disabled={props.disabled}
                fill={true}
                testid={props.testid.clone()} />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class;

    fn fruit() -> Vec<SearchOption<u32>> {
        vec![
            SearchOption::new(0, "Apple"),
            SearchOption::new(1, "Pear"),
            SearchOption::new(2, "Plum"),
        ]
    }

    fn render(props: ChipSelectProps<u32>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<ChipSelect<u32>>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(value: Vec<u32>, ordered: bool, min: usize) -> ChipSelectProps<u32> {
        yew::props!(ChipSelectProps<u32> {
            options: fruit(),
            value,
            on_change: Callback::noop(),
            label: "Fruit",
            ordered,
            min,
        })
    }

    #[test]
    fn a_pick_goes_last_when_ordered_else_by_the_options() {
        assert_eq!(added(&fruit(), &[2, 1], 0, true), [2, 1, 0]);
        assert_eq!(added(&fruit(), &[2, 1], 0, false), [0, 1, 2]);
    }

    #[test]
    fn the_chips_are_a_stack_with_a_cross_each_above_the_minimum() {
        let names: Vec<_> = chip_select().into_iter().map(|c| c.to_string()).collect();
        assert_eq!(names, [class::STACK, class::TIGHT]);

        let html = render(props(vec![2, 0], true, 0));
        assert!(html.contains(class::CHIP_LIST), "{html}");
        assert_eq!(html.matches(class::CHIP_REMOVE).count(), 2, "{html}");
        assert!(html.find("Plum") < html.find("Apple"), "{html}");

        let html = render(props(vec![2, 0], false, 0));
        assert!(html.find("Apple") < html.find("Plum"), "{html}");

        let html = render(props(vec![1], true, 1));
        assert!(html.contains("Pear"), "{html}");
        assert!(!html.contains(class::CHIP_REMOVE), "{html}");
    }
}
