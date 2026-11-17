use yew::prelude::*;

use plinth::{Card, Gap, Inset, Stack};

/// What `STYLESHEET` and `Styles` are; the showcase itself still loads
/// plinth's Sass through trunk (its own stylesheet `@use`s `style/`).
#[function_component]
pub fn StylesPage() -> Html {
    let css = plinth::STYLESHEET;
    let rules = css.matches('}').count();

    html! {
        <Stack gap={Gap::Loose}>
            <p class="prose">
                { "STYLESHEET is plinth's style/ compiled with its defaults at build time; \
                   mounting <Styles /> once adds it to <head>, however many times it is mounted. \
                   The showcase does not use them yet: it compiles plinth's Sass through trunk, \
                   with its own colour tokens, until card 3dd69aec moves it over." }
            </p>
            <Card inset={Inset::Compact}>
                <Stack gap={Gap::Snug}>
                    <span class="control-label">{ format!("STYLESHEET: {} bytes, about {rules} blocks", css.len()) }</span>
                    <details>
                        <summary>{ "Show the stylesheet" }</summary>
                        <pre class="stylesheet">{ css }</pre>
                    </details>
                </Stack>
            </Card>
        </Stack>
    }
}
