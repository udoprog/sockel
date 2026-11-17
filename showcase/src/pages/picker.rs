use yew::prelude::*;

use plinth::{
    Badge, Button, ButtonSize, ButtonVariant, ChoiceList, ChoicePopup, Cluster, Dismiss, Field,
    Glyph, Icon, PickerDialog, PickerMenu, PickerOption, PickerVariant, Stack, field_describedby,
    toggled,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select as Control, Text, Toggle, bind};

const VARIANTS: &[(&str, PickerVariant)] = &[
    ("Field", PickerVariant::Field),
    ("Small", PickerVariant::Small),
];

const TEXTS: &[(&str, Text)] = &[
    ("Short", Text::Short),
    ("Long", Text::Long),
    ("Long, unbroken", Text::Unbroken),
];

/// How many options a picker offers.
const COUNTS: &[(&str, usize)] = &[("5 (no search)", 5), ("24 (searches)", 24), ("0", 0)];

const LANGUAGES: [(&str, &str, &str); 24] = [
    ("Swedish", "sv", "svenska"),
    ("Swedish (Finland)", "sv-FI", "svenska"),
    ("English (Sweden)", "en-SE", ""),
    ("English", "en", ""),
    ("Finnish", "fi", "suomi"),
    ("Norwegian", "nb", "norsk bokmal"),
    ("Danish", "da", "dansk"),
    ("Icelandic", "is", "islenska"),
    ("Lule Sami (Sweden)", "smj-SE", ""),
    ("Dutch", "nl", "nederlands"),
    ("German", "de", "deutsch"),
    ("French", "fr", "francais"),
    ("Spanish", "es", "espanol"),
    ("Portuguese", "pt", "portugues"),
    ("Italian", "it", "italiano"),
    ("Polish", "pl", "polski"),
    ("Czech", "cs", "cestina"),
    ("Hungarian", "hu", "magyar"),
    ("Estonian", "et", "eesti"),
    ("Latvian", "lv", "latviesu"),
    ("Lithuanian", "lt", "lietuviu"),
    ("Greek", "el", ""),
    ("Turkish", "tr", "turkce"),
    ("Japanese", "ja", "nihongo"),
];

fn languages(
    count: usize,
    text: Text,
    icons: bool,
    badge: bool,
    marked: bool,
) -> Vec<PickerOption<usize>> {
    LANGUAGES
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, &(name, code, native))| {
            let label = if i == 0 {
                text.of(name)
            } else {
                name.to_owned()
            };
            let mut option = PickerOption::new(i, label).detail(code).keywords(native);
            if icons {
                option = option.icon(html! { <Icon glyph={Glyph::Search} /> });
            }
            if badge && i == 1 {
                option = option.badge(html! { <Badge>{ "New" }</Badge> });
            }
            option.disabled(i == 3).marked(marked && i == 2)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    count: usize,
    text: Text,
    variant: PickerVariant,
    icons: bool,
    fill: bool,
    invalid: bool,
    disabled: bool,
    always_search: bool,
    marked: bool,
    paged: bool,
    detail_below: bool,
    chevron: bool,
}

impl Settings {
    fn page_size(&self) -> Option<usize> {
        self.paged.then_some(10)
    }

    fn options(&self) -> Vec<PickerOption<usize>> {
        languages(self.count, self.text, self.icons, true, self.marked)
    }
}

#[derive(Properties, PartialEq)]
struct SampleProps {
    settings: Settings,
    value: Option<usize>,
    on_change: Callback<usize>,
}

#[function_component]
fn PickerSample(props: &SampleProps) -> Html {
    let s = props.settings;
    let id = use_control_id();
    let error = s.invalid.then(|| html! { "Pick a language you read." });
    html! {
        <Stack>
            <Field id={id.clone()} label={html! { "Language" }} error={error} reserve_error=true
                disabled={s.disabled}>
                <plinth::Picker<usize> id={id.clone()} options={s.options()} page_size={s.page_size()}
                    value={props.value} on_change={props.on_change.clone()}
                    label="Language" placeholder="Choose a language" variant={s.variant}
                    fill={s.fill} invalid={s.invalid} disabled={s.disabled}
                    always_search={s.always_search} detail_below={s.detail_below}
                    describedby={field_describedby(&id, false, s.invalid)} testid="picker" />
            </Field>
            <p class={class::SAMPLE}>{ "The field keeps its width whatever is chosen. Past eight options the list has a search field; the list is --picker-rows rows high however many match." }</p>
        </Stack>
    }
}

