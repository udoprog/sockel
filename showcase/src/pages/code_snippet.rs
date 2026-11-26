//! The CodeSnippet page: one line of the CodeEditor page's toy language,
//! highlighted by its highlighter, with marks, alone, in an ErrorBox or
//! inline.

use std::rc::Rc;

use yew::prelude::*;

use sockel::{CodeSnippet, CodeSpan, Diagnostic, ErrorBox, Severity, code_snippet};

use super::code_editor::highlight;
use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Toggle, bind};

const LINE: &str = "let total = round(price * rate(\"SE\")) + shiping + handling_for_the_selected_region_and_carrier(order)";

/// The marks on [`LINE`], as a checker would report them.
fn marks() -> Vec<Diagnostic> {
    let at = |word: &str| {
        let start = LINE.find(word).unwrap_or(0);
        start..start + word.len()
    };

    vec![
        Diagnostic::new(at("shiping"), Severity::Error, "unknown name `shiping`"),
        Diagnostic::new(
            at("rate(\"SE\")"),
            Severity::Info,
            "looked up once per order",
        ),
        Diagnostic::new(
            at("handling_for_the_selected_region_and_carrier"),
            Severity::Warning,
            "deprecated: use `handling(order)`",
        ),
    ]
}

#[function_component]
pub fn CodeSnippetPage() -> Html {
    let spans = use_state(|| true);
    let marked = use_state(|| true);
    let inline = use_state(|| false);
    let in_error = use_state(|| true);

    let controls = html! {
        <>
            <Toggle label="spans" value={*spans} on_change={bind(&spans)} />
            <Toggle label="marks" value={*marked} on_change={bind(&marked)} />
            <Toggle label="inline" value={*inline} on_change={bind(&inline)} />
            <Toggle label="in ErrorBox" value={*in_error} on_change={bind(&in_error)} />
        </>
    };

    let el = El::new("CodeSnippet")
        .expr("code", "line")
        .expr_if(*spans, "spans", "highlight(line)")
        .expr_if(*marked, "marks", "check(line)")
        .flag("inline", *inline);
    let code = if *in_error {
        snippet(
            &[],
            El::new("ErrorBox")
                .text("title", "Could not check the expression")
                .line("\"unknown name `shiping`\"")
                .child(el),
        )
    } else {
        snippet(&[], el)
    };

    let highlighted = if *spans {
        Rc::<[CodeSpan]>::from(highlight(LINE).0)
    } else {
        Rc::from([])
    };
    let found = if *marked {
        Rc::<[Diagnostic]>::from(marks())
    } else {
        Rc::from([])
    };
    let shown = html! {
        <CodeSnippet code={LINE} spans={highlighted} marks={found} inline={*inline}
            testid="code-snippet" />
    };

    html! {
        <Demo {controls} {code}>
            if *in_error {
                <ErrorBox title="Could not check the expression">
                    { if *inline { "unknown name `shiping` in " } else { "unknown name `shiping`" } }
                    { shown }
                </ErrorBox>
            } else {
                { shown }
            }
            <span class={class::CONTROL_LABEL}>
                { format!("code_snippet({}) → \"{}\"", *inline, code_snippet(*inline).to_string()) }
            </span>
        </Demo>
    }
}
