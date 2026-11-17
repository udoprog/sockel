//! Every component keeps the classes passed in its `class` prop beside its own.

use plinth::*;
use yew::LocalServerRenderer;
use yew::prelude::*;

const MINE: &str = "app-sidebar";

fn render<C: BaseComponent>(props: C::Properties) -> String {
    futures::executor::block_on(
        LocalServerRenderer::<C>::with_props(props)
            .hydratable(false)
            .render(),
    )
}

fn keeps(html: String, own: &str) {
    let expected = format!("class=\"{own}\"");
    assert!(html.contains(&expected), "{expected} in {html}");
}

#[test]
fn layout_primitives_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<Stack>(yew::props!(StackProps { class: mine() })),
        &own(class::STACK),
    );
    keeps(
        render::<Cluster>(yew::props!(ClusterProps { class: mine() })),
        &own(class::CLUSTER),
    );
    keeps(
        render::<Card>(yew::props!(CardProps { class: mine() })),
        &own(class::CARD),
    );
    keeps(
        render::<Toolbar>(yew::props!(ToolbarProps {
            label: "Tools",
            class: mine()
        })),
        &own(class::TOOLBAR),
    );
    keeps(
        render::<ToolbarSpacer>(yew::props!(ToolbarSpacerProps { class: mine() })),
        &own(class::TOOLBAR_SPACER),
    );
}

#[test]
fn components_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<ActionGroup>(yew::props!(ActionGroupProps { class: mine() })),
        &own(class::ACTION_GROUP),
    );
    keeps(
        render::<IconButton>(yew::props!(IconButtonProps { class: mine() })),
        &format!("{MINE} {}", class::ICONED),
    );
    keeps(
        render::<PageHead>(yew::props!(HeadProps {
            title: "Title",
            class: mine()
        })),
        &own(class::PAGE_HEAD),
    );
    keeps(
        render::<SectionHead>(yew::props!(HeadProps {
            title: "Title",
            class: mine()
        })),
        &own(class::SECTION_HEAD),
    );
    keeps(
        render::<LibraryToolbar>(yew::props!(LibraryToolbarProps {
            value: "",
            label: "Search",
            testid: "search",
            on_input: Callback::noop(),
            class: mine(),
        })),
        &own(class::LIBRARY_TOOLBAR),
    );
}

#[test]
fn icon_keeps_a_passed_class() {
    keeps(
        render::<Icon>(yew::props!(IconProps {
            glyph: Glyph::Close,
            class: classes!(MINE)
        })),
        &format!("{MINE} {}", class::GLYPH),
    );
}

#[test]
fn media_keeps_a_passed_class() {
    let mine = || classes!(MINE);
    keeps(
        render::<Image>(yew::props!(ImageProps {
            ratio: Ratio::Poster,
            class: mine()
        })),
        &format!("{MINE} {} {} {}", class::IMAGE, class::POSTER, class::EMPTY),
    );
    keeps(
        render::<Avatar>(yew::props!(AvatarProps {
            name: "Ada Lovelace",
            class: mine()
        })),
        &format!("{MINE} {}", class::AVATAR),
    );
}

#[test]
fn toggles_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<Checkbox>(yew::props!(CheckboxProps {
            checked: false,
            on_change: Callback::noop(),
            class: mine()
        })),
        &own(class::CHECKBOX),
    );
    keeps(
        render::<Switch>(yew::props!(SwitchProps {
            on: false,
            on_change: Callback::noop(),
            class: mine()
        })),
        &own(class::SWITCH),
    );
    keeps(
        render::<Choice>(yew::props!(ChoiceProps {
            options: vec![ChoiceOption::new("a", "A")],
            on_change: Callback::noop(),
            label: "Pick",
            class: mine()
        })),
        &own(class::CHOICE),
    );
    keeps(
        render::<ChipToggle>(yew::props!(ChipToggleProps {
            on: false,
            on_toggle: Callback::noop(),
            class: mine()
        })),
        &own(class::CHIP),
    );
    keeps(
        render::<ChipList>(yew::props!(ChipListProps {
            label: "Filters",
            class: mine()
        })),
        &own(class::CHIP_LIST),
    );
}

#[test]
fn fields_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    let field = || FieldProps {
        id: "x".into(),
        label: html! { "X" },
        required: false,
        help: None,
        hint: None,
        error: None,
        reserve_error: false,
        disabled: false,
        busy: false,
        class: mine(),
        testid: None,
        children: Html::default(),
    };
    keeps(render::<Field>(field()), &own(class::FIELD));
    keeps(render::<FormRow>(field()), &own(class::FORM_ROW));
    keeps(
        render::<FormRows>(yew::props!(FormRowsProps { class: mine() })),
        &own(class::FORM_ROWS),
    );
    keeps(
        render::<Fieldset>(yew::props!(FieldsetProps {
            legend: html! { "Legend" },
            class: mine()
        })),
        &own(class::FIELDSET),
    );
    keeps(
        render::<FieldCheck>(yew::props!(FieldCheckProps {
            label: html! { "Label" },
            class: mine()
        })),
        &own(class::FIELD_CHECK),
    );
}