fn settings_controls(settings: &UseStateHandle<Settings>) -> Html {
    let s = **settings;
    let set = |f: fn(&mut Settings, bool)| {
        let settings = settings.clone();
        Callback::from(move |on| {
            let mut next = *settings;
            f(&mut next, on);
            settings.set(next);
        })
    };
    html! {
        <>
            <Control<usize> label="options" options={COUNTS} value={s.count} on_change={{
                let settings = settings.clone();
                Callback::from(move |count| settings.set(Settings { count, ..*settings }))
            }} />
            <Control<Text> label="Option text" options={TEXTS} value={s.text} on_change={{
                let settings = settings.clone();
                Callback::from(move |text| settings.set(Settings { text, ..*settings }))
            }} />
            <Control<PickerVariant> label="variant" options={VARIANTS} value={s.variant} on_change={{
                let settings = settings.clone();
                Callback::from(move |variant| settings.set(Settings { variant, ..*settings }))
            }} />
            <Toggle label="icons" value={s.icons} on_change={set(|s, on| s.icons = on)} />
            <Toggle label="always_search" value={s.always_search} on_change={set(|s, on| s.always_search = on)} />
            <Toggle label="page_size 10" value={s.paged} on_change={set(|s, on| s.paged = on)} />
            <Toggle label="marked (English (Sweden))" value={s.marked} on_change={set(|s, on| s.marked = on)} />
            <Toggle label="detail_below" value={s.detail_below} on_change={set(|s, on| s.detail_below = on)} />
            <Toggle label="chevron (PickerMenu, with an icon)" value={s.chevron} on_change={set(|s, on| s.chevron = on)} />
            <Toggle label="fill" value={s.fill} on_change={set(|s, on| s.fill = on)} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
        </>
    }
}

fn initial() -> Settings {
    Settings {
        count: 24,
        text: Text::Short,
        variant: PickerVariant::Field,
        icons: false,
        fill: false,
        invalid: false,
        disabled: false,
        always_search: false,
        marked: false,
        paged: false,
        detail_below: false,
        chevron: false,
    }
}

const OPTIONS_LET: &str = "let options = vec![PickerOption::new(Lang::Sv, \"Swedish\").detail(\"sv\").keywords(\"svenska\"), ..];";

#[function_component]
pub fn PickerPage() -> Html {
    let settings = use_state(initial);
    let value = use_state(|| None::<usize>);
    let s = *settings;
    let code = snippet(
        &[OPTIONS_LET.to_owned()],
        El::new("Picker<Lang>")
            .expr("options", "options")
            .expr("value", "lang")
            .expr("on_change", "set_lang")
            .text("label", "Language")
            .text("placeholder", "Choose a language")
            .variant("variant", s.variant)
            .flag("always_search", s.always_search)
            .expr_if(s.paged, "page_size", "Some(10)")
            .flag("detail_below", s.detail_below)
            .flag("fill", s.fill)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled),
    );
    let on_change = {
        let value = value.clone();
        Callback::from(move |v: usize| value.set(Some(v)))
    };
    html! {
        <Demo controls={settings_controls(&settings)} {code}>
            <PickerSample settings={s} value={*value} {on_change} />
        </Demo>
    }
}

#[function_component]
pub fn PickerMenuPage() -> Html {
    let settings = use_state(initial);
    let added = use_state(Vec::<usize>::new);
    let multi = use_state(|| false);
    let field = use_state(|| false);
    let open = use_state(|| false);
    let s = *settings;
    let options = s.options();
    let on_pick = {
        let (added, multi) = (added.clone(), *multi);
        Callback::from(move |v: usize| {
            if multi {
                added.set(toggled(&added, &v));
            } else if !added.contains(&v) {
                added.set(added.iter().copied().chain([v]).collect());
            }
        })
    };
    let on_toggle = {
        let open = open.clone();
        Callback::from(move |to: bool| open.set(to))
    };
    let done = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(false))
    };
    let names: Vec<String> = added
        .iter()
        .filter_map(|&i| options.get(i).map(|o| o.label.to_string()))
        .collect();
    let code = snippet(
        &[OPTIONS_LET.to_owned()],
        El::new("PickerMenu<Lang>")
            .expr("options", "options")
            .expr("on_pick", "add_lang")
            .expr("text", "html! { \"Add a language\" }")
            .flag("field", *field)
            .text("label", "Add a language")
            .expr_if(
                s.chevron,
                "icon",
                "Some(html! { <Icon glyph={Glyph::Plus} /> })",
            )
            .flag("chevron", s.chevron)
            .flag("always_search", s.always_search)
            .expr_if(s.paged, "page_size", "Some(10)")
            .flag("detail_below", s.detail_below)
            .flag("multi", *multi)
            .expr_if(*multi, "chosen", "chosen")
            .expr_if(*multi, "open", "Some(*open)")
            .expr_if(*multi, "on_toggle", "set_open")
            .flag("disabled", s.disabled),
    );
    let controls = html! {
        <>
            { settings_controls(&settings) }
            <Toggle label="multi, with Done" value={*multi} on_change={bind(&multi)} />
            <Toggle label="field" value={*field} on_change={bind(&field)} />
        </>
    };
    html! {
        <Demo {controls} {code}>
            <Stack>
                <Cluster>
                    <PickerMenu<usize> {options} {on_pick} text={html! { "Add a language" }}
                        label="Add a language" always_search={s.always_search} page_size={s.page_size()}
                        icon={s.chevron.then(|| html! { <Icon glyph={Glyph::Plus} /> })} chevron={s.chevron}
                        detail_below={s.detail_below} multi={*multi} chosen={(*added).clone()}
                        open={multi.then_some(*open)} {on_toggle} field={*field}
                        disabled={s.disabled} testid="picker-menu">
                        if *multi {
                            <Button variant={ButtonVariant::Primary} size={ButtonSize::Small} onclick={done}>
                                { "Done" }
                            </Button>
                        }
                    </PickerMenu<usize>>
                </Cluster>
                <p class={class::SAMPLE}>{ format!("Added: {}", if names.is_empty() { "none".to_owned() } else { names.join(", ") }) }</p>
            </Stack>
        </Demo>
    }
}

