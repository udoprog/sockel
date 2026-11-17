use yew::prelude::*;

use crate::class;

use plinth::{Card, Gap, Inset, Stack};

/// What `STYLESHEET` and `Styles` are, and how the showcase itself is styled.
#[function_component]
pub fn StylesPage() -> Html {
    let css = plinth::STYLESHEET;
    let rules = css.matches('}').count();

    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "STYLESHEET is plinth's style/ compiled with its defaults at build time; \
                   mounting <Styles /> adds it to <head> as <style id=\"plinth-styles\">, once. \
                   An app with styles of its own builds them with plinth-build after plinth's, \
                   into one stylesheet, and mounts <Styles css={STYLESHEET} /> with it instead, \
                   as the showcase does: its colours and controls are its own style/, \
                   classes prefixed showcase-." }
            </p>
            <Card inset={Inset::Compact}>
                <Stack gap={Gap::Snug}>
                    <span class={class::CONTROL_LABEL}>{ format!("STYLESHEET: {} bytes, about {rules} blocks", css.len()) }</span>
                    <details>
                        <summary>{ "Show the stylesheet" }</summary>
                        <pre class={class::STYLESHEET}>{ css }</pre>
                    </details>
                </Stack>
            </Card>
        </Stack>
    }
}
