use yew::prelude::*;

use sockel::{
    EditKey, InlineEdit, Picker, PickerOption, PropertyList, PropertyRow, SelectOption,
    SelectWidth, Time, TimeStyle, edit_key, inline_edit, now_seconds, property_list, property_row,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::Select as Control;
use crate::controls::{Demo, Text, Toggle, bind};

const STATUSES: [&str; 5] = ["triage", "ready", "running", "review", "done"];

#[function_component]
pub fn PropertyListPage() -> Html {
    let boxed = use_state(|| false);
    let disabled = use_state(|| false);
    let busy = use_state(|| false);
    let text = use_state(|| Text::Short);
    let title = use_state(|| AttrValue::from("Redesign 2.8: property list"));
    let owner = use_state(|| AttrValue::from(""));
    let description = use_state(|| {
        AttrValue::from("Label and value rows.\nA row edits in place, or opens a picker.")
    });
    let status = use_state(|| 2usize);
    let priority = use_state(|| 1usize);
    let commits = use_state(|| 0u32);
    let opened = use_state(now_seconds);

    let controls = html! {
        <>
            <Toggle label="boxed" value={*boxed} on_change={bind(&boxed)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Control<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let commit = |state: &UseStateHandle<AttrValue>| {
        let (state, commits) = (state.clone(), commits.clone());
        Callback::from(move |value: String| {
            state.set(value.into());
            commits.set(*commits + 1);
        })
    };
    let pick = |state: &UseStateHandle<usize>| {
        let (state, commits) = (state.clone(), commits.clone());
        Callback::from(move |value: usize| {
            state.set(value);
            commits.set(*commits + 1);
        })
    };

    let code = snippet(
        &[],
        El::new("PropertyList")
            .text("label", "Card")
            .flag("boxed", *boxed)
            .child(
                El::new("PropertyRow").text("label", "Title").child(
                    El::new("InlineEdit")
                        .expr("value", "title")
                        .text("label", "Title")
                        .expr("on_commit", "rename")
                        .flag("disabled", *disabled)
                        .flag("busy", *busy),
                ),
            )
            .child(
                El::new("PropertyRow").text("label", "Status").child(
                    El::new("Picker<Status>")
                        .expr("options", "statuses")
                        .expr("value", "Some(status)")
                        .expr("on_change", "move_to")
                        .text("label", "Status")
                        .flag("fill", true),
                ),
            )
            .child(
                El::new("PropertyRow")
                    .text("label", "Description")
                    .flag("stacked", true)
                    .child(
                        El::new("InlineEdit")
                            .expr("value", "description")
                            .text("label", "Description")
                            .flag("multiline", true)
                            .expr("on_commit", "describe"),
                    ),
            ),
    );

    let label = |words: &str| AttrValue::from(text.of(words));
    let statuses: Vec<PickerOption<usize>> = STATUSES
        .iter()
        .enumerate()
        .map(|(i, name)| PickerOption::new(i, *name))
        .collect();
    let priorities = vec![
        SelectOption::new(0, "Low"),
        SelectOption::new(1, "Normal"),
        SelectOption::new(2, "High"),
    ];
    let enter = edit_key("Enter", false, false, false) == EditKey::Commit;

    html! {
        <Demo {controls} {code}>
            <PropertyList label="Card" boxed={*boxed}>
                <PropertyRow label={label("Title")}>
                    <InlineEdit value={(*title).clone()} label="Title" on_commit={commit(&title)}
                        disabled={*disabled} busy={*busy} />
                </PropertyRow>
                <PropertyRow label={label("Status")}>
                    <Picker<usize> options={statuses} value={Some(*status)} on_change={pick(&status)}
                        label="Status" fill=true disabled={*disabled} />
                </PropertyRow>
                <PropertyRow label={label("Priority")}>
                    <sockel::Select<usize> options={priorities} value={Some(*priority)}
                        on_change={pick(&priority)} label="Priority" width={SelectWidth::Fill}
                        disabled={*disabled} />
                </PropertyRow>
                <PropertyRow label={label("Owner")} hint="Empty until a worker claims it">
                    <InlineEdit value={(*owner).clone()} label="Owner" placeholder="Nobody"
                        on_commit={commit(&owner)} disabled={*disabled} />
                </PropertyRow>
                <PropertyRow label={label("Created")}>
                    <span><Time at={*opened - 3 * 3600} style={TimeStyle::Ago} /></span>
                </PropertyRow>
                <PropertyRow label={label("Description")} stacked=true>
                    <InlineEdit value={(*description).clone()} label="Description" multiline=true
                        on_commit={commit(&description)} disabled={*disabled} />
                </PropertyRow>
            </PropertyList>
            <code class={class::CLASS_CALL}>
                { format!(
                    "property_list(boxed) → \"{}\" · property_row(true) → \"{}\" · inline_edit(empty, multiline) → \"{}\"",
                    property_list(*boxed).to_string(),
                    property_row(true).to_string(),
                    inline_edit(owner.is_empty(), false).to_string(),
                ) }
            </code>
            <span class={class::CONTROL_LABEL}>
                { format!(
                    "{} commits. Click a value, or Tab to it and press Enter: {} a line or blur commits, Escape cancels; long text saves with Ctrl/Cmd+Enter.",
                    *commits,
                    if enter { "Enter on" } else { "on" },
                ) }
            </span>
        </Demo>
    }
}
