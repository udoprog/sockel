// Not compiled: a rename to the `usage-` prefix that missed some literals.
fn link(name: &str, class: Option<&'static str>) -> Html {
    html! { <a class={classes!(class)} title={name}>{ name }</a> }
}

fn view(severity: Severity) -> Html {
    let (class, text) = match severity {
        Severity::Error => ("live", "usage-live-error"),
        Severity::Warning => ("usage-live", "warning"),
    };
    html! {
        <div class={classes!(class)} role="once">
            { link("once", Some("dead")) }
            <span kind="dead" class={classes!(class::ONCE, if wide { "once" } else { "usage-once" })}>{ text }</span>
        </div>
    }
}

fn matches(kind: &str) -> bool {
    kind == "dead" || matches!(kind, "live")
}

