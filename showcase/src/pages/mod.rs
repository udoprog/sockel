mod actions;
mod heads;
mod layout;
mod library;

use yew::prelude::*;

use plinth::Head;

pub struct Page {
    /// The page's address, after the `#`.
    pub id: &'static str,
    pub title: &'static str,
    /// What the page is about, under its title.
    pub about: &'static str,
    /// The names exported from plinth that the page shows (its component,
    /// props and enums).
    pub covers: &'static [&'static str],
    pub render: fn() -> Html,
}

/// Every page, in the navigation's order. A name plinth exports must be in
/// some page's `covers`, which the test below checks.
pub const PAGES: &[Page] = &[
    Page {
        id: "stack",
        title: "Stack",
        about: "A column, its blocks a gap apart.",
        covers: &["Stack", "StackProps", "Gap"],
        render: || html! { <layout::StackPage /> },
    },
    Page {
        id: "cluster",
        title: "Cluster",
        about: "A row that wraps, its items centred.",
        covers: &["Cluster", "ClusterProps", "Justify", "Align"],
        render: || html! { <layout::ClusterPage /> },
    },
    Page {
        id: "action-group",
        title: "ActionGroup",
        about: "The buttons that end a form, a panel or a section, the primary first.",
        covers: &["ActionGroup", "ActionGroupProps", "action_group"],
        render: || html! { <actions::ActionGroupPage /> },
    },
    Page {
        id: "icon-button",
        title: "IconButton",
        about: "A button that leads with (or ends in) an icon the app supplies. Click it; tab to it.",
        covers: &["IconButton", "IconButtonProps", "icon_button"],
        render: || html! { <actions::IconButtonPage /> },
    },
    Page {
        id: "toolbar",
        title: "Toolbar",
        about: "One line of controls; a ToolbarSpacer pushes the rest to its end. Tab through its buttons.",
        covers: &["Toolbar", "ToolbarProps", "ToolbarSpacer"],
        render: || html! { <layout::ToolbarPage /> },
    },
    Page {
        id: "card",
        title: "Card",
        about: "A surface lifted off the page.",
        covers: &["Card", "CardProps", "Inset"],
        render: || html! { <layout::CardPage /> },
    },
    Page {
        id: "page-head",
        title: "PageHead",
        about: "A page's title with its help, and its actions.",
        covers: &["PageHead", "HeadProps", "Head"],
        render: || html! { <heads::HeadPage kind={Head::Page} /> },
    },
    Page {
        id: "section-head",
        title: "SectionHead",
        about: "A section's title with its help, and its actions.",
        covers: &["SectionHead", "HeadProps", "Head"],
        render: || html! { <heads::HeadPage kind={Head::Section} /> },
    },
    Page {
        id: "library-toolbar",
        title: "LibraryToolbar",
        about: "A list's search field and its filters: click a filter to change its text, and the field keeps its width.",
        covers: &["LibraryToolbar", "LibraryToolbarProps"],
        render: || html! { <library::LibraryToolbarPage /> },
    },
    Page {
        id: "classes",
        title: "Class functions",
        about: "The classes of a stack, cluster, toolbar and card, for an element that cannot be the component.",
        covers: &["stack", "cluster", "toolbar", "card"],
        render: || html! { <layout::ClassesPage /> },
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::PAGES;

    /// The names in `src/lib.rs`'s `pub use` items.
    fn exported() -> BTreeSet<&'static str> {
        let lib = include_str!("../../../src/lib.rs");
        let mut names = BTreeSet::new();

        for item in lib.split("pub use ").skip(1) {
            let item = &item[..item.find(';').expect("a `pub use` ends with `;`")];
            let list = match (item.find('{'), item.rfind('}')) {
                (Some(open), Some(close)) => &item[open + 1..close],
                _ => item.rsplit("::").next().unwrap_or(item),
            };

            for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
                // `Name as Alias` exports the alias.
                names.insert(name.rsplit(' ').next().unwrap_or(name));
            }
        }

        names
    }

    #[test]
    fn every_export_has_a_showcase_page() {
        let exported = exported();
        let covered: BTreeSet<&str> = PAGES
            .iter()
            .flat_map(|p| p.covers.iter().copied())
            .collect();

        assert!(exported.contains("Stack"), "the parser found {exported:?}");
        let missing: Vec<_> = exported.difference(&covered).collect();
        assert!(
            missing.is_empty(),
            "exported from plinth without a showcase page: {missing:?}"
        );
        let stale: Vec<_> = covered.difference(&exported).collect();
        assert!(
            stale.is_empty(),
            "showcase pages cover names plinth does not export: {stale:?}"
        );
    }

    #[test]
    fn page_ids_are_unique() {
        let ids: BTreeSet<_> = PAGES.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PAGES.len());
    }
}
