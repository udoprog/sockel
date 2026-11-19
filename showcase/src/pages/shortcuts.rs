use yew::prelude::*;

use sockel::{
    Button, Chord, Kbd, Menu, MenuItem, Shortcut, ShortcutHelp, Stack, TextInput, is_apple, kbd,
    shortcut_applies, use_shortcut, use_shortcut_layer, use_shortcut_list,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const CHORDS: &[(&str, &str)] = &[
    ("Mod+Enter", "Mod+Enter"),
    ("?", "?"),
    ("S (Shift+s)", "S"),
    ("Ctrl+Shift+k", "Ctrl+Shift+k"),
    ("ArrowDown", "ArrowDown"),
    ("Escape", "Escape"),
];

const ROWS: [&str; 4] = ["Inbox", "Boards", "Sessions", "Settings"];

#[function_component]
pub fn ShortcutsPage() -> Html {
    let chord = use_state(|| "Mod+Enter");
    let decorative = use_state(|| false);
    let editing = use_state(|| false);
    let enabled = use_state(|| true);
    let help = use_state(|| false);
    let row = use_state(|| 0usize);
    let last = use_state(String::new);

    // The page's bindings, once (the frames below draw its demo twice).
    let said = |label: &'static str, then: Callback<()>| {
        let last = last.clone();
        Callback::from(move |e: KeyboardEvent| {
            last.set(format!("{label} ({})", e.key()));
            then.emit(());
        })
    };
    let step = |by: isize| {
        let row = row.clone();
        Callback::from(move |()| {
            row.set((*row as isize + by).rem_euclid(ROWS.len() as isize) as usize);
        })
    };
    let set = |state: &UseStateHandle<bool>, value: bool| {
        let state = state.clone();
        Callback::from(move |()| state.set(value))
    };
    use_shortcut(
        Shortcut::new("?", "Show shortcuts").always(),
        said("Show shortcuts", set(&help, true)),
    );
    use_shortcut(
        Shortcut::new("j", "Next row")
            .or("ArrowDown")
            .group("Rows")
            .enabled(*enabled),
        said("Next row", step(1)),
    );
    use_shortcut(
        Shortcut::new("k", "Previous row")
            .or("ArrowUp")
            .group("Rows")
            .enabled(*enabled),
        said("Previous row", step(-1)),
    );
    use_shortcut(
        Shortcut::new("e", "Edit the row")
            .group("Rows")
            .enabled(*enabled),
        said("Edit the row", set(&editing, true)),
    );
    use_shortcut(
        Shortcut::new("S", "Shut down")
            .group("Session")
            .enabled(*enabled),
        said("Shut down", Callback::noop()),
    );
    use_shortcut(
        Shortcut::new("Mod+Enter", "Save").layer("edit").in_fields(),
        said("Save", set(&editing, false)),
    );
    use_shortcut(
        Shortcut::new("Escape", "Stop editing")
            .layer("edit")
            .in_fields(),
        said("Stop editing", set(&editing, false)),
    );
    use_shortcut_layer("edit".into(), *editing);
    let listed = use_shortcut_list().len();
    let typed = use_state(String::new);

    let controls = html! {
        <>
            <Select<&'static str> label="keys" options={CHORDS} value={*chord} on_change={bind(&chord)} />
            <Toggle label="decorative" value={*decorative} on_change={bind(&decorative)} />
            <Toggle label="Editing layer" value={*editing} on_change={bind(&editing)} />
            <Toggle label="Row keys enabled" value={*enabled} on_change={bind(&enabled)} />
        </>
    };

    let code = snippet(
        &[
            "use_shortcut(Shortcut::new(\"?\", \"Show shortcuts\").always(), open_help);".into(),
            "use_shortcut(Shortcut::new(\"j\", \"Next row\").or(\"ArrowDown\").group(\"Rows\"), next);".into(),
            "use_shortcut(Shortcut::new(\"Mod+Enter\", \"Save\").layer(\"edit\").in_fields(), save);".into(),
            "use_shortcut_layer(\"edit\".into(), editing);".into(),
        ],
        El::new("")
            .child(
                El::new("Kbd")
                    .text("keys", &chord)
                    .flag("decorative", *decorative),
            )
            .child(
                El::new("MenuItem")
                    .text("keys", "e")
                    .line("{ \"Edit\" }"),
            )
            .child(
                El::new("ShortcutHelp")
                    .expr("open", "help")
                    .expr("on_dismiss", "close_help"),
            ),
    );

    let close = {
        let help = help.clone();
        Callback::from(move |_| help.set(false))
    };
    let open = {
        let help = help.clone();
        Callback::from(move |_| help.set(true))
    };
    let parsed = Chord::parse(&chord);
    let applies = shortcut_applies(None, false, editing.then_some("edit"), false);

    html! {
        <>
            <Demo {controls} {code}>
                <Stack>
                    <span class={class::CONTROL_LABEL}>
                        <Kbd keys={*chord} decorative={*decorative} />
                        { format!(" {} · aria {} · {}", kbd().to_string(), parsed.aria(is_apple()), parsed.text()) }
                    </span>
                    <ul class={class::SAMPLE}>
                        { for ROWS.iter().enumerate().map(|(n, name)| html! {
                            <li aria-current={(n == *row).then_some("true")}>
                                { if n == *row { format!("▸ {name}") } else { format!("  {name}") } }
                                if n == *row && *editing { { " (editing)" } }
                            </li>
                        }) }
                    </ul>
                    <TextInput value={(*typed).clone()} on_input={bind(&typed)}
                        label="Typing" placeholder="Typing here skips j and k" />
                    <Menu text={html! { "Row actions" }} label="Row actions">
                        <MenuItem keys="e">{ "Edit" }</MenuItem>
                        <MenuItem keys="Mod+Enter" hint="saves">{ "Save" }</MenuItem>
                        <MenuItem keys="S" danger=true>{ "Shut down" }</MenuItem>
                    </Menu>
                    <Button onclick={open}>{ "Keyboard shortcuts" }</Button>
                    <span class={class::CONTROL_LABEL}>
                        { format!(
                            "Last: {} · base keys {} · {listed} listed. Press ? for the help, j/k to move, e to edit (Ctrl/Cmd+Enter or Esc leaves).",
                            if last.is_empty() { "none" } else { last.as_str() },
                            if applies { "apply" } else { "held by the edit layer" },
                        ) }
                    </span>
                </Stack>
            </Demo>
            <ShortcutHelp open={*help} on_dismiss={close} />
        </>
    }
}
