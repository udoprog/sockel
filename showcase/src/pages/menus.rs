use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, ButtonVariant, Cluster, ContextMenu, DataTips, Dialog, Glyph, Icon, Menu,
    MenuCheck, MenuHeading, MenuItem, MenuNote, MenuRadio, MenuSeparator, MenuWidth, RowAction,
    RowActions, SplitButton, Stack, TextInput,
};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const WIDTHS: &[(&str, MenuWidth)] = &[
    ("Regular", MenuWidth::Regular),
    ("Wide", MenuWidth::Wide),
    ("Wider", MenuWidth::Wider),
    ("Panel", MenuWidth::Panel),
];

const VARIANTS: &[(&str, ButtonVariant)] = &[
    ("Secondary", ButtonVariant::Secondary),
    ("Primary", ButtonVariant::Primary),
    ("Ghost", ButtonVariant::Ghost),
    ("Quiet", ButtonVariant::Quiet),
];

const SIZES: &[(&str, ButtonSize)] = &[
    ("Regular", ButtonSize::Regular),
    ("Small", ButtonSize::Small),
    ("Touch", ButtonSize::Touch),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Items {
    Actions,
    Choices,
    Empty,
}

const ITEMS: &[(&str, Items)] = &[
    ("Actions", Items::Actions),
    ("Checks and radios", Items::Choices),
    ("Empty (a note)", Items::Empty),
];

#[derive(Properties, PartialEq)]
struct ItemsProps {
    items: Items,
    text: Text,
    disabled: bool,
}

/// The sample items, with state of their own so a check or radio can be
/// picked in each frame.
#[function_component]
fn SampleItems(props: &ItemsProps) -> Html {
    let wrap = use_state(|| true);
    let sort = use_state(|| 0usize);
    let picked = use_state(|| None::<&'static str>);
    let pick = |name: &'static str| {
        let picked = picked.clone();
        Callback::from(move |_: MouseEvent| picked.set(Some(name)))
    };
    let label = props.text.of("Duplicate");

    match props.items {
        Items::Empty => {
            html! { <MenuNote testid="menu-empty">{ "Nothing to do here." }</MenuNote> }
        }
        Items::Actions => html! {
            <>
                <MenuHeading>{ "Edit" }</MenuHeading>
                <MenuItem icon={html! { <Icon glyph={Glyph::Plus} /> }} hint="Ctrl+D" onclick={pick("duplicate")}>
                    { label }
                </MenuItem>
                <MenuItem detail="Keeps its history" onclick={pick("rename")}>{ "Rename" }</MenuItem>
                <MenuItem disabled={props.disabled} onclick={pick("move")}>{ "Move to…" }</MenuItem>
                <MenuItem onclick={pick("copy")}>{ "Copy link" }</MenuItem>
                <MenuItem href="#menu" current=true detail="A link row, the page shown">{ "Open this page" }</MenuItem>
                <MenuItem href="https://example.org/" new_tab=true detail="A link row in a new tab"
                    detail_html={html! { <>{ " · " }<em>{ "example.org" }</em></> }}>
                    { "Look it up" }
                </MenuItem>
                <MenuSeparator />
                <MenuItem danger=true tip="Delete this file" onclick={pick("delete")}>{ "Delete" }</MenuItem>
            </>
        },
        Items::Choices => {
            let radio = |n: usize, name: &'static str| {
                let sort = sort.clone();
                html! {
                    <MenuRadio checked={*sort == n} on_pick={Callback::from(move |()| sort.set(n))}>
                        { name }
                    </MenuRadio>
                }
            };
            let toggle = {
                let wrap = wrap.clone();
                Callback::from(move |on| wrap.set(on))
            };
            html! {
                <>
                    <MenuCheck checked={*wrap} on_change={toggle} disabled={props.disabled}
                        icon={html! { <Icon glyph={Glyph::Search} /> }}>
                        { "Wrap lines" }
                    </MenuCheck>
                    <MenuSeparator />
                    <MenuHeading>{ "Sort by" }</MenuHeading>
                    { radio(0, "Name") }
                    { radio(1, "Date") }
                    { radio(2, "Size") }
                </>
            }
        }
    }
}

/// A tool whose click toggles it and whose right-click opens its sizes,
/// the checked one focused on open.
#[function_component]
fn ToolMenu() -> Html {
    let on = use_state(|| false);
    let size = use_state(|| 1usize);
    let toggle = {
        let on = on.clone();
        Callback::from(move |_: MouseEvent| on.set(!*on))
    };
    let radio = |n: usize, name: &'static str| {
        let size = size.clone();
        html! {
            <MenuRadio checked={*size == n} on_pick={Callback::from(move |()| size.set(n))}>
                { name }
            </MenuRadio>
        }
    };
    html! {
        <Menu label="Pen" tip="Pen: right-click for its size" icon={html! { <Icon glyph={Glyph::Plus} /> }}
            on_click={toggle} pressed={Some(*on)}>
            <MenuHeading>{ "Size" }</MenuHeading>
            { radio(0, "Fine") }
            { radio(1, "Medium") }
            { radio(2, "Broad") }
        </Menu>
    }
}

