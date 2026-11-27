# sockel

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sockel.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/sockel/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/sockel/actions?query=branch%3Amain)

[Yew] components that hold still.

A button turning busy, a count gaining a digit, a field showing its error, a
menu opening: in most UIs each of these nudges the layout around it. With
sockel none of them does. Every component reserves the room its widest state
needs, and overlays float in a layer of their own, so an app's pages never
jitter.

**[See every component live on the documentation site](https://udoprog.github.io/sockel/)**:
each one has controls for its props, the code that renders it, and a probe
that checks nothing moves.

- **Styled out of the box.** Every component ships its Sass, and its variants
  are props, so markup and styles never drift apart.
- **Typed classes.** Your Sass and sockel's compile into one stylesheet, with a
  Rust constant per class: a misspelt class does not compile.
- **Pay for what you render.** Each component is a Cargo feature that gates
  its Rust and its CSS together.
- **Your colours.** sockel brings no palette. It reads seven colours from the
  app and derives the rest, in light or dark.
- **Translated.** No English in the code: every word comes from a catalog the
  app can extend.

<br>

## Getting started

An app depends on `sockel`, and builds one stylesheet (sockel's CSS and its
own Sass) with `sockel-build` in its `build.rs`. The blocks below are the
files of a complete app.

In `Cargo.toml`, pick the features the app renders (or leave out
`default-features` and `features` to get all of them):

<!-- check: Cargo.toml -->
```toml
[dependencies]
sockel = { version = "0.4", default-features = false, features = ["button", "layout", "overlay"] }
yew = { version = "0.23", features = ["csr"] }

[build-dependencies]
sockel-build = "0.4"

[dev-dependencies]
sockel-build = "0.4"
```

The app's Sass lives in `style/`, with an `_index.scss` that loads sockel's
tokens and mixins as `"sockel"`. It also sets the colours sockel reads:

<!-- check: style/_index.scss -->
```scss
@use "sockel" as p;

:root {
    --bg: #fafafa;
    --surface: #fff;
    --text: #1d1d1f;
    --muted: #6e6e73;
    --border: #d2d2d7;
    --accent: #0a66c2;
    --shadow-1: 0 1px 2px rgb(0 0 0 / 0.08);
}

.app-sidebar {
    padding: var(--space-4);

    @include p.below("phone") {
        padding: var(--space-2);
    }
}
```

The build is described once, in `style.rs` at the package root, so that the
build script and a test can share it:

<!-- check: style.rs -->
```rust
fn style() -> sockel_build::Config {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
}
```

<!-- check: build.rs -->
```rust
include!("style.rs");

fn main() {
    style().compile();
}
```

The build writes `style.css` and `classes.rs` to `OUT_DIR`. The app mounts the
stylesheet once at its root with
[`Styles`](https://docs.rs/sockel/latest/sockel/struct.Styles.html), and wraps
itself in an
[`OverlayHost`](https://docs.rs/sockel/latest/sockel/struct.OverlayHost.html)
so menus and dialogs have somewhere to float:

<!-- check: src/main.rs -->
```rust
use yew::prelude::*;

mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

#[function_component]
fn App() -> Html {
    html! {
        <>
            <sockel::Styles css={STYLESHEET} />
            <sockel::OverlayHost>
                <sockel::Stack class={classes!(sockel::class::CARD, class::SIDEBAR)}>
                    <sockel::Button>{ "Save" }</sockel::Button>
                </sockel::Stack>
            </sockel::OverlayHost>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

Finally, a test that fails when the Sass defines a class no Rust code writes:

<!-- check: tests/style_classes.rs -->
```rust
use sockel_build::Usage;

include!("../style.rs");

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .assert_clean(&style().style(env!("SOCKEL_STYLE")));
}
```

No other Sass step runs: `index.html` links no stylesheet, and `Trunk.toml`
has no `[tools] sass`. An app with no Sass of its own needs none of this; it
mounts `<sockel::Styles/>` and uses `sockel::class`.

To build against a local checkout of sockel, patch both crates in the
workspace's `Cargo.toml`:

```toml
[patch.crates-io]
sockel = { path = "../sockel" }
sockel-build = { path = "../sockel/sockel-build" }
```

<br>

## Nothing moves

The idea sockel is built around is that a page holds still while its state
changes. A component never grows or shrinks because it turned busy, showed an
error or got a longer number: it reserves room for its widest state up front.
Borders are always there and only change colour, focus is an outline rather
than a border, numbers use figures of one width, and anything that floats
(menus, popovers, dialogs, toasts) sits in a layer above the page instead of
pushing it aside.

Every component's page on the [documentation
site](https://udoprog.github.io/sockel/) runs a probe that steps through its
states and reports anything that moved.

The same rules are available for the app's own markup as Sass mixins, and
most as utility classes: `numeric`, `reserve-ch`, `truncate`, `clamp`,
`scroll-area`, `focus-ring`, `visually-hidden` and more. There are also
mixins with the exact rules of sockel's components (`card`, `button`,
`input`, `stack`, ...), for markup sockel does not render, such as a
dependency's sign-in form (the [mixins
reference](https://udoprog.github.io/sockel/#mixins) lists every one, with
its arguments):

```scss
@use "sockel" as p;

.auth-card {
    @include p.card;
}
```

<br>

## Your classes, checked by the compiler

`sockel-build` compiles sockel's CSS and the app's Sass into one stylesheet.
For every class the app defines it writes a constant, named without the
prefix: `.app-sidebar` becomes `class::SIDEBAR`. sockel's own classes are in
`sockel::class`. Since markup names classes through constants, a typo or a
deleted class is a compile error rather than an unstyled element.

Every class the app defines must start with its prefix (`app-`), so it never
collides with sockel's `sockel-` classes. sockel's classes are stable, so the
app may also refine them (`.sockel-card.app-sidebar`).

The test from getting started covers the other direction: a class that is
styled but never written is dead weight, and the test fails on it.

See the [sockel-build](https://docs.rs/sockel-build) documentation for the
details.

<br>

## Pay for what you render

Every component, or family of components that only work together, is a Cargo
feature. `full`, the default, enables them all. A feature gates a component's
Rust and its CSS together, so a component whose feature is off does not
compile, rather than rendering unstyled. Each feature brings in the features
it needs; the [features reference](https://udoprog.github.io/sockel/#features)
lists them all.

`sockel-build` knows which features sockel was built with, and builds only
their CSS. It can also prune rules for components the app never reaches, and
the Sass variable `$omit` leaves out smaller parts the app never uses (a
button variant, a modifier); the parts it can name are listed on the
[configuration reference](https://udoprog.github.io/sockel/#reference/parts-omit-can-name).

<br>

## Colours and configuration

sockel supplies no colours. The app sets seven custom properties, `--bg`,
`--surface`, `--text`, `--muted`, `--border`, `--accent` and `--shadow-1`, and
sockel derives the rest from them (danger, success, field borders, the dialog
scrim and so on). The status colours are mixed toward `--text`, so they keep
their contrast on light and dark backgrounds alike.

sockel's scales (type sizes, spacing, radii, control heights) are custom
properties too, which the app can override in its own CSS, at the root or for
part of the page:

```css
.dense {
    --control-height: 2rem;
}
```

Every value is also a `!default` Sass variable, which the build can set. This
changes values, never class names:

<!-- check-variant: build.rs -->
```rust
fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
        .with("text-md", "1.0625rem")
        .with("$radius-lg", "12px")
        .compile();
}
```

Every variable, its default and what it sets is listed on the
[configuration reference](https://udoprog.github.io/sockel/#reference),
generated from sockel's Sass.

<br>

## Translations

sockel has no English in its code. Every word a component shows or announces
is a message in a [phrasebook-catalog] catalog,
`sockel::phrasebookcatalog01::L10N`, in English and Swedish. The app adds it
after its own catalog, and can reword any of sockel's messages by defining the
same id itself:

```rust
let book = Phrasebook::builder()
    .language(Lang::EN_US, "English")
    .language(Lang::new("sv-SE"), "Svenska")
    .catalog(&APP)
    .catalog(&sockel::phrasebookcatalog01::L10N)
    .build();
```

Components read the `Phrases` provided with `ContextProvider<Phrases>`. Label
props (`close_label` and the like) override a single use. A library built on
sockel ships its own catalog the same way; see
[`phrasebookcatalog01`](https://docs.rs/sockel/latest/sockel/phrasebookcatalog01/index.html).

<br>

## Odds and ends

- **Test ids.** Every component takes a `testid` prop, written to
  `data-testid`. Set `SOCKEL_TEST_ID_ATTR` in the build environment to use
  another attribute name.
- **Props.** Nearly every props struct implements `Default`, so a struct
  literal ending in `..Default::default()` keeps compiling when a release adds
  a prop.
- **trug.** sockel's Sass can also be built with [trug], which gives the same
  stylesheet: `@use "pkg:sockel";` before the app's own Sass.

<br>

## Working on sockel

The documentation site is built from `showcase/` and served locally with
[trug]:

```sh
cd showcase
trug serve --port 8095
```

A new component lands with its page in `showcase/src/pages/mod.rs`, or
`cargo test --workspace` fails. Before sending a change, run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash scripts/check-features.sh
bash scripts/check-packaged.sh
```

[Yew]: https://yew.rs
[phrasebook-catalog]: https://docs.rs/phrasebook-catalog
[trug]: https://crates.io/crates/trug
