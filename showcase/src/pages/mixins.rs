use yew::prelude::*;

use sockel::{Gap, Stack, Table, TableCell, TableRow};

use super::reference::{MIXINS, MixinSection, head, prose};
use crate::class;
use crate::code::CodeCard;
use crate::controls::section_head;

const ID: &str = "mixins";

const USE: &str = r#"@use "sockel" as p;

.app-sign-in {
    @include p.card;
}

.app-total {
    @include p.numeric;
}"#;

fn section(section: &MixinSection) -> Html {
    let rows: Html = section
        .mixins
        .iter()
        .map(|mixin| {
            let call = if mixin.args.is_empty() {
                mixin.name.to_owned()
            } else {
                format!("{}({})", mixin.name, mixin.args)
            };
            let block = mixin.content.then(|| {
                html! { <span class={class::CONTROL_LABEL}>{ "Takes a block" }</span> }
            });
            html! {
                <TableRow>
                    <TableCell title=true>
                        <Stack gap={Gap::Hair}>
                            <code>{ call }</code>
                            { block }
                        </Stack>
                    </TableCell>
                    <TableCell wrap=true>{ prose(mixin.doc) }</TableCell>
                </TableRow>
            }
        })
        .collect();

    html! {
        <Stack gap={Gap::Snug}>
            { section_head(ID, section.title) }
            <Table head={head(&["Mixin", "What it does"])}
                columns={2} top=true cards=true framed=true label={section.title}>
                { rows }
            </Table>
        </Stack>
    }
}

/// Every public mixin of sockel's Sass, from `style/_tokens.scss`.
#[function_component]
pub fn MixinsPage() -> Html {
    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "sockel's Sass module, @use \"sockel\" in an app's partials, has the mixins its own \
                   components are built from. The rules against jitter keep an element's size as its \
                   content and state change; the recipes are the exact rules of sockel's classes, for \
                   markup sockel does not render. This page is generated from the comments beside each mixin." }
            </p>
            <CodeCard code={USE} />
            { for MIXINS.iter().map(section) }
        </Stack>
    }
}
