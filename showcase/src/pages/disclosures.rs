use yew::prelude::*;

use sockel::{Button, ButtonSize, Disclosure, HeadLevel, disclosure};

use crate::class;
use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const BODIES: &[(&str, usize)] = &[("3 lines", 3), ("1 line", 1), ("12 lines", 12)];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn DisclosurePage() -> Html {
    let title = use_state(|| Text::Short);
    let summary = use_state(|| Text::Short);
    let body = use_state(|| 3usize);
    let open = use_state(|| false);
    let actions = use_state(|| true);
    let disabled = use_state(|| false);
    let heading = use_state(|| false);
    let wrap = use_state(|| false);

    let controls = html! {
        <>
            <Select<Text> label="Title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<Text> label="Summary" options={Text::OPTIONS} value={*summary} on_change={bind(&summary)} />
            <Select<usize> label="Body" options={BODIES} value={*body} on_change={bind(&body)} />
            <Toggle label="open" value={*open} on_change={bind(&open)} />
            <Toggle label="actions" value={*actions} on_change={bind(&actions)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="heading" value={*heading} on_change={bind(&heading)} />
            <Toggle label="wrap" value={*wrap} on_change={bind(&wrap)} />
        </>
    };
    let title_text = title.of("Advanced");
    let summary_text = summary.of("3 rules, 1 off");
    let code = snippet(
        &[],
        El::new("Disclosure")
            .expr("title", format!("html! {{ {} }}", text_node(&title_text)))
            .expr_if(
                !summary_text.is_empty(),
                "summary",
                format!("Some(html! {{ {} }})", text_node(&summary_text)),
            )
            .expr_if(
                *actions,
                "actions",
                "html! { <Button>{ \"Edit\" }</Button> }",
            )
            .expr("open", "open")
            .expr("on_toggle", "on_toggle.clone()")
            .flag("disabled", *disabled)
            .expr_if(*heading, "level", "Some(HeadLevel::H3)")
            .expr_if(*wrap, "class", "classes!(sockel::class::WRAP)")
            .line("{ body }"),
    );

    html! {
        <Demo {controls} {code}>
            <Disclosure title={html! { { title_text } }}
                summary={(!summary_text.is_empty()).then(|| html! { { summary_text } })}
                actions={if *actions {
                    html! { <Button size={ButtonSize::Small}>{ "Edit" }</Button> }
                } else {
                    html! {}
                }}
                open={*open} on_toggle={bind(&open)} id="disclosure-body" disabled={*disabled}
                level={heading.then_some(HeadLevel::H3)}
                class={classes!(wrap.then_some(sockel::class::WRAP))}>
                { for (1..=*body).map(|n| html! { <p>{ format!("Line {n} of the section") }</p> }) }
            </Disclosure>
            { shown("disclosure(open)", &disclosure(*open)) }
        </Demo>
    }
}
