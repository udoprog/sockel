//! [`FEATURES`] against sockel's Cargo.toml and its partials.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{Config, FEATURE_PARTS, FEATURES, STYLE, class_names, sockel, test_sources};

/// Partials in every build, whatever the features.
const ALWAYS: &[&str] = &[
    "index",
    "tokens",
    "hidden",
    "hooks",
    "menu-surface",
    "utilities",
];

/// sockel's `[features]`, each with the features it names.
fn cargo_features() -> BTreeMap<String, Vec<String>> {
    let manifest = Path::new(STYLE).join("../Cargo.toml");
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

fn source(partial: &str) -> String {
    std::fs::read_to_string(Path::new(STYLE).join(format!("_{partial}.scss"))).unwrap()
}

fn partial_classes(partial: &str) -> BTreeSet<String> {
    let sources = test_sources();
    let options = sasso::Options::new().with_importer(&sources);
    let css = sasso::compile(&format!("@use \"sockel/{partial}\";\n"), &options).unwrap();
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
    for entry in std::fs::read_dir(STYLE).unwrap() {
        let file = entry.unwrap().file_name().into_string().unwrap();
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
    let all = sockel().features(names).build().unwrap();
    assert_eq!(all, sockel().build().unwrap());
    assert!(sockel().features(["nope"]).build().is_err());
}

#[test]
fn each_feature_s_css_is_its_own() {
    let cargo = cargo_features();
    let always: BTreeSet<_> = ALWAYS
        .iter()
        .filter(|partial| **partial != "index")
        .flat_map(|partial| partial_classes(partial))
        .collect();
    let none = sockel().features([] as [&str; 0]).build().unwrap();
    for (feature, _) in FEATURES {
        let enabled = closure(&cargo, feature);
        let mut allowed = always.clone();
        allowed.extend(none.sockel_classes.iter().cloned());
        for (_, partials) in FEATURES.iter().filter(|(name, _)| enabled.contains(*name)) {
            for partial in *partials {
                allowed.extend(own_classes(partial));
            }
        }
        let css = sockel().features(&enabled).build().unwrap().css;
        let foreign: Vec<_> = class_names(&css)
            .into_iter()
            .filter(|class| !allowed.contains(class))
            .collect();
        assert!(foreign.is_empty(), "{feature}: {foreign:?}");
    }
}

/// The custom properties `css` defines (`--x: ...`) and reads (`var(--x`).
fn custom_properties(css: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut defined, mut read) = (BTreeSet::new(), BTreeSet::new());
    for (at, _) in css.match_indices("--") {
        let rest = &css[at + 2..];
        let len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(rest.len());
        let name = rest[..len].to_owned();
        if rest[len..].starts_with(':') {
            defined.insert(name);
        } else if css[..at].ends_with("var(") {
            read.insert(name);
        }
    }
    (defined, read)
}

#[test]
fn feature_parts_name_features() {
    for (part, features) in FEATURE_PARTS {
        for feature in *features {
            assert!(
                FEATURES.iter().any(|(name, _)| name == feature),
                "{part}: {feature}"
            );
        }
    }
}

#[test]
fn each_feature_defines_the_tokens_it_reads() {
    let cargo = cargo_features();
    let sources = test_sources();
    let options = sasso::Options::new().with_importer(&sources);
    let tokens = sasso::compile("@use \"sockel/tokens\";\n", &options).unwrap();
    let (full, _) = custom_properties(&tokens);
    for (feature, _) in FEATURES {
        let css = sockel()
            .features(closure(&cargo, feature))
            .build()
            .unwrap()
            .css;
        let (defined, read) = custom_properties(&css);
        let missing: Vec<_> = read
            .iter()
            .filter(|name| full.contains(*name) && !defined.contains(*name))
            .collect();
        assert!(missing.is_empty(), "{feature}: {missing:?}");
    }
}

#[test]
fn a_subset_leaves_out_its_parts_and_an_apps() {
    let css = |config: Config| config.build().unwrap().css;
    let layout = css(sockel().features(["layout"]));
    assert!(!layout.contains("--table-max-height:"));
    assert!(css(sockel()).contains("--table-max-height:"));
    let omitted = css(sockel()
        .features(["layout"])
        .with("omit", r#""layout.toolbar""#));
    assert!(!omitted.contains("--table-max-height:"));
    assert!(!omitted.contains("--band-chrome-gap:"));
    assert!(layout.contains("--band-chrome-gap:"));
}

#[test]
fn tokens_features_leave_out_the_feature_parts() {
    let cargo = cargo_features();
    let sources = test_sources();
    let options = sasso::Options::new().with_importer(&sources);
    let compile = |with: String| {
        let sass = format!("@use \"sockel/tokens\" with ({with});\n@use \"sockel/skeleton\";\n");
        sasso::compile(&sass, &options).unwrap()
    };
    for (feature, _) in FEATURES {
        let on = closure(&cargo, feature);
        let map: Vec<_> = on.iter().map(|name| format!("\"{name}\": true")).collect();
        let omit: Vec<_> = FEATURE_PARTS
            .iter()
            .filter(|(_, features)| !features.iter().any(|name| on.contains(*name)))
            .map(|(part, _)| format!("\"{part}\""))
            .collect();
        assert_eq!(
            compile(format!("$features: ({})", map.join(", "))),
            compile(format!("$omit: ({},)", omit.join(", "))),
            "{feature}"
        );
    }
    assert_eq!(
        compile("$features: ()".into()),
        compile(format!(
            "$omit: ({},)",
            FEATURE_PARTS
                .iter()
                .map(|(part, _)| format!("\"{part}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    );
}

#[test]
fn a_component_s_feature_leaves_out_its_siblings() {
    let css = |features: &[&str]| {
        let enabled = closure(&cargo_features(), features[0]);
        sockel().features(&enabled).build().unwrap().css
    };
    let switch = css(&["switch"]);
    assert!(switch.contains(".sockel-switch"));
    for sibling in [".sockel-checkbox", ".sockel-choice", ".sockel-chip"] {
        assert!(!switch.contains(sibling), "{sibling}");
    }
    let toasts = css(&["toasts"]);
    assert!(toasts.contains(".sockel-toast-host"));
    assert!(!toasts.contains(".sockel-save-bar"));
    let save_bar = css(&["save-bar"]);
    assert!(save_bar.contains(".sockel-save-bar"));
    assert!(!save_bar.contains(".sockel-toast"));
    for (whole, parts) in [
        (
            "toggles",
            [".sockel-checkbox", ".sockel-switch", ".sockel-choice"],
        ),
        (
            "toast",
            [".sockel-toast-host", ".sockel-save-bar", ".sockel-toast "],
        ),
    ] {
        let whole = css(&[whole]);
        for part in parts {
            assert!(whole.contains(part), "{part}");
        }
    }
}
