//! sockel's messages as a [phrasebook-catalog] 0.1 catalog, [`L10N`].
//!
//! [phrasebook-catalog]: https://docs.rs/phrasebook-catalog

use phrasebook02::Phrases;
use phrasebookcatalog01::{Catalog, Lang, Source};
use yew::AttrValue;

const SV_SE: Lang = Lang::new("sv-SE");

/// sockel's messages, in English and Swedish, with every id under the
/// `sockel-` prefix.
///
/// sockel's components look their words up in the `Phrases` the app provides
/// with `ContextProvider<Phrases>`, and fall back to these messages for any
/// id the app's phrasebook lacks, or when nothing is provided. A label prop
/// (`close_label`, `save_label`, ...) given on a use wins over both. An app
/// adds this catalog after its own, so its own `sockel-*` translations win:
///
/// ```ignore
/// use phrasebook::{Catalog, Lang, Phrasebook, Source};
///
/// const SV_SE: Lang = Lang::new("sv-SE");
///
/// static APP: Catalog = Catalog::new("app", &[
///     Source::new(Lang::EN_US, include_str!("../locales/en-US.ftl")),
///     Source::new(SV_SE, include_str!("../locales/sv-SE.ftl")),
/// ]);
///
/// let book = Phrasebook::builder()
///     .language(Lang::EN_US, "English")
///     .language(SV_SE, "Svenska")
///     .catalog(&APP)
///     .catalog(&sockel::phrasebookcatalog01::L10N)
///     .build();
/// ```
pub static L10N: Catalog = Catalog::new(
    "sockel",
    &[
        Source::new(Lang::EN_US, include_str!("../locales/en-US.ftl")),
        Source::new(SV_SE, include_str!("../locales/sv-SE.ftl")),
    ],
)
.with_prefix("sockel-");

/// The label given on a use, else `id`'s message from `phrases`: how a
/// component honours an `Option<AttrValue>` label prop.
pub fn label(phrases: &Phrases, given: &Option<AttrValue>, id: &str) -> AttrValue {
    match given {
        Some(given) => given.clone(),
        None => AttrValue::from(phrases.text(id)),
    }
}

#[cfg(test)]
mod tests {
    use phrasebook02::{Lang, Phrasebook, Phrases, check};
    use yew::prelude::*;

    use super::{L10N, SV_SE};
    use crate::{Pager, SaveBar};

    #[derive(Properties, PartialEq)]
    struct Shown {
        phrases: Option<Phrases>,
        label: Option<AttrValue>,
    }

    #[function_component]
    fn Show(props: &Shown) -> Html {
        let bar = html! { <SaveBar dirty=true guard=false label={props.label.clone()} /> };
        match &props.phrases {
            Some(phrases) => html! {
                <ContextProvider<Phrases> context={phrases.clone()}>{ bar }</ContextProvider<Phrases>>
            },
            None => bar,
        }
    }

    fn render(phrases: Option<Phrases>, label: Option<&'static str>) -> String {
        let label = label.map(AttrValue::from);
        futures::executor::block_on(
            yew::LocalServerRenderer::<Show>::with_props(Shown { phrases, label })
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn components_read_the_provided_language_else_english() {
        let book = Phrasebook::builder()
            .language(Lang::EN_US, "English")
            .language(SV_SE, "Svenska")
            .catalog(&L10N)
            .build();

        let en = render(None, None);
        assert!(
            en.contains(">Unsaved changes<") && en.contains(">Save<"),
            "{en}"
        );
        let sv = render(Some(book.phrases(SV_SE)), None);
        assert!(
            sv.contains(">Osparade ändringar<") && sv.contains(">Spara<"),
            "{sv}"
        );
        let given = render(Some(book.phrases(SV_SE)), Some("Changed"));
        assert!(
            given.contains(">Changed<") && given.contains(">Spara<"),
            "{given}"
        );
    }

    #[derive(Properties, PartialEq)]
    struct Paged {
        phrases: Option<Phrases>,
        page_label: Option<AttrValue>,
    }

    #[function_component]
    fn ShowPager(props: &Paged) -> Html {
        let pager = html! {
            <Pager page={0} pages={3} on_page={Callback::noop()}
                page_label={props.page_label.clone()} />
        };
        match &props.phrases {
            Some(phrases) => html! {
                <ContextProvider<Phrases> context={phrases.clone()}>{ pager }</ContextProvider<Phrases>>
            },
            None => pager,
        }
    }

    #[test]
    fn a_numbered_page_is_named_in_the_language() {
        let render = |phrases, page_label: Option<&'static str>| {
            let page_label = page_label.map(AttrValue::from);
            futures::executor::block_on(
                yew::LocalServerRenderer::<ShowPager>::with_props(Paged {
                    phrases,
                    page_label,
                })
                .hydratable(false)
                .render(),
            )
        };
        let book = Phrasebook::builder()
            .language(Lang::EN_US, "English")
            .language(SV_SE, "Svenska")
            .catalog(&L10N)
            .build();

        let en = render(None, None);
        assert!(
            en.contains("aria-label=\"Page 2\"") && en.contains("aria-label=\"Next page\""),
            "{en}"
        );
        let sv = render(Some(book.phrases(SV_SE)), None);
        assert!(
            sv.contains("aria-label=\"Sida 2\"") && sv.contains("aria-label=\"Nästa sida\""),
            "{sv}"
        );
        let given = render(Some(book.phrases(SV_SE)), Some("Blad"));
        assert!(given.contains("aria-label=\"Blad 2\""), "{given}");
    }

    #[test]
    fn catalog() {
        check::catalog(&L10N).unwrap();
    }

    #[test]
    fn swedish_has_every_english_id() {
        check::parity(&L10N, SV_SE).unwrap();
    }

    /// Every `sockel-` id the code asks for is in the catalog, and every
    /// message in it is asked for.
    #[test]
    fn catalog_matches_the_code() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut asked = Vec::new();

        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for (start, _) in source.match_indices("\"sockel-") {
                let line = source[..start].rsplit('\n').next().unwrap().trim_start();
                if line.starts_with("//") || line.contains("assert") {
                    continue;
                }
                // Class names share the prefix: a message id is the argument
                // of a lookup, `text("sockel-..")` or `label(.., "sockel-..")`,
                // or a match arm's `=> ("sockel-..", ..)`.
                let before = source[..start].trim_end();
                if ["text(", "msg(", ",", "=> ("]
                    .iter()
                    .any(|end| before.ends_with(end))
                {
                    let rest = &source[start + 1..];
                    asked.push(rest[..rest.find('"').unwrap()].to_owned());
                }
            }
        }

        check::coverage(&L10N, Lang::EN_US, asked.iter().map(String::as_str), &[]).unwrap();
    }
}
