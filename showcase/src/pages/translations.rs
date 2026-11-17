use yew::prelude::*;

use crate::class;
use crate::code::CodeCard;

use plinth::{Card, Cluster, Gap, Inset, Pager, Stack, Stepper};

/// The app's setup: a phrasebook with plinth's catalog after its own.
const SETUP: &str = r#"use phrasebook::{Catalog, Lang, Phrasebook, Phrases, Source, browser};

const SV_SE: Lang = Lang::new("sv-SE");

static APP: Catalog = Catalog::new("app", &[
    Source::new(Lang::EN_US, include_str!("../locales/en-US.ftl")),
    Source::new(SV_SE, include_str!("../locales/sv-SE.ftl")),
]);

let book = Phrasebook::builder()
    .language(Lang::EN_US, "English")
    .language(SV_SE, "Svenska")
    .catalog(&APP)
    .catalog(&plinth::phrasebookcatalog01::L10N)
    .build();
let phrases = use_state(|| browser::initial(&book));

html! {
    <ContextProvider<Phrases> context={(*phrases).clone()}>
        <App />
    </ContextProvider<Phrases>>
}"#;

/// How an app gives plinth its language, with components in the language
/// picked in the navigation.
#[function_component]
pub fn TranslationsPage() -> Html {
    let value = use_state(|| 3);
    let page = use_state(|| 0);
    let on_change = {
        let value = value.clone();
        Callback::from(move |next| value.set(next))
    };
    let on_page = {
        let page = page.clone();
        Callback::from(move |next| page.set(next))
    };

    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "plinth's words (Close, Cancel, the pager's and the stepper's names, the reorder \
                   announcements) are messages in plinth::phrasebookcatalog01::L10N, a \
                   phrasebook-catalog 0.1 catalog with en-US and sv-SE, every id under plinth-. \
                   Each component looks them up in the Phrases the app provides with \
                   ContextProvider<Phrases>; with none, or one without plinth's catalog, it shows \
                   the catalog's English and phrasebook warns once." }
            </p>
            <p class={class::PROSE}>
                { "An app adds L10N after its own catalog, so a plinth- id in its own .ftl wins: \
                   that is how it translates plinth into a language plinth lacks, or rewords one \
                   of its messages, and phrasebook::check::parity_with checks it has them all. \
                   A label prop on one use (less_label, close_label, ...) wins over both. The \
                   provider must be phrasebook 0.2's Phrases, the version plinth reads." }
            </p>
            <p class={class::PROSE}>
                { "The showcase does this with no messages of its own: pick Svenska under \
                   Language in the navigation and every component's words change; the page \
                   texts are the showcase's and stay English." }
            </p>
            <Card inset={Inset::Compact}>
                <Cluster gap={Gap::Loose}>
                    <Stepper value={*value} min={0} max={9} label="Copies" {on_change} />
                    <Pager page={*page} pages={12} {on_page} />
                </Cluster>
            </Card>
            <CodeCard code={SETUP} />
        </Stack>
    }
}
