use std::error::Error;
use std::fmt::Write as _;
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    sockel_build::Config::new()
        .local("style/")
        .prefix("showcase-")
        .compile();

    let out_dir = env::var("OUT_DIR")?;

    // Where scripts/trug-css.sh finds this build's CSS.
    let out = Path::new(&out_dir).join("style.css");
    let known = Path::new("../target/showcase-css/style.css");
    if let Some(parent) = known.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(out, known)?;

    let style = env::var(sockel_build::STYLE_VAR)?;
    let tokens = Path::new(&style).join("_tokens.scss");
    println!("cargo::rerun-if-changed={}", tokens.display());
    let source = fs::read_to_string(&tokens)?;
    let mut emitted = String::new();
    for entry in fs::read_dir(&style)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "scss") {
            println!("cargo::rerun-if-changed={}", path.display());
            emitted.push_str(&fs::read_to_string(&path)?);
        }
    }

    let reference = reference(&source, &emitted)?;
    fs::write(Path::new(&out_dir).join("reference.rs"), reference)?;

    let manifest = Path::new(&style).join("../Cargo.toml");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let features = features(&fs::read_to_string(&manifest)?)?;
    fs::write(Path::new(&out_dir).join("features.rs"), features)?;
    Ok(())
}

struct Var<'a> {
    name: &'a str,
    default: &'a str,
    doc: String,
}

struct Mixin<'a> {
    name: &'a str,
    args: &'a str,
    content: bool,
    doc: String,
}

enum Item<'a> {
    Var(&'a str, &'a str),
    Mixin(&'a str, &'a str),
}

/// Items grouped by the `////` heading they follow, in the order each
/// group's first item appears.
struct Groups<'a, T>(Vec<(&'a str, Vec<T>)>);

impl<'a, T> Groups<'a, T> {
    fn push(&mut self, title: &'a str, item: T) {
        match self.0.iter_mut().find(|(t, _)| *t == title) {
            Some((_, items)) => items.push(item),
            None => self.0.push((title, vec![item])),
        }
    }
}

/// The reference pages' data from `_tokens.scss`: every `!default` variable
/// and public mixin with the `///` lines above it, grouped by the `////`
/// headings, and the private lists of breakpoints and omittable parts.
fn reference(source: &str, emitted: &str) -> Result<String, Box<dyn Error>> {
    let mut sections = Groups::<Var>(Vec::new());
    let mut mixins = Groups::<Mixin>(Vec::new());
    let mut title = None;
    let mut doc: Vec<&str> = Vec::new();
    let mut undocumented = Vec::new();
    let mut lines = source.lines().enumerate();

    while let Some((number, line)) = lines.next() {
        if let Some(heading) = line.strip_prefix("//// ") {
            title = Some(heading);
            continue;
        }

        if let Some(text) = line.strip_prefix("///") {
            doc.push(text.trim());
            continue;
        }

        let item = match (variable(line), mixin(line)) {
            (Some((name, default)), _) => Item::Var(name, default),
            (_, Some((name, args))) => Item::Mixin(name, args),
            _ => {
                if !doc.is_empty() {
                    return Err(format!(
                        "_tokens.scss:{}: a `///` comment documents no variable or mixin",
                        number + 1
                    )
                    .into());
                }
                continue;
            }
        };

        let label = match item {
            Item::Var(name, _) => format!("${name}"),
            Item::Mixin(name, _) => format!("@mixin {name}"),
        };

        if doc.is_empty() {
            undocumented.push(label.clone());
        }

        let Some(title) = title else {
            return Err(format!("{label} comes before any `////` heading").into());
        };

        let text = doc.join(" ");
        doc.clear();

        match item {
            Item::Var(name, default) => sections.push(
                title,
                Var {
                    name,
                    default,
                    doc: text,
                },
            ),
            Item::Mixin(name, args) => {
                let content = lines
                    .by_ref()
                    .map(|(_, l)| l)
                    .take_while(|l| *l != "}")
                    .filter(|l| l.trim() == "@content;")
                    .count()
                    > 0;
                mixins.push(
                    title,
                    Mixin {
                        name,
                        args,
                        content,
                        doc: text,
                    },
                );
            }
        }
    }

    if !undocumented.is_empty() {
        return Err(format!(
            "_tokens.scss: without a `///` line above them: {}",
            undocumented.join(", ")
        )
        .into());
    }

    let mut out = String::new();
    writeln!(out, "pub const MIXINS: &[MixinSection] = &[")?;
    for (title, items) in &mixins.0 {
        writeln!(out, "    MixinSection {{ title: {title:?}, mixins: &[")?;
        for m in items {
            writeln!(
                out,
                "        Mixin {{ name: {:?}, args: {:?}, content: {:?}, doc: {:?} }},",
                m.name, m.args, m.content, m.doc
            )?;
        }
        writeln!(out, "    ] }},")?;
    }
    writeln!(out, "];")?;

    writeln!(out, "pub const SECTIONS: &[Section] = &[")?;
    for (title, vars) in &sections.0 {
        writeln!(out, "    Section {{ title: {title:?}, vars: &[")?;
        for var in vars {
            let property = format!("--{}", var.name);
            let property = emitted
                .contains(&format!("{property}: #{{${}}}", var.name))
                .then_some(property);
            writeln!(
                out,
                "        Var {{ name: {:?}, default: {:?}, doc: {:?}, property: {property:?} }},",
                var.name, var.default, var.doc
            )?;
        }
        writeln!(out, "    ] }},")?;
    }
    writeln!(out, "];")?;

    let breakpoints = block(source, "$-default-breakpoints: (")?;
    writeln!(out, "pub const BREAKPOINTS: &[(&str, &str)] = &[")?;
    for line in breakpoints {
        let Some((name, width)) = line.trim().trim_end_matches(',').split_once(": ") else {
            continue;
        };
        writeln!(out, "    ({:?}, {width:?}),", name.trim_matches('"'))?;
    }
    writeln!(out, "];")?;

    let owners = block(source, "$-feature-parts: (")?;
    writeln!(out, "pub const PARTS: &[(&str, &[&str])] = &[")?;
    for part in block(source, "$-parts: (")?.iter().flat_map(|l| quoted(l)) {
        let features = owners
            .iter()
            .find_map(|l| {
                let (key, rest) = l.trim().split_once(": ")?;
                (key.trim_matches('"') == part).then(|| quoted(rest))
            })
            .unwrap_or_default();
        writeln!(out, "    ({part:?}, &{features:?}),")?;
    }
    writeln!(out, "];")?;
    Ok(out)
}

