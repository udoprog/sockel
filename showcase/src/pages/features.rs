use yew::prelude::*;

use sockel::{Gap, Stack, Table, TableCell, TableRow};

use super::reference::{head, prose};
use crate::class;
use crate::code::CodeCard;

const MANIFEST: &str = r#"[dependencies]
sockel = { version = "0.3", default-features = false, features = ["button", "layout", "overlay"] }"#;

pub struct Feature {
    pub name: &'static str,
    /// The features it turns on with it.
    pub enables: &'static [&'static str],
    pub doc: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/features.rs"));

/// Every Cargo feature of sockel, from its `Cargo.toml`.
#[function_component]
pub fn FeaturesPage() -> Html {
    let rows: Html = FEATURES
        .iter()
        .map(|feature| {
            let enables = (!feature.enables.is_empty() && feature.name != "full").then(|| {
                html! {
                    <span class={class::CONTROL_LABEL}>
                        { "Enables: " }
                        { for feature.enables.iter().map(|f| html! { <><code>{ *f }</code>{ " " }</> }) }
                    </span>
                }
            });
            html! {
                <TableRow>
                    <TableCell title=true><code>{ feature.name }</code></TableCell>
                    <TableCell wrap=true>
                        <Stack gap={Gap::Tight}>
                            <span>{ prose(feature.doc) }</span>
                            { enables }
                        </Stack>
                    </TableCell>
                </TableRow>
            }
        })
        .collect();

    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "Each component, or family of components that only work together, is a Cargo feature \
                   gating its Rust and its CSS. full, the default, enables them all; an app that renders \
                   a few lists those, and each brings in the features it needs. This page is generated \
                   from sockel's Cargo.toml. The CSS each feature adds is not listed here: \
                   scripts/feature-sizes.sh in the repository measures it." }
            </p>
            <CodeCard code={MANIFEST} />
            <Table head={head(&["Feature", "What it brings in"])}
                columns={2} top=true cards=true framed=true label="Features">
                { rows }
            </Table>
        </Stack>
    }
}
