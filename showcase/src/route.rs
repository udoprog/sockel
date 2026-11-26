//! The address: `#<component>` or `#<page id>` names a page, and
//! `#<page id>/<section>` a section on it.

use crate::pages::{FALLBACK, PAGES, Page};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Route {
    /// The index into `PAGES`.
    pub page: usize,
    /// The id of the element to scroll to, when the address names a section.
    pub target: Option<String>,
}

/// The route an address's hash names: a page id first (the older links), then
/// a component sockel exports, as written and then in any case.
pub fn parse(hash: &str) -> Route {
    let hash = hash.trim_start_matches('#');
    let (name, section) = match hash.split_once('/') {
        Some((name, section)) => (name, Some(section)),
        None => (hash, None),
    };
    let found = find(name);
    let page = found
        .or_else(|| PAGES.iter().position(|p| p.id == FALLBACK))
        .unwrap_or_default();
    let target = found
        .and(section)
        .filter(|s| !s.is_empty())
        .map(|_| hash.to_owned());
    Route { page, target }
}

fn find(name: &str) -> Option<usize> {
    let lower = name.to_lowercase();
    PAGES
        .iter()
        .position(|p| p.id == name)
        .or_else(|| PAGES.iter().position(|p| p.covers.contains(&name)))
        .or_else(|| {
            PAGES
                .iter()
                .position(|p| p.covers.iter().any(|c| c.to_lowercase() == lower))
        })
}

/// A page's own link: its component's name when the page is titled by one,
/// else its id.
pub fn slug(page: &Page) -> &'static str {
    if page.covers.contains(&page.title) {
        page.title
    } else {
        page.id
    }
}

/// The id, and link, of a section on a page: its title in lower case, words
/// joined by `-`.
pub fn section(page: &str, title: &str) -> String {
    let words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    format!("{page}/{}", words.join("-"))
}

/// Whether a page matches the navigation's filter: its title or a name it
/// covers holds the text, in any case.
pub fn matches(page: &Page, filter: &str) -> bool {
    let filter = filter.trim().to_lowercase();
    filter.is_empty()
        || page.title.to_lowercase().contains(&filter)
        || page
            .covers
            .iter()
            .any(|name| name.to_lowercase().contains(&filter))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(id: &str) -> usize {
        PAGES.iter().position(|p| p.id == id).unwrap()
    }

    #[test]
    fn every_page_link_leads_back_to_it() {
        for (index, p) in PAGES.iter().enumerate() {
            assert_eq!(parse(&format!("#{}", slug(p))).page, index, "{}", p.id);
            assert_eq!(parse(&format!("#{}", p.id)).page, index, "{}", p.id);
        }
    }

    #[test]
    fn names_page_ids_and_sections() {
        assert_eq!(parse("#chip").page, page("chip"));
        assert_eq!(parse("#Chip").page, page("chip-item"));
        assert_eq!(parse("#ButtonVariant").page, page("button"));
        assert_eq!(parse("#buttonvariant").page, page("button"));
        assert_eq!(parse("").page, page(FALLBACK));
        assert_eq!(parse("#nothing-here").page, page(FALLBACK));

        let route = parse("#intro/getting-started");
        assert_eq!(route.page, page("intro"));
        assert_eq!(route.target.as_deref(), Some("intro/getting-started"));
        assert_eq!(parse("#intro/").target, None);
        assert_eq!(parse("#nothing/here").target, None);
    }

    #[test]
    fn sections_are_slugged() {
        assert_eq!(
            section("intro", "Without and with sockel"),
            "intro/without-and-with-sockel"
        );
        assert_eq!(
            section("intro", "Bring your own colours!"),
            "intro/bring-your-own-colours"
        );
    }

    #[test]
    fn filters_by_title_and_covers() {
        let button = &PAGES[page("button")];
        assert!(matches(button, ""));
        assert!(matches(button, " BUTT "));
        assert!(matches(button, "variant"));
        assert!(!matches(button, "zzz"));
    }
}