/// A public `@mixin name(args) {` line: its name and its arguments, empty
/// when it takes none.
fn mixin(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("@mixin ")?.strip_suffix(" {")?;
    let (name, args) = match rest.split_once('(') {
        Some((name, args)) => (name, args.strip_suffix(')')?),
        None => (rest, ""),
    };
    name.starts_with(|c: char| c.is_ascii_lowercase())
        .then_some((name, args))
}

/// The features page's data from sockel's `Cargo.toml`: each of its
/// `[features]` but `default`, the features it enables and its line in
/// `[package.metadata.sockel.features]`, which every feature must have.
fn features(manifest: &str) -> Result<String, Box<dyn Error>> {
    let docs = table(manifest, "[package.metadata.sockel.features]")?;
    let features = table(manifest, "[features]")?;
    let mut out = String::new();
    let mut undocumented = Vec::new();
    writeln!(out, "pub const FEATURES: &[Feature] = &[")?;

    for &(name, value) in &features {
        if name == "default" {
            continue;
        }

        let enables = quoted(value);

        let Some((_, doc)) = docs.iter().find(|(n, _)| *n == name) else {
            undocumented.push(name);
            continue;
        };

        let Some(doc) = doc.strip_prefix('"').and_then(|d| d.strip_suffix('"')) else {
            return Err(
                format!("the description of feature `{name}` is not one plain string").into(),
            );
        };

        writeln!(
            out,
            "    Feature {{ name: {name:?}, enables: &{enables:?}, doc: {doc:?} }},"
        )?;
    }

    writeln!(out, "];")?;

    if !undocumented.is_empty() {
        return Err(format!(
            "features without a line in [package.metadata.sockel.features]: {}",
            undocumented.join(", ")
        )
        .into());
    }

    if let Some((name, _)) = docs
        .iter()
        .find(|(doc, _)| !features.iter().any(|(name, _)| name == doc))
    {
        return Err(
            format!("[package.metadata.sockel.features] describes no feature `{name}`").into(),
        );
    }

    Ok(out)
}

/// The `key = value` lines of a TOML table, up to the next table.
fn table<'a>(manifest: &'a str, header: &str) -> Result<Vec<(&'a str, &'a str)>, Box<dyn Error>> {
    let mut lines = manifest.lines().skip_while(|l| *l != header);
    if lines.next().is_none() {
        return Err(format!("Cargo.toml has no `{header}`").into());
    }
    Ok(lines
        .take_while(|l| !l.starts_with('['))
        .filter_map(|l| l.split_once(" = "))
        .collect())
}

/// A public `$name: default !default;` line.
fn variable(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix('$')?;
    let (name, value) = rest.split_once(": ")?;
    let default = value.strip_suffix(" !default;")?;
    let public = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    public.then_some((name, default))
}

/// The lines between `start` and the `);` that closes it.
fn block<'a>(source: &'a str, start: &str) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let mut lines = source.lines().skip_while(|l| !l.starts_with(start));
    if lines.next().is_none() {
        return Err(format!("_tokens.scss has no `{start}`").into());
    }
    Ok(lines.take_while(|l| !l.starts_with(");")).collect())
}

/// The double-quoted strings in a line.
fn quoted(line: &str) -> Vec<&str> {
    line.split('"').skip(1).step_by(2).collect()
}
