//! [`FEATURES`] against plinth's Cargo.toml and its partials.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{Config, FEATURES, SOURCES, Sources, class_names};

/// Partials in every build, whatever the features.
const ALWAYS: &[&str] = &[
    "index",
    "tokens",
    "hidden",
    "hooks",
    "menu-surface",
    "utilities",
];

/// plinth's `[features]`, each with the features it names.
fn cargo_features() -> BTreeMap<String, Vec<String>> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml");
    let manifest = std::fs::read_to_string(manifest).unwrap();
    let section = manifest.split("[features]\n").nth(1).unwrap();
    let section = section.split("\n[").next().unwrap();
    section
        .lines()
        .filter_map(|line| line.split_once(" = "))
        .map(|(name, deps)| {
            let deps = deps.trim_matches(['[', ']']).split(", ");
            let deps = deps.filter(|dep| !dep.is_empty());
            let deps = deps.map(|dep| dep.trim_matches('"').to_owned()).collect();
            (name.to_owned(), deps)
        })
        .collect()
}

fn closure(features: &BTreeMap<String, Vec<String>>, name: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut todo = vec![name.to_owned()];
    while let Some(name) = todo.pop() {
        if found.insert(name.clone()) {
            todo.extend(features[&name].iter().cloned());
        }
    }
    found
}

fn source(partial: &str) -> &'static str {
    let name = format!("_{partial}.scss");
    SOURCES.iter().find(|(file, _)| *file == name).unwrap().1
}

fn partial_classes(partial: &str) -> BTreeSet<String> {
    let options = grass::Options::default()
        .fs(&Sources { index: None })
        .load_path("");
    let css = grass::from_string(format!("@use \"plinth/{partial}\";\n"), &options).unwrap();
    class_names(&css).into_iter().collect()
}

/// A partial's classes less those of the partials it loads.
fn own_classes(partial: &str) -> BTreeSet<String> {
    let mut classes = partial_classes(partial);
    for line in source(partial).lines() {
        let used = line
            .strip_prefix("@use \"")
            .and_then(|rest| rest.split('"').next());
        if let Some(used) = used.filter(|used| !used.starts_with("sass:")) {
            for class in partial_classes(used) {
                classes.remove(&class);
            }
        }
    }
    classes
}

#[test]
fn every_feature_is_cargo_s_and_full_has_them_all() {
    let cargo = cargo_features();
    let mapped: BTreeSet<_> = FEATURES.iter().map(|(name, _)| name.to_string()).collect();
    let declared: BTreeSet<_> = cargo
        .keys()
        .filter(|name| !["default", "full"].contains(&name.as_str()))
        .cloned()
        .collect();
    assert_eq!(mapped, declared);
    assert_eq!(cargo["default"], ["full"]);
    assert_eq!(
        cargo["full"].iter().cloned().collect::<BTreeSet<_>>(),
        mapped
    );
}

#[test]
fn every_partial_has_one_feature_or_is_always_in() {
    for (file, _) in SOURCES {
        let partial = &file[1..file.len() - ".scss".len()];
        let owners = FEATURES
            .iter()
            .filter(|(_, partials)| partials.contains(&partial));
        let expected = usize::from(!ALWAYS.contains(&partial));
        assert_eq!(owners.count(), expected, "{partial}");
    }
}

#[test]
fn all_features_build_the_whole_stylesheet() {
    let names = FEATURES.iter().map(|(name, _)| *name);
    let all = Config::new().features(names).build().unwrap();
    assert_eq!(all, Config::new().build().unwrap());
    assert!(Config::new().features(["nope"]).build().is_err());
}

#[test]
fn each_feature_s_css_is_its_own() {
    let cargo = cargo_features();
    let always: BTreeSet<_> = ALWAYS
        .iter()
        .filter(|partial| **partial != "index")
        .flat_map(|partial| partial_classes(partial))
        .collect();
    let none = Config::new().features([] as [&str; 0]).build().unwrap();
    for (feature, _) in FEATURES {
        let enabled = closure(&cargo, feature);
        let mut allowed = always.clone();
        allowed.extend(none.plinth_classes.iter().cloned());
        for (_, partials) in FEATURES.iter().filter(|(name, _)| enabled.contains(*name)) {
            for partial in *partials {
                allowed.extend(own_classes(partial));
            }
        }
        let css = Config::new().features(&enabled).build().unwrap().css;
        let foreign: Vec<_> = class_names(&css)
            .into_iter()
            .filter(|class| !allowed.contains(class))
            .collect();
        assert!(foreign.is_empty(), "{feature}: {foreign:?}");
    }
}
