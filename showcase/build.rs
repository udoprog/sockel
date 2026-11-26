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
    Ok(())
}

struct Var<'a> {
    name: &'a str,
    default: &'a str,
    doc: String,
}

/// The reference page's data from `_tokens.scss`: every `!default` variable
/// with the `///` lines above it, grouped by the `////` headings, and the
/// private lists of breakpoints and omittable parts.
fn reference(source: &str, emitted: &str) -> Result<String, Box<dyn Error>> {
    let mut sections: Vec<(&str, Vec<Var>)> = Vec::new();
    let mut doc: Vec<&str> = Vec::new();
    let mut undocumented = Vec::new();

    for (number, line) in source.lines().enumerate() {
        if let Some(title) = line.strip_prefix("//// ") {
            if !sections.iter().any(|(t, _)| *t == title) {
                sections.push((title, Vec::new()));
            }
            continue;
        }

        if let Some(text) = line.strip_prefix("///") {
            doc.push(text.trim());
            continue;
        }

        if let Some((name, default)) = variable(line) {
            if doc.is_empty() {
                undocumented.push(name);
            }

            let Some((_, vars)) = sections.last_mut() else {
                return Err(format!("${name} comes before any `////` heading").into());
            };

            vars.push(Var {
                name,
                default,
                doc: doc.join(" "),
            });
            doc.clear();
            continue;
        }

        if !doc.is_empty() {
            return Err(format!(
                "_tokens.scss:{}: a `///` comment documents no variable",
                number + 1
            )
            .into());
        }
    }

    if !undocumented.is_empty() {
        return Err(format!(
            "variables without a `///` line above them: ${}",
            undocumented.join(", $")
        )
        .into());
    }

    let mut out = String::new();
    writeln!(out, "pub const SECTIONS: &[Section] = &[")?;
    for (title, vars) in &sections {
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
