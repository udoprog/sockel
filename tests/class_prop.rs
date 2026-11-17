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