#[test]
fn marks_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<Pill>(yew::props!(PillProps { class: mine() })),
        &own(class::PILL),
    );
    keeps(
        render::<Badge>(yew::props!(BadgeProps { class: mine() })),
        &own(class::BADGE),
    );
    keeps(
        render::<Dot>(yew::props!(DotProps { class: mine() })),
        &format!("{} {} {MINE}", class::DOT, class::ON),
    );
    keeps(
        render::<Count>(yew::props!(CountProps {
            value: 3,
            class: mine()
        })),
        &own(class::COUNT),
    );
    keeps(
        render::<StatusLabel>(yew::props!(StatusLabelProps {
            states: vec![],
            class: mine()
        })),
        &own(class::STATUS_LABEL),
    );
}

#[test]
fn rows_and_grids_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<RowList>(yew::props!(RowListProps { class: mine() })),
        &own(class::ROW_LIST),
    );
    keeps(
        render::<RowListItem>(yew::props!(RowListItemProps {
            name: html! { "Name" },
            class: mine()
        })),
        &own(class::ROW_LIST_ITEM),
    );
    keeps(
        render::<PickRow>(yew::props!(PickRowProps {
            name: html! { "Name" },
            class: mine()
        })),
        &own(class::PICK_ROW),
    );
    keeps(
        render::<SelectionBar>(yew::props!(SelectionBarProps {
            count: 0usize,
            label: "None",
            class: mine()
        })),
        &format!("{} {} {MINE}", class::SELECTION_BAR, class::IDLE),
    );
    keeps(
        render::<ActionRow>(yew::props!(ActionRowProps { class: mine() })),
        &own(class::ACTION_ROW),
    );
    keeps(
        render::<CardGrid>(yew::props!(CardGridProps { class: mine() })),
        &own(class::CARD_GRID),
    );
}

#[test]
fn an_overlay_action_row_renders_its_modifier_and_stays_focusable() {
    let html = render::<ActionRow>(yew::props!(ActionRowProps {
        overlay: true,
        children: html! { <button>{ "Edit" }</button> },
    }));
    assert!(html.contains(&format!("{} {}", class::ACTION_ROW, class::OVERLAY)));
    assert!(!html.contains("tabindex"), "{html}");
    assert!(!html.contains("inert"), "{html}");
}

#[test]
fn menus_keep_a_passed_class() {
    let mine = || classes!(MINE);
    let own = |class: &str| format!("{class} {MINE}");
    keeps(
        render::<MenuItem>(yew::props!(MenuItemProps { class: mine() })),
        &own(class::MENU_ITEM),
    );
    keeps(
        render::<MenuSeparator>(yew::props!(MenuPartProps { class: mine() })),
        &own(class::MENU_SEPARATOR),
    );
    keeps(
        render::<RowActions>(yew::props!(RowActionsProps { class: mine() })),
        &own(class::ROW_ACTIONS),
    );
    keeps(
        render::<SplitButton>(yew::props!(SplitButtonProps {
            text: html! { "Save" },
            onclick: Callback::noop(),
            class: mine()
        })),
        &own(class::SPLIT_BUTTON),
    );
    let open = render::<Menu>(yew::props!(MenuProps {
        open: Some(true),
        class: mine()
    }));
    assert!(
        open.contains(&format!("{} {MINE}\"", class::MENU)),
        "{open}"
    );
}

#[test]
fn loading_components_keep_a_passed_class() {
    let mine = || classes!(MINE);
    keeps(
        render::<Spinner>(yew::props!(SpinnerProps { class: mine() })),
        &format!("{} {MINE}", class::SPINNER),
    );
    keeps(
        render::<Skeleton>(yew::props!(SkeletonProps { class: mine() })),
        &format!("{} {MINE}", class::SKELETON),
    );
    keeps(
        render::<Skeleton>(yew::props!(SkeletonProps {
            count: 2,
            class: mine()
        })),
        &format!("{} {MINE}", class::SKELETON_GROUP),
    );
    keeps(
        render::<Loadable>(yew::props!(LoadableProps { class: mine() })),
        &format!("{} {MINE}", class::LOADABLE),
    );
}

#[test]
fn a_select_keeps_a_passed_class() {
    let html = render::<Select<u32>>(yew::props!(SelectProps<u32> {
        options: vec![SelectOption::new(1, "One")],
        class: classes!(MINE),
    }));
    assert!(
        html.contains(&format!("{} {MINE}\"", class::SELECT)),
        "{html}"
    );
}
