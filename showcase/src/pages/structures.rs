use yew::prelude::*;

use crate::class;

use sockel::{
    Gap, Panel, PanelGround, Seam, SeamAxis, Stack, Toolbar, ToolbarPart, ToolbarSpacer, use_width,
};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

const GROUNDS: &[(&str, PanelGround)] = &[
    ("Sunk", PanelGround::Sunk),
    ("Tint", PanelGround::Tint),
    ("Bare", PanelGround::Bare),
];

const AXES: &[(&str, SeamAxis)] = &[("Columns", SeamAxis::Columns), ("Rows", SeamAxis::Rows)];

const SHARES: &[(&str, f64)] = &[
    ("0.35", 0.35),
    ("0.5", 0.5),
    ("0 (the least)", 0.0),
    ("1 (the most)", 1.0),
];

/// A button of the showcase's own, of one width whatever its state.
fn tool(label: String, disabled: bool, small: bool) -> Html {
    html! {
        <button type="button" {disabled}
            class={classes!(class::BUTTON, small.then_some(class::SMALL))}>
            <span class={class::LABEL_TEXT}>{ label }</span>
        </button>
    }
}

#[function_component]
pub fn ToolBandPage() -> Html {
    let band = use_state(|| true);
    let chrome = use_state(|| true);
    let search = use_state(|| false);
    let ruled = use_state(|| true);
    let disabled = use_state(|| false);
    let small = use_state(|| false);
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Toggle label="band" value={*band} on_change={bind(&band)} />
            <Toggle label="chrome" value={*chrome} on_change={bind(&chrome)} />
            <Toggle label="search" value={*search} on_change={bind(&search)} />
            <Toggle label="ruled" value={*ruled} on_change={bind(&ruled)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="Small buttons" value={*small} on_change={bind(&small)} />
            <Select<usize> label="Buttons per part" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let part = |name: &str, ruled: bool| -> Html {
        html! {
            <ToolbarPart {ruled}>
                { for (1..=*count).map(|n| tool(text.of(&format!("{name} {n}")), *disabled, *small)) }
            </ToolbarPart>
        }
    };

    let code = snippet(
        &[],
        El::new("Toolbar")
            .text("label", "Showcase band")
            .flag("band", *band)
            .flag("chrome", *chrome)
            .flag("search", *search)
            .child(
                El::new("ToolbarPart")
                    .line_if(*count > 0, each("view", "button", *count, "button")),
            )
            .child(
                El::new("ToolbarPart")
                    .flag("ruled", *ruled)
                    .line_if(*count > 0, each("edit", "button", *count, "button")),
            )
            .line("<ToolbarSpacer />")
            .child(
                El::new("ToolbarPart").line_if(*count > 0, each("end", "button", *count, "button")),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <Toolbar label="Showcase band" band={*band} chrome={*chrome} search={*search}>
                { part("View", false) }
                { part("Edit", *ruled) }
                <ToolbarSpacer />
                { part("End", *ruled) }
            </Toolbar>
        </Demo>
    }
}

#[function_component]
pub fn PanelPage() -> Html {
    let ground = use_state(|| PanelGround::Sunk);
    let line = use_state(|| false);
    let centered = use_state(|| false);
    let small = use_state(|| false);
    let disabled = use_state(|| false);
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<PanelGround> label="ground" options={GROUNDS} value={*ground} on_change={bind(&ground)} />
            <Toggle label="line" value={*line} on_change={bind(&line)} />
            <Toggle label="centered" value={*centered} on_change={bind(&centered)} />
            <Toggle label="small" value={*small} on_change={bind(&small)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Select<usize> label="Fields" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("Panel")
            .text("label", "Filters")
            .variant("ground", *ground)
            .flag("line", *line)
            .flag("centered", *centered)
            .flag("small", *small)
            .line_if(*count > 0, each("filters", "filter", *count, "field")),
    );

    html! {
        <Demo {controls} {code}>
            <Panel label="Filters" ground={*ground} line={*line} centered={*centered} small={*small}>
                { for (1..=*count).map(|n| html! {
                    <label class={class::CONTROL_LABEL}>
                        <input type="checkbox" disabled={*disabled} />
                        { " " }{ text.of(&format!("Filter {n}")) }
                    </label>
                }) }
            </Panel>
        </Demo>
    }
}

#[function_component]
pub fn SeamPage() -> Html {
    let axis = use_state(|| SeamAxis::Columns);
    let at = use_state(|| 0.35);
    let settled = use_state(|| 0.35);
    let narrowed = use_state(|| false);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<SeamAxis> label="axis" options={AXES} value={*axis} on_change={bind(&axis)} />
            <Select<f64> label="at" options={SHARES} value={*at} on_change={bind(&at)} />
            <Toggle label="min 0.25, max 0.75" value={*narrowed} on_change={bind(&narrowed)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <span class={classes!(class::CONTROL_LABEL, sockel::class::NUMERIC)}>
                { format!("settled at {:.2}", *settled) }
            </span>
        </>
    };

    let (min, max) = if *narrowed { (0.25, 0.75) } else { (0.0, 1.0) };
    let code = snippet(
        &["let at = use_state(|| 0.35);".to_owned()],
        El::new("Seam")
            .expr("at", "*at")
            .expr("on_change", "bind(&at)")
            .variant("axis", *axis)
            .expr_if(*narrowed, "min", "0.25")
            .expr_if(*narrowed, "max", "0.75")
            .expr(
                "start",
                format!("html! {{ {} }}", text_node(&text.of("Start pane"))),
            )
            .expr(
                "end",
                format!("html! {{ {} }}", text_node(&text.of("End pane"))),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <Seam class={class::SEAM_BOX} at={*at} on_change={bind(&at)}
                on_settle={bind(&settled)} axis={*axis} {min} {max}
                start={html! { <div class={class::SAMPLE}>{ text.of("Start pane") }</div> }}
                end={html! { <div class={class::SAMPLE}>{ text.of("End pane") }</div> }} />
        </Demo>
    }
}

#[derive(Properties, PartialEq)]
struct MeasuredProps {
    text: Text,
}

/// Its own node, as the demo shows it in both frames.
#[function_component]
fn Measured(props: &MeasuredProps) -> Html {
    let node = use_node_ref();
    let width = use_width(&node);

    html! {
        <Stack gap={Gap::Snug}>
            <div ref={node} class={class::SAMPLE}>{ props.text.of("Measured") }</div>
            <span class={classes!(class::READOUT, sockel::class::NUMERIC)}>
                { format!("{width:.0}px wide") }
            </span>
        </Stack>
    }
}

#[function_component]
pub fn WidthPage() -> Html {
    let text = use_state(|| Text::Short);

    let controls = html! {
        <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
    };

    let code = snippet(
        &[
            "let node = use_node_ref();".to_owned(),
            "let width = use_width(&node);".to_owned(),
        ],
        El::new("div")
            .expr("ref", "node")
            .line("{ format!(\"{width:.0}px wide\") }"),
    );

    html! {
        <Demo {controls} {code}>
            <Measured text={*text} />
        </Demo>
    }
}