#[function_component]
pub fn PickerDialogPage() -> Html {
    let settings = use_state(|| Settings {
        always_search: true,
        ..initial()
    });
    let open = use_state(|| false);
    let multi = use_state(|| true);
    let chosen = use_state(|| vec![0usize, 4]);
    let value = use_state(|| Some(0usize));
    let s = *settings;
    let options = s.options();
    let on_pick = {
        let (chosen, value, open, multi) = (chosen.clone(), value.clone(), open.clone(), *multi);
        Callback::from(move |v: usize| {
            if multi {
                chosen.set(toggled(&chosen, &v));
            } else {
                value.set(Some(v));
                open.set(false);
            }
        })
    };
    let close = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    let controls = html! {
        <>
            { settings_controls(&settings) }
            <Toggle label="multi" value={*multi} on_change={bind(&multi)} />
        </>
    };
    let code = snippet(
        &[OPTIONS_LET.to_owned()],
        El::new("PickerDialog<Lang>")
            .expr("open", "open")
            .expr("on_dismiss", "close")
            .text("title", "Languages")
            .expr("options", "options")
            .flag("multi", *multi)
            .expr_if(*multi, "chosen", "langs")
            .expr_if(!*multi, "value", "lang")
            .expr("on_pick", "pick")
            .expr_if(
                *multi,
                "actions",
                "html! { <Button onclick={done}>{ \"Done\" }</Button> }",
            ),
    );
    let actions = (*multi).then(|| {
        html! { <Button variant={ButtonVariant::Primary} onclick={close.reform(|_: MouseEvent| ())}>{ "Done" }</Button> }
    });
    html! {
        <Demo {controls} {code}>
            <Stack>
                <Cluster>
                    <Button onclick={{ let open = open.clone(); Callback::from(move |_| open.set(true)) }}
                        disabled={s.disabled} testid="picker-dialog-open">{ "Choose languages" }</Button>
                </Cluster>
                <ChoiceList<usize> options={options.clone()} value={*value} multi={*multi}
                    chosen={(*chosen).clone()} on_pick={on_pick.clone()} label="Languages"
                    always_search={s.always_search} page_size={s.page_size()} testid="choice-list" />
                <p class={class::SAMPLE}>{ "The same ChoiceList inline: filtering keeps its height. The dialog holds it, its search field focused." }</p>
                <PickerDialog<usize> open={*open} on_dismiss={close.reform(|_: plinth::Dismiss| ())} title="Languages"
                    {options} value={*value} multi={*multi} chosen={(*chosen).clone()} {on_pick}
                    actions={actions.unwrap_or_default()} testid="picker-dialog" />
            </Stack>
        </Demo>
    }
}

#[function_component]
pub fn ChoicePopupPage() -> Html {
    let opener = use_node_ref();
    let open = use_state_eq(|| false);
    let chosen = use_state(Vec::<usize>::new);
    let options = languages(12, Text::Short, false, true, false);
    let close = {
        let open = open.clone();
        Callback::from(move |_: Dismiss| open.set(false))
    };
    let on_pick = {
        let chosen = chosen.clone();
        Callback::from(move |v: usize| chosen.set(toggled(&chosen, &v)))
    };
    let onclick = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    let done = close.reform(|_: MouseEvent| Dismiss::Outside);
    let code = snippet(
        &["let opener = use_node_ref();".to_owned()],
        El::new("ChoicePopup")
            .expr("anchor", "opener")
            .expr("open", "*open")
            .expr("on_dismiss", "close")
            .expr(
                "children",
                "html! { <ChoiceList<Lang> multi=true autofocus=true .. /> }",
            ),
    );
    html! {
        <Demo controls={Html::default()} {code}>
            <Cluster>
                <Button node_ref={opener.clone()} variant={ButtonVariant::Ghost}
                    icon={html! { <Icon glyph={Glyph::Plus} /> }} chevron=true
                    expanded={Some(*open)} haspopup={AttrValue::Static("listbox")} {onclick}
                    testid="choice-popup-opener">
                    { format!("Readers: {}", chosen.len()) }
                </Button>
                <ChoicePopup anchor={opener} open={*open} on_dismiss={close} testid="choice-popup">
                    <Stack>
                        <p class={class::SAMPLE}>{ "Who reads it" }</p>
                        <ChoiceList<usize> {options} multi=true chosen={(*chosen).clone()} {on_pick}
                            label="Readers" autofocus=true />
                        <Cluster>
                            <Button variant={ButtonVariant::Primary} onclick={done}>{ "Done" }</Button>
                        </Cluster>
                    </Stack>
                </ChoicePopup>
            </Cluster>
        </Demo>
    }
}
