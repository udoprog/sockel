use yew::prelude::*;

use crate::class;

use plinth::LibraryToolbar;

use crate::code::{El, each, snippet};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

const PICKS: &[&str] = &["All", "Mine", "A much longer choice than the others"];

#[function_component]
pub fn LibraryToolbarPage() -> Html {
    let value = use_state(String::new);
    let label = use_state(|| Text::Short);
    let autofocus = use_state(|| false);
    let filters = use_state(|| 3);
    let picked = use_state(|| vec![0; 20]);

    let controls = html! {
        <>
            <Select<Text> label="label" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Select<usize> label="Filters" options={COUNTS} value={*filters} on_change={bind(&filters)} />
            <Toggle label="autofocus" value={*autofocus} on_change={bind(&autofocus)} />
        </>
    };

    let filter = |n: usize| {
        let on_click = {
            let picked = picked.clone();
            Callback::from(move |_: MouseEvent| {
                let mut next = (*picked).clone();
                next[n] = (next[n] + 1) % PICKS.len();
                picked.set(next);
            })
        };

        html! {
            <button type="button" class={classes!(class::BUTTON, class::FILTER)} onclick={on_click}
                title="Click to pick another: the field keeps its width">
                <span class={class::FILTER_TEXT}>{ format!("Filter {}: {}", n + 1, PICKS[picked[n]]) }</span>
            </button>
        }
    };

    let code = snippet(
        &[],
        El::new("LibraryToolbar")
            .expr("value", "query.clone()")
            .text("label", &label.of("Search the library"))
            .text("testid", "showcase-search")
            .expr("on_input", "on_input")
            .flag("autofocus", *autofocus)
            .line_if(
                *filters > 0,
                each("filters", "filter_button", *filters, "button"),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <LibraryToolbar value={(*value).clone()} label={label.of("Search the library")}
                testid="showcase-search" on_input={bind(&value)} autofocus={*autofocus}>
                { for (0..*filters).map(filter) }
            </LibraryToolbar>
            <p class={class::CONTROL_LABEL}>{ format!("Searching for {:?}", *value) }</p>
        </Demo>
    }
}