/// A menu holding a field: Tab moves between it and the button.
#[function_component]
fn FieldMenu() -> Html {
    let name = use_state(|| AttrValue::Static("Draft"));
    let on_input = {
        let name = name.clone();
        Callback::from(move |v: String| name.set(v.into()))
    };
    html! {
        <Menu text={html! { "Rename" }} label="Rename" width={MenuWidth::Panel}>
            <Stack>
                <TextInput value={(*name).clone()} {on_input} label="Name" />
                <Button variant={ButtonVariant::Primary}>{ "Save" }</Button>
            </Stack>
        </Menu>
    }
}

/// An item opening a dialog: when it closes, focus is back on the opener.
#[function_component]
fn DialogMenu() -> Html {
    let open = use_state(|| false);
    let show = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(true))
    };
    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |_: plinth::Dismiss| open.set(false))
    };
    html! {
        <>
            <Menu text={html! { "File" }} label="File">
                <MenuItem onclick={show}>{ "Properties…" }</MenuItem>
            </Menu>
            <Dialog open={*open} {on_dismiss} title="Properties">
                <p>{ "Close this dialog: focus goes back to File." }</p>
            </Dialog>
        </>
    }
}

#[function_component]
pub fn MenuPage() -> Html {
    let width = use_state(MenuWidth::default);
    let variant = use_state(ButtonVariant::default);
    let size = use_state(ButtonSize::default);
    let items = use_state(|| Items::Actions);
    let text = use_state(|| Text::Short);
    let icon_only = use_state(|| false);
    let tight = use_state(|| false);
    let disabled = use_state(|| false);
    let item_disabled = use_state(|| true);

    let controls = html! {
        <>
            <Select<MenuWidth> label="width" options={WIDTHS} value={*width} on_change={bind(&width)} />
            <Select<ButtonVariant> label="variant" options={VARIANTS} value={*variant} on_change={bind(&variant)} />
            <Select<ButtonSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<Items> label="Items" options={ITEMS} value={*items} on_change={bind(&items)} />
            <Select<Text> label="Item text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="icon only" value={*icon_only} on_change={bind(&icon_only)} />
            <Toggle label="tight" value={*tight} on_change={bind(&tight)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="an item disabled" value={*item_disabled} on_change={bind(&item_disabled)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("Menu")
            .expr_if(!*icon_only, "text", "html! { \"Actions\" }")
            .text("label", "Actions")
            .variant("variant", *variant)
            .variant("size", *size)
            .variant("width", *width)
            .flag("tight", *tight)
            .flag("disabled", *disabled)
            .child(
                El::new("MenuItem")
                    .text("hint", "Ctrl+D")
                    .expr("onclick", "duplicate")
                    .line(text_node(&text.of("Duplicate"))),
            )
            .line("<MenuSeparator />")
            .child(
                El::new("MenuItem")
                    .flag("danger", true)
                    .line(text_node("Delete")),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <Stack>
                <Cluster>
                    <Menu text={if *icon_only { Html::default() } else { html! { "Actions" } }}
                        label="Actions" variant={*variant} size={*size} width={*width}
                        tight={*tight} disabled={*disabled}>
                        <SampleItems items={*items} text={*text} disabled={*item_disabled} />
                    </Menu>
                    <button type="button" class={classes!(class::BUTTON)}>{ "Beside it" }</button>
                </Cluster>
                <p class={class::SAMPLE}>{ "Open the menu: nothing here moves. Arrows, Home, End and typing move between items; Escape closes it and focus returns to its opener." }</p>
                <Cluster>
                    <ToolMenu />
                    <FieldMenu />
                    <DialogMenu />
                </Cluster>
                <DataTips />
                <p class={class::SAMPLE}>{ "The pen is a tool: a click toggles it, a right-click opens its sizes with the checked one focused. Rename holds a field: Tab moves between the field and Save, and closes past either end. Tab from any other open menu closes it and moves on past its opener. File's item opens a dialog, which gives focus back to File when it closes." }</p>
            </Stack>
        </Demo>
    }
}

#[function_component]
pub fn RowActionsPage() -> Html {
    let text = use_state(|| Text::Short);
    let more = use_state(|| true);
    let disabled = use_state(|| false);
    let more_disabled = use_state(|| false);
    let tips = use_state(|| false);
    let pressed = use_state(|| false);

    let controls = html! {
        <>
            <Select<Text> label="Row text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="more (a menu)" value={*more} on_change={bind(&more)} />
            <Toggle label="more_disabled" value={*more_disabled} on_change={bind(&more_disabled)} />
            <Toggle label="tips" value={*tips} on_change={bind(&tips)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let toggle = {
        let pressed = pressed.clone();
        Callback::from(move |_: MouseEvent| pressed.set(!*pressed))
    };
    let actions = vec![
        RowAction::new(html! { <Icon glyph={Glyph::Search} /> }, "Show", toggle).pressed(*pressed),
        RowAction::new(
            html! { <Icon glyph={Glyph::Close} /> },
            "Delete",
            Callback::noop(),
        )
        .danger(),
        RowAction::new(
            html! { <Icon glyph={Glyph::Plus} /> },
            "Add",
            Callback::noop(),
        ),
    ];

    let code = snippet(
        &[
            "let actions = vec![".to_owned(),
            "    RowAction::new(show_icon, \"Show\", show).pressed(shown),".to_owned(),
            "    RowAction::new(delete_icon, \"Delete\", delete).danger(),".to_owned(),
            "    RowAction::new(add_icon, \"Add\", add),".to_owned(),
            "];".to_owned(),
        ],
        El::new("RowActions")
            .expr("actions", "actions")
            .expr_if(
                *more,
                "more",
                "Some(html! { <MenuItem>{ \"Rename\" }</MenuItem> })",
            )
            .flag("more_disabled", *more_disabled)
            .flag("tips", *tips)
            .flag("disabled", *disabled),
    );

    html! {
        <Demo {controls} {code}>
            <Cluster justify={plinth::Justify::Between}>
                <span class={class::SAMPLE}>{ text.of("A row") }</span>
                <DataTips />
                <RowActions {actions} disabled={*disabled} more_disabled={*more_disabled} tips={*tips}
                    more={more.then(|| html! {
                        <>
                            <MenuItem>{ "Rename" }</MenuItem>
                            <MenuItem>{ "Duplicate" }</MenuItem>
                        </>
                    })} />
            </Cluster>
        </Demo>
    }
}

#[function_component]
pub fn SplitButtonPage() -> Html {
    let variant = use_state(|| ButtonVariant::Primary);
    let size = use_state(ButtonSize::default);
    let text = use_state(|| Text::Short);
    let busy = use_state(|| false);
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<ButtonVariant> label="variant" options={VARIANTS} value={*variant} on_change={bind(&variant)} />
            <Select<ButtonSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let label = text.of("Save");
    let code = snippet(
        &[],
        El::new("SplitButton")
            .expr("text", format!("html! {}", text_node(&label)))
            .expr("onclick", "save")
            .expr_if(
                *variant != ButtonVariant::Primary,
                "variant",
                format!("ButtonVariant::{:?}", *variant),
            )
            .variant("size", *size)
            .flag("busy", *busy)
            .flag("disabled", *disabled)
            .line("<MenuItem>{ \"Save as copy\" }</MenuItem>")
            .line("<MenuItem>{ \"Save and close\" }</MenuItem>"),
    );

    html! {
        <Demo {controls} {code}>
            <Cluster>
                <SplitButton text={html! { { label } }} onclick={Callback::noop()}
                    variant={*variant} size={*size} busy={*busy} disabled={*disabled}>
                    <MenuItem>{ "Save as copy" }</MenuItem>
                    <MenuItem>{ "Save and close" }</MenuItem>
                </SplitButton>
                <button type="button" class={classes!(class::BUTTON)}>{ "Beside it" }</button>
            </Cluster>
        </Demo>
    }
}

#[function_component]
pub fn ContextMenuPage() -> Html {
    let width = use_state(MenuWidth::default);
    let items = use_state(|| Items::Actions);
    let text = use_state(|| Text::Short);
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<MenuWidth> label="width" options={WIDTHS} value={*width} on_change={bind(&width)} />
            <Select<Items> label="Items" options={ITEMS} value={*items} on_change={bind(&items)} />
            <Select<Text> label="Item text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="disabled (the browser's menu)" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("ContextMenu")
            .expr(
                "items",
                "html! { <MenuItem onclick={copy}>{ \"Copy\" }</MenuItem> }",
            )
            .variant("width", *width)
            .flag("disabled", *disabled)
            .line("<div class=\"row\">{ \"Right-click or long-press me\" }</div>"),
    );

    html! {
        <Demo {controls} {code}>
            <ContextMenu width={*width} disabled={*disabled} label="Row"
                items={html! { <SampleItems items={*items} text={*text} disabled=true /> }}>
                <p class={class::SAMPLE}>{ "Right-click here, or press and hold on a touch screen: the menu opens at the pointer." }</p>
            </ContextMenu>
        </Demo>
    }
}
