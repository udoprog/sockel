use yew::prelude::*;

use crate::class;

use sockel::{
    Button, LoadState, Loadable, Ratio, RowList, RowListItem, Skeleton, SkeletonShape, Spinner,
    skeleton,
};

use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn SpinnerPage() -> Html {
    let label = use_state(|| true);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Toggle label="label" value={*label} on_change={bind(&label)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let words = text.of("Saving your changes");
    let code = snippet(&[], El::new("Spinner").text_if(*label, "label", "Saving"));

    html! {
        <Demo {controls} {code}>
            <p>
                <Spinner label={label.then_some(AttrValue::Static("Saving"))} />
                { format!(" {words}") }
            </p>
            <Button busy=true>{ "Save" }</Button>
        </Demo>
    }
}

const SHAPES: &[(&str, SkeletonShape)] = &[
    ("Row", SkeletonShape::Row),
    ("Line", SkeletonShape::Line),
    ("Block", SkeletonShape::Block),
    ("Poster", SkeletonShape::Image(Ratio::Poster)),
    ("Screenshot", SkeletonShape::Image(Ratio::Screenshot)),
    ("Banner", SkeletonShape::Image(Ratio::Banner)),
    ("Square", SkeletonShape::Image(Ratio::Square)),
    (
        "Explicit 4 / 3",
        SkeletonShape::Image(Ratio::Explicit(4, 3)),
    ),
];

const COUNT: &[(&str, usize)] = &[("3", 3), ("1", 1), ("5", 5), ("8", 8)];

const WIDTHS: &[(&str, &str)] = &[("none", ""), ("14rem", "14rem"), ("50%", "50%")];

#[function_component]
pub fn SkeletonPage() -> Html {
    let shape = use_state(|| SkeletonShape::Row);
    let count = use_state(|| 3);
    let width = use_state(|| "");
    let compare = use_state(|| true);

    let controls = html! {
        <>
            <Select<SkeletonShape> label="shape" options={SHAPES} value={*shape} on_change={bind(&shape)} />
            <Select<usize> label="count" options={COUNT} value={*count} on_change={bind(&count)} />
            <Select<&'static str> label="width" options={WIDTHS} value={*width} on_change={bind(&width)} />
            <Toggle label="RowList beside" value={*compare} on_change={bind(&compare)} />
        </>
    };

    let name = SHAPES
        .iter()
        .find(|(_, s)| s == &*shape)
        .map_or("Block", |(name, _)| *name);
    let code = snippet(
        &[],
        El::new("Skeleton")
            .expr_if(
                *shape != SkeletonShape::Block,
                "shape",
                match *shape {
                    SkeletonShape::Image(ratio) => {
                        format!("SkeletonShape::Image(Ratio::{ratio:?})")
                    }
                    other => format!("SkeletonShape::{other:?}"),
                },
            )
            .expr_if(*count != 1, "count", count.to_string())
            .text_if(!width.is_empty(), "width", &width),
    );
    let shown_width = (!width.is_empty()).then(|| AttrValue::from(*width));

    html! {
        <Demo {controls} {code}>
            { shown(&format!("skeleton({name}, short)"), &skeleton(*shape, false)) }
            <Skeleton shape={*shape} count={*count} width={shown_width} />
            if *compare {
                <RowList divided=true>
                    { for (1..=*count).map(|n| html! {
                        <RowListItem name={html! { { format!("Row {n}") } }} />
                    }) }
                </RowList>
            }
        </Demo>
    }
}

const STATES: &[(&str, LoadState)] = &[
    ("Ready", LoadState::Ready),
    ("Loading", LoadState::Loading),
    ("Empty", LoadState::Empty),
    ("Error", LoadState::Error),
];

const DELAYS: &[(&str, u32)] = &[("250 ms", 250), ("0 (at once)", 0), ("1000 ms", 1000)];

#[function_component]
pub fn LoadablePage() -> Html {
    let state = use_state(|| LoadState::Ready);
    let rows = use_state(|| 3);
    let delay = use_state(|| 250);
    let custom = use_state(|| false);
    let message = use_state(|| Text::Short);
    let retry = use_state(|| true);
    let retrying = use_state(|| false);
    let retries = use_state(|| 0);

    let controls = html! {
        <>
            <Select<LoadState> label="state" options={STATES} value={*state} on_change={bind(&state)} />
            <Select<usize> label="rows" options={COUNT} value={*rows} on_change={bind(&rows)} />
            <Select<u32> label="empty_delay" options={DELAYS} value={*delay} on_change={bind(&delay)} />
            <Toggle label="placeholder" value={*custom} on_change={bind(&custom)} />
            <Select<Text> label="error" options={Text::OPTIONS} value={*message} on_change={bind(&message)} />
            <Toggle label="onretry" value={*retry} on_change={bind(&retry)} />
            <Toggle label="retrying" value={*retrying} on_change={bind(&retrying)} />
        </>
    };

    let onretry = {
        let retries = retries.clone();
        Callback::from(move |_: MouseEvent| retries.set(*retries + 1))
    };
    let error = message.of("Could not load the items");

    let state_name = STATES
        .iter()
        .find(|(_, s)| s == &*state)
        .map_or("Ready", |(name, _)| *name);
    let code = snippet(
        &[],
        El::new("Loadable")
            .expr("state", format!("LoadState::{state_name}"))
            .expr_if(*rows != 3, "rows", rows.to_string())
            .expr_if(*delay != 250, "empty_delay", delay.to_string())
            .expr_if(
                *custom,
                "placeholder",
                "html! { <Skeleton shape={SkeletonShape::Line} count=4 /> }",
            )
            .text_if(
                !error.is_empty() && *message != Text::Short,
                "error",
                &error,
            )
            .expr_if(*retry, "onretry", "on_retry")
            .flag("retrying", *retrying)
            .line("<RowList divided=true>{ for items }</RowList>"),
    );

    let placeholder = (*custom).then(|| html! { <Skeleton shape={SkeletonShape::Line} count=4 /> });

    html! {
        <Demo {controls} {code}>
            <Loadable state={*state} rows={*rows} empty_delay={*delay} {placeholder}
                error={error.clone()} onretry={retry.then_some(onretry)} retrying={*retrying}>
                <RowList divided=true>
                    { for (1..=*rows).map(|n| html! {
                        <RowListItem name={html! { { format!("Row {n}") } }} />
                    }) }
                </RowList>
            </Loadable>
            <span class={class::CONTROL_LABEL}>{ format!("Retried {} times", *retries) }</span>
        </Demo>
    }
}
