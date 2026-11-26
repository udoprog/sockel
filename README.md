# sockel

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sockel.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/sockel/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/sockel/actions?query=branch%3Amain)

[Yew] components that hold still.

A button turning busy, a count gaining a digit, a field showing its error, a
menu opening: in most UIs each of these nudges the layout around it. With
sockel none of them does. Every component reserves the room its widest
state needs, and overlays sit in a layer of their own, so an app's pages
never jitter.

**[See every component live in the showcase](https://udoprog.github.io/sockel/)**:
each page has controls for its props, its code, and a probe that checks
nothing moves.

- **Styled out of the box.** Every component ships its Sass, and its
  variants are props, so markup and styles never drift apart.
- **Typed classes.** `sockel-build` compiles the app's own Sass with
  sockel's into one stylesheet, with a Rust constant per class: a misspelt
  class does not compile, and a test finds the dead ones.
- **Pay for what you render.** Each component is a Cargo feature gating its
  Rust and its CSS together.
- **Your colours.** sockel brings no palette: it reads seven colour tokens
  from the app and derives the rest, light or dark. Its scales (space, type,
  radii, control heights) are tokens too.
- **Tools against jitter.** Mixins and classes for numerals of one width,
  truncation, line clamps, scroll areas and focus rings, for the app's own
  markup (see
  [Against jitter](https://docs.rs/sockel/latest/sockel/#against-jitter-mixins-and-utilities)).
- **Translated.** No English in the code: every word comes from a catalog
  the app can extend (see
  [Translations](https://docs.rs/sockel/latest/sockel/#translations)).

<br>

## Getting started

An app depends on `sockel` and builds one stylesheet, sockel's CSS and its
own Sass, with `sockel-build` in its `build.rs`. No other Sass step runs, and
no path leads into sockel's source. The blocks below are the files of a
complete app.

<br>

### Dependencies

In the app's `Cargo.toml`, sockel with the features the app renders (see
[Choosing features](https://docs.rs/sockel/latest/sockel/#choosing-features); leave out `default-features` and
`features` for all of them), and `sockel-build` to build and test:

<!-- check: Cargo.toml -->
```toml
[dependencies]
sockel = { version = "0.3", default-features = false, features = ["button", "layout", "overlay"] }
yew = { version = "0.23", features = ["csr"] }

[build-dependencies]
sockel-build = "0.3"

[dev-dependencies]
sockel-build = "0.3"
```

<br>

### Developing against a checkout

To work against a local checkout of sockel, patch both crates once in the
`Cargo.toml` at the workspace root (the app's own, when it is not in a
workspace):

```toml
[patch.crates-io]
sockel = { path = "../sockel" }
sockel-build = { path = "../sockel/sockel-build" }
```

The paths are relative to that `Cargo.toml`. Where they do not resolve (a git
worktree of the app), put the same `[patch.crates-io]` with absolute paths in
an untracked `.cargo/config.toml`. The dependency lines stay as they are, and
removing the patch returns to the registry.

<br>

### Styles

The app's Sass lives in a directory, say `style/`, with an `_index.scss`
(or `index.scss`) that `@use`s its partials. It loads sockel's tokens and
mixins as `"sockel"`:

<!-- check: style/_index.scss -->
```scss
@use "sockel" as p;

// The colours sockel reads; it supplies none (see Theming).
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

// sockel's classes are stable, so the app may refine them.
.sockel-card.app-sidebar {
    --radius-lg: 12px;
}
```

`"sockel"` is the `style/` of the sockel package the app builds against. It
emits no second copy of sockel's CSS, which is compiled first in the same
build. Configure sockel's tokens through the build rather than `@use "sockel"
with (...)` (see [Configuration](https://docs.rs/sockel/latest/sockel/#configuration)).

Every class the app's Sass defines must be sockel's or start with the app's
prefix (`app-`), or the build fails, so the two never collide. Classes a
dependency's markup writes (a sign-in form the app styles) are the exception:
`.foreign(["auth-*"])` on the Config lets the app style them unprefixed (a trailing
`*` is a prefix), and they get no constant.

The stylesheet is sockel's CSS, then the app's, so an app rule as specific as
sockel's wins. An app may also style a sockel class whose part its `$omit`
leaves out (a label head it draws itself, with `heads.label` omitted); a
misspelt `sockel-` class is still an error.

<br>

### `style.rs` and `build.rs`

At the package root, one file defines the
[`Config`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html), so the build and the guard below share it:

<!-- check: style.rs -->
```rust
fn style() -> sockel_build::Config {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
}
```

`build.rs` includes it and compiles:

<!-- check: build.rs -->
```rust
include!("style.rs");

fn main() {
    style().compile();
}
```

It writes `style.css` and `classes.rs` to `OUT_DIR`, and reruns when
`build.rs`, anything under `style/` (partials and new files included) or
sockel's Sass changes.

<br>

### The class constants and the stylesheet

`classes.rs` holds the app's classes, named without the prefix
(`.app-sidebar` is `class::SIDEBAR`); sockel's are in `sockel::class`. Every
sockel component takes a `class` prop for them, and a class the Sass does not
define does not compile. Mount the stylesheet once at the root with
[`Styles`](https://docs.rs/sockel/latest/sockel/struct.Styles.html)'s `css`:

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
            <sockel::Stack class={classes!(sockel::class::CARD, class::SIDEBAR)}>
                <sockel::Button>{ "Save" }</sockel::Button>
            </sockel::Stack>
        </>
    }
}
```

The app's icons and overlays join at the same root: an
[`IconSet`](https://docs.rs/sockel/latest/sockel/struct.IconSet.html)
provider and an
[`OverlayHost`](https://docs.rs/sockel/latest/sockel/struct.OverlayHost.html),
from the `overlay` feature:

<!-- check: src/main.rs -->
```rust
#[function_component]
fn Root() -> Html {
    // A glyph the set leaves out is sockel's built-in one.
    let icons = use_state(|| {
        sockel::IconSet::default().with_named(
            "save",
            html! { <svg viewBox="0 0 16 16"><path d="M3 8h10" /></svg> },
        )
    });

    html! {
        <ContextProvider<sockel::IconSet> context={(*icons).clone()}>
            <sockel::OverlayHost>
                <App />
            </sockel::OverlayHost>
        </ContextProvider<sockel::IconSet>>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}
```

<br>

### No Sass elsewhere

`index.html` links no stylesheet (no `<link data-trunk rel="scss">`), and
`Trunk.toml` has no `[tools] sass`:

```html
<link data-trunk rel="rust" href="Cargo.toml" />
```

<br>

### A guard against dead classes

A class the Sass defines and no Rust writes is dead weight.
`sockel_build::Usage` finds them in a test:

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

`SOCKEL_STYLE` is sockel's style directory, which `compile()` hands the
package's crates and only `build.rs` is told. The guard includes the same
`style.rs`, so it checks the stylesheet the app builds (see [One Config for
the build and the guard](https://docs.rs/sockel-build/latest/sockel_build/#one-config-for-the-build-and-the-guard));
`.dynamic(["app-tone-*"])` in it allows classes built at runtime (a trailing
`*` is a prefix).

A class counts as written when a source names its constant (`class::SIDEBAR`)
or the class in a string literal; comments and items marked `#[cfg(test)]` do
not count. The test fails on a dead class and on an allowed pattern that
matches no class, and prints the class, single-use and dead counts.
`STYLE_INVENTORY=PATH` also writes `class<TAB>uses` lines to `PATH`.

<br>

### How sockel-build finds sockel

sockel's `build.rs` passes two values to the `build.rs` of every package
that depends on it directly (Cargo's `links` metadata): `DEP_SOCKEL_FEATURES`,
its enabled features, and `DEP_SOCKEL_STYLE`, its packaged `style/`. `Config`
builds those features from those sources, so the app's stylesheet always
matches the sockel it links. A package that reaches sockel only through
another crate names them itself, with `Config::features([...])` and
`Config::style(dir)`.

<br>

### One stylesheet per app

Every [`Styles`](https://docs.rs/sockel/latest/sockel/struct.Styles.html) fills the same element, `<style id="sockel-styles">`
([`STYLE_ID`](https://docs.rs/sockel/latest/sockel/constant.STYLE_ID.html)): with `css` it sets the element's text to the app's
stylesheet, which holds sockel's; without it, it adds sockel's only when the
element is missing. Whichever mounts first (the app's, or a
`<sockel::Styles/>` inside a component), the page ends up with the app's
stylesheet, once. Never link or inject sockel's CSS another way beside it.

An app without Sass of its own needs no `build.rs`: it mounts
`<sockel::Styles/>` once at its root, which adds [`STYLESHEET`](https://docs.rs/sockel/latest/sockel/constant.STYLESHEET.html), and uses
`sockel::class`.

Every class sockel styles starts with `sockel-`, modifiers included
(`.sockel-stack.sockel-tight`), so none collides with an app's classes. A
constant is named after its class without the prefix, as an app's are:
`.sockel-page-head-title` is `sockel::class::PAGE_HEAD_TITLE`,
`.sockel-tight` is `sockel::class::TIGHT`.

<br>

### With trug

[trug] builds the same stylesheet from sockel's packaged Sass, pruned to the
enabled features (sockel opts in with `[package.metadata.trug]`). The entry
emits sockel's CSS, then the app's:

```scss
// trug/app.scss
@use "pkg:sockel";
@use "../style/index";
```

A `trug/sockel.scss` holding `@forward "pkg:sockel";` lets the app's own
`@use "sockel"` resolve; build with `trug css trug/app.scss -I trug -o
style.css`. `build.rs` still writes the class constants, and both builds give
the same stylesheet.

<br>

## Migrating from trunk's Sass step

An app that compiles its styles with trunk (`<link data-trunk rel="scss">`
and `@use "../../sockel/style/..."`) moves to the
[getting-started](https://docs.rs/sockel/latest/sockel/#getting-started) setup in five steps:

1. **Dependencies.** Replace `sockel = { path = ... }` with the lines in
   [Dependencies](https://docs.rs/sockel/latest/sockel/#dependencies) and the patch in
   [Developing against a checkout](https://docs.rs/sockel/latest/sockel/#developing-against-a-checkout).
2. **`@use` lines.** Delete every `@use` of sockel's source
   (`"../…/sockel/style"`, `"../…/sockel/style/tokens"`, a partial):
   `sockel-build` compiles sockel's CSS first, with the app's features. A
   partial that needs sockel's mixins or token functions writes
   `@use "sockel" as p;` and calls `p.below(...)` and the like. A `with (...)` on the old `@use` moves to
   `Config::with` in `style.rs` (see [Configuration](https://docs.rs/sockel/latest/sockel/#configuration)).
3. **The prefix.** Every class the app's Sass defines must start with one
   prefix (`app-`), given to `Config::prefix`. Rename the app's classes in
   the Sass and the markup together; a rule that refines a sockel class
   (`.sockel-card.app-sidebar`) keeps sockel's part as it is, and a class
   already starting with the prefix keeps its name. Watch a search and
   replace: it also reaches strings that are not classes (ARIA roles,
   `data-testid` values), and tests' selectors (CSS, XPath `@class`, JS)
   need the prefix too.
4. **Class literals.** Add the `mod class` of
   [the class constants](https://docs.rs/sockel/latest/sockel/#the-class-constants-and-the-stylesheet) and replace
   the app's class strings in Rust with its constants (`"app-sidebar"` with
   `class::SIDEBAR`, `classes!("sockel-card", "app-sidebar")` with
   `classes!(sockel::class::CARD, class::SIDEBAR)`). A misspelt or removed
   class then fails to compile, and the dead-class test finds the rest.
   Class names also travel through props, builders, match arms, `format!`
   and `Option` or tuple values; look there too. A constant's value is the
   prefixed class, so a string also used for something else (a test id)
   needs its own literal. While some constants are unused,
   `#[allow(dead_code)]` on `mod class` keeps `-D warnings` passing.
5. **trunk.** Remove `<link data-trunk rel="scss">` from `index.html` and
   `[tools] sass` from `Trunk.toml`, and mount `<sockel::Styles
   css={STYLESHEET} />` once at the root. The stylesheet it linked
   (`style/main.scss`) becomes the style directory's `_index.scss`.

`sockel-build` compiles with [sasso], a Rust port of dart-sass, and its
CSS matches dart-sass's. A block's declarations written after a nested
rule or an `@include` of rules come out in a second block after those
rules, so a `padding` after `@include p.below("phone") { padding: ... }`
wins over the media query. A dart-sass feature sasso lacks fails in
`build.rs` with the error and its line.

Including sockel's Sass by path (`@use "../sockel/style"`) does not work with
sockel from a registry, and is not supported.

<br>

## Choosing features

sockel has a Cargo feature per component, or per family of components that
only work together; `full`, the default, is all of them. An app that renders
a few turns the default off and names those, and each brings in the ones it
needs:

```toml
[dependencies]
sockel = { version = "0.3", default-features = false, features = ["field", "input", "layout"] }
```

A feature gates a component's Rust and its CSS together. [`STYLESHEET`](https://docs.rs/sockel/latest/sockel/constant.STYLESHEET.html) and
`sockel::class` hold only the enabled features' rules and classes, so a
component or class whose feature is off does not compile, rather than
rendering unstyled. The tokens, hooks, utilities and the menus' shared
surface are always in, and so is the `:root` tokens' part of a feature
(`tokens.table`, ...) only while a feature that reads it is on.

An app compiling its own Sass through `sockel-build` gets the same subset
without asking (see [How sockel-build finds sockel](https://docs.rs/sockel/latest/sockel/#how-sockel-build-finds-sockel)).
A package that reaches sockel only through another crate is not told, and
gets the whole stylesheet unless `Config::features([...])` names its
features. Cargo builds one set
of features for everything built together, so a workspace whose other
member wants more gets the union.

| Feature | Brings in | CSS alone |
| --- | --- | --- |
| `action-group` | | 3.8 KB |
| `badge` | `icons` | 9.8 KB |
| `button` | `icons`, `skeleton` | 13.6 KB |
| `card-grid` | | 4.2 KB |
| `checkbox` | `icons` | 6.6 KB |
| `chip` | `icons` | 6.6 KB |
| `chip-select` | `chip`, `layout`, `search-select` | 50.1 KB |
| `choice` | | 7.2 KB |
| `code-editor` | `overlay`, `select` | 50.9 KB |
| `color-select` | `select` | 50.4 KB |
| `composer` | `drop-zone`, `input` | 23.8 KB |
| `confirm` | `action-group`, `badge`, `button`, `notice`, `overlay` | 39.5 KB |
| `disclosure` | `head`, `icons` | 15.8 KB |
| `drop-zone` | `button` | 16.0 KB |
| `field` | | 6.4 KB |
| `file-browser` | `badge`, `button`, `field`, `icons`, `input`, `layout`, `notice`, `overlay`, `rows` | 61.5 KB |
| `head` | `layout` | 13.0 KB |
| `icons` | | 4.0 KB |
| `input` | `icons` | 10.5 KB |
| `lanes` | `badge`, `reorder` | 44.8 KB |
| `layout` | | 9.1 KB |
| `library-toolbar` | `input`, `layout` | 16.2 KB |
| `lightbox` | `overlay` | 29.5 KB |
| `loadable` | `notice`, `skeleton` | 24.3 KB |
| `log-view` | `button` | 15.7 KB |
| `media` | `icons` | 5.7 KB |
| `menu` | `button`, `icons`, `overlay` | 33.2 KB |
| `notice` | `badge`, `button` | 24.0 KB |
| `overlay` | `action-group`, `button`, `head`, `icons` | 27.7 KB |
| `pager` | `icons` | 6.9 KB |
| `panel` | | 4.9 KB |
| `picker` | `overlay`, `pager`, `select` | 54.9 KB |
| `property-list` | `button`, `input` | 23.0 KB |
| `quantity` | `select`, `stepper` | 47.8 KB |
| `reorder` | `icons`, `rows`, `tooltip` | 36.7 KB |
| `rows` | `button` | 19.2 KB |
| `search-select` | `select` | 47.5 KB |
| `save-bar` | `button` | 14.7 KB |
| `seam` | | 5.3 KB |
| `select` | `input`, `menu`, `tooltip` | 46.1 KB |
| `shell` | `badge` | 14.3 KB |
| `shortcuts` | `overlay` | 29.2 KB |
| `skeleton` | `icons`, `media` | 7.5 KB |
| `split-view` | `seam` | 6.4 KB |
| `stepper` | `input` | 12.1 KB |
| `switch` | `icons` | 6.0 KB |
| `table` | `icons` | 10.8 KB |
| `tabs` | | 9.7 KB |
| `time` | | 3.8 KB |
| `toast` | `save-bar`, `toasts` | 36.4 KB |
| `toasts` | `badge`, `button`, `overlay` | 35.3 KB |
| `toggles` | `checkbox`, `chip`, `choice`, `switch` | 14.6 KB |
| `tooltip` | `overlay` | 28.8 KB |

A feature is named for the component or family it gates; the
[showcase](https://udoprog.github.io/sockel/) shows each one. "CSS alone" is `STYLESHEET` (bytes / 1000) with only that feature and what it brings in;
with none it is 3.4 KB, and with `full` 145 KB. Each feature builds alone,
and its CSS holds no other feature's rules.
`scripts/feature-sizes.sh` in the repository measures every row and rewrites the
column and these figures in place; run it when the styles change.

<br>

## Theming

The app supplies seven colour tokens: `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`. sockel derives the rest
from them, each a default the app overrides by name (or with `Config::with`,
as `$danger` and so on):

| Token | Default | For |
| --- | --- | --- |
| `--danger` | `#d92d20` mixed 70% with `--text` | errors, destructive actions |
| `--success` | `#12a150` mixed 65% with `--text` | confirmations |
| `--warning` | `#dc8a00` mixed 70% with `--text` | cautions |
| `--on-accent` | `--bg` | text and icons on an accent background |
| `--surface-raised` | `--surface` | menus, popovers, dialogs |
| `--scrim` | `rgb(0 0 0 / 0.45)` | what dims the page behind a dialog |
| `--field-border` | `--border` | a field's edge (inputs, select and picker faces) and a checkbox's, switch's and choice's |
| `--field-bg` | `--surface` | a field's fill |
| `--card-border` | `--border` | a card's edge (`transparent` for cards without one) |
| `--card-border-width` | `--space-px` | its width (`0` for a card whose content meets its rounded edge, with no frame) |
| `--panel-border` | `--border` | a [`Panel`](https://docs.rs/sockel/latest/sockel/struct.Panel.html)'s ring |
| `--panel-tint-text` | `--text` | a tint `Panel`'s text |
| `--band-chrome-ground` | `--surface` | a chrome `ToolBand`'s ground |
| `--menu-item-icon-ink` | `currentColor` | a menu row's icon (an accent beside two-line rows, say) |
| `--selection` | 16% of `--accent` in a `NavList`, 14% in a menu (unset: optional) | the current row's ground under its accent text (a `NavList`'s current item, a menu's current page) |

The status hues are mixed toward `--text`, so they keep their contrast on a
light and a dark background alike. The defaults are computed on `:root`: an
app that sets its colours on a narrower element (a dark panel) sets these
there too. An app whose theme defines one of these itself sets its Sass
variable to `false` (`$danger: false`), and sockel leaves that default off
`:root`.
`--focus-ring-color` (default `--accent`) colours the focus ring.

sockel's own scales are custom properties on `:root` too (`--text-*`,
`--space-*`, `--radius-*`, `--control-height`, `--page-*` and the rest, listed
under [Configuration](https://docs.rs/sockel/latest/sockel/#configuration)), and every rule reads them, so an app
retunes the stylesheet by overriding them in its own CSS, loaded after it or
with a more specific selector:

```css
:root {
    --bg: #fafafa;
    --surface: #fff;
    --text: #1d1d1f;
    --muted: #6e6e73;
    --border: #d2d2d7;
    --accent: #0a66c2;
    --shadow-1: 0 1px 2px rgb(0 0 0 / 0.08);
    --radius-lg: 12px;
}

.dense {
    --control-height: 2rem;
}
```

Breakpoints and most component settings are Sass-only, with no custom
property: `$breakpoints` and every `-breakpoint`, `$badge-*`, `$button-*`,
`$card-padding*`, `$chip-*`, `$choice-*` (but `$choice-width`),
`$confirm-*`, `$corner-transform`, `$count-*`, `$dialog-*` (but its
widths, heights and top), `$disclosure-*`, `$drag-handle-*`, `$drop-*` (but
`-min-height`, `-tint`, `-tint-over`), `$empty-state-*`, `$error-box-*`,
`$glyph-phone-size`, `$input-*`, `$listbox-*`, `$menu-*` (but the widths and
`-item-icon-ink`), `$notice-*`, `$pager-*`, `$pick-row-*`,
`$picker-popup-width`, `$popover-*` (but `-max-width`, `-sheet-height`),
`$reorder-carried-opacity`, `$row-list-*`, `$select-sheet-row-height`,
`$selection-bar-*`, `$skeleton-*` (but `-duration`), `$stepper-*`,
`$switch-fill`, `$switch-on-thumb`, `$switch-thumb-inset`, `$tab-*` and
`$toolbar-gap`. An app built with `sockel-build` sets them with
`Config::with`.

<br>

## Configuration

sockel's values are defaults. Every token is a `!default` variable of
`style/_tokens.scss`, and all but the Sass-only ones (see
[Theming](https://docs.rs/sockel/latest/sockel/#theming)) are also emitted as custom properties on `:root`. An app
sets its own in `build.rs`, before `compile()`; values change, never classes:

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

The `$` is optional and the value is any Sass expression. Without `.local`,
the same call configures sockel's own stylesheet. Local Sass cannot `@use
"sockel" with (...)`, since sockel compiles first. The rules keep reading
the custom properties, so a page can still override those at runtime.
`"null"` sets nothing for a token whose default is not `null` (Sass's
`!default` takes `null` as unset); turn one off with a CSS value (`auto`,
`inherit`, `0`).

A value can name the app's own Sass variables:
`with_module("style/_vars.scss", "v")` loads that module first, so
`.with("text-sm", "v.$font-sm")` keeps one source for the value. The module
must emit no CSS.

| Variable | Default |
| --- | --- |
| `$text-3xs`, `$text-2xs`, `$text-xs`, `$text-sm` | `0.625rem`, `0.6875rem`, `0.75rem`, `0.8125rem` |
| `$text-md`, `$text-base`, `$text-heading`, `$text-lg` | `0.875rem`, `0.9375rem`, `1.125rem`, `1.25rem` |
| `$text-title`, `$text-xl`, `$text-2xl` | `1.4rem`, `1.5rem`, `1.75rem` |
| `$section-title-size` (a section head's title) | `var(--text-lg)` |
| `$space-0`, `$space-px`, `$space-1`, `$space-2`, `$space-3` | `0.125rem`, `1px`, `0.25rem`, `0.5rem`, `0.75rem` |
| `$space-4`, `$space-5`, `$space-6`, `$space-7`, `$space-8`, `$space-9` | `1rem`, `1.25rem`, `1.5rem`, `2rem`, `2.5rem`, `3rem` |
| `$radius-xs`, `$radius-sm`, `$radius-md`, `$radius-lg`, `$radius-pill` | `4px`, `8px`, `10px`, `16px`, `999px` |
| `$nest-inset` | `var(--space-1)` |
| `$control-height`, `$control-height-small` | `2.25rem`, `1.75rem` |
| `$mark-height`, `$touch-height` | `1.5rem`, `2.75rem` |
| `$dot-size`, `$corner-size` | `0.5rem`, `1.125rem` |
| `$badge-height`, `$badge-neutral-tone`, `$badge-tint` | `var(--mark-height)`, `var(--muted)`, `null` (a badge's height, the ink of a neutral one, and the percentage of its tone a toned, unfilled badge is filled with and its edge tinted by; `null`: no fill, the edge as a neutral's) |
| `$pill-ink` | `color-mix(in srgb, var(--tone) 70%, var(--text))` (a pill's text, its tone mixed toward `--text` so it reads at 4.5:1 on the pill's tint; `var(--tone)` paints it in the tone itself) |
| `$pill-button-hover-ink` | `color-mix(in srgb, var(--tone) 45%, var(--text))` (a pill button's text on hover, mixed further toward `--text` so it reads at 4.5:1 on the hover's 20% tint) |
| `--corner-top`, `--corner-end` | `0` each, set on a corner badge (its offset from its parent's top end corner; not a token, so a class of the app's may set it without out-ranking `.sockel-corner`) |
| `$corner-transform` | `translate(50%, -50%)` (how a corner badge sits on its parent's top end corner: centred on it; `none` lays it inside) |
| `$count-height`, `$count-padding` | `var(--mark-height)`, `var(--space-2)` (a count's height and inline padding, which its least width includes) |
| `$glyph-size` | `1em` |
| `$glyph-phone-size` | `null` (`--glyph-size` below the phone breakpoint; `null` keeps `$glyph-size`) |
| `$skeleton-duration` | `1.4s` |
| `$pager-gap`, `$pager-button-radius`, `$pager-button-ink`, `$pager-button-text` | `var(--space-1)`, `var(--radius-sm)`, `var(--text)`, `null` (inherited) (a pager's gap and its buttons' corners, ink and size) |
| `$pager-button-hover-fill`, `$pager-button-hover-ink`, `$pager-current-fill`, `$pager-current-ink`, `$pager-step-ink`, `$pager-phone-height` | 12% of `--accent`, `null` (as at rest), `var(--accent)`, `var(--on-accent)`, `null` (the button's), `null` (a button's hover, the current page's fill and ink, the chevrons' ink, and the controls' height below the phone breakpoint) |
| `$button-min-width`, `$button-part-justify`, `$button-part-min-width`, `$button-part-radius`, `$input-clear-min-height`, `$input-clear-disabled-opacity`, `$input-clear-disabled-cursor` | `auto`, `null`, `null`, `null`, `null`, `null`, `null` (a button's least width under `--button-min-width`; a switch's, cross's, clear's and action's, picker face's, menu item's (a button row's, not a link row's) and step's justify-content, least width and corner radius (switch and cross only), as a blanket button rule gives them; a clear's least height and its disabled opacity and cursor) |
| `$pager-button-height`, `$pager-button-padding`, `$pager-padding-top`, `$pager-current-edge`, `$pager-range-text` | `var(--control-height)`, `var(--space-2)`, `null` (none), `null` (the ordinary edge), `null` (inherited) (a pager button's and gap's height, the room either side of its number, the room above the row, the current page's edge colour, the range text's size) |
| `$skeleton-fill`, `$skeleton-edge`, `$skeleton-radius` | 8% of `--text` over `--surface`, `null` (none), `var(--radius-sm)` (a skeleton's fill, the 1px line inside its edge and its corners) |
| `$skeleton-line-height`, `$skeleton-line-inset`, `$skeleton-phone-height` | `1lh`, `null` (a fifth of a line), `null` (a line's height and the transparent inset that thins it; a block's height below the phone breakpoint) |
| `$check-size`, `$switch-width`, `$switch-height`, `$choice-width` | `1.25rem`, `2.5rem`, `1.5rem`, `6rem` |
| `$choice-fill`, `$choice-radius`, `$choice-inset`, `$choice-padding` | `var(--surface)`, `var(--radius-md)`, `var(--nest-inset)`, `var(--space-3)` (a choice's ground, radius, inset and an option's inline padding) |
| `$choice-ink`, `$choice-font-size`, `$choice-divider` | `var(--text)`, `null`, `null` (an option's ink and size, and the colour of the rule between options; `null`: inherited, none) |
| `$choice-hover-fill`, `$choice-hover-ink`, `$choice-selected-fill`, `$choice-selected-ink` | `color-mix(in srgb, var(--accent) 12%, transparent)`, `null` (as at rest), `var(--accent)`, `var(--on-accent)` |
| `$choice-height`, `$choice-stack-breakpoint` | `var(--control-height)`, `"phone"` (a choice's height, a stacked option's least, and where a `Narrow` one stacks) |
| `$choice-columns`, `$choice-align-self`, `$choice-gap`, `$choice-icon-width`, `$choice-focus-edge` | `1fr`, `null`, `var(--space-2)`, `null`, `null` (the options' columns, `auto` each its own width; the group's `align-self` at no specificity; an option's icon-to-label gap; an icon-only option's least width, unpadded (`ICON`); the group's border while it holds the focus) |
| `$tab-gap`, `$tab-phone-icons` | `var(--space-2)`, `false` (a tab's icon-to-label gap, and whether its icon stays below the phone breakpoint) |
| `$tab-bar-icon-size`, `$tab-bar-gap` | `var(--text-xl)`, `var(--space-1)` (a stacked tab's icon size and its icon-to-label gap) |
| `$chip-height`, `$chip-padding-block`, `$chip-padding`, `$chip-gap`, `$chip-font-size`, `$chip-line-height` | `var(--control-height-small)`, `0`, `var(--space-3)`, `var(--space-2)`, `var(--text-sm)`, `1` (a chip's height, `auto` as its content; its block and inline padding, gap, size, `inherit` its parent's, and line height) |
| `$chip-media-inset`, `$chip-icon-inset` | `var(--space-1)`, `null` (how far a chip's avatar and its icon sit from its start edge; `null` keeps the padding) |
| `$chip-on-edge`, `$chip-on-fill`, `$chip-on-ink` | `var(--accent)`, `var(--accent)`, `var(--on-accent)` (an on toggle chip's edge, fill and ink) |
| `$chip-remove-glyph-size` | `null` (the size of an item chip's remove cross; `null` keeps the glyph's own) |
| `$switch-fill`, `$switch-on-thumb`, `$switch-thumb-inset` | `var(--surface)`, `var(--on-accent)`, `null` (a switch's track fill while off, its thumb's colour while on, and the thumb's distance from the track's outer edge; `null` keeps two `--space-px`) |
| `$filter-width` | `10.5rem` |
| `$field-label-width`, `$field-message-line`, `$field-error-lines` | `12rem`, `1rem`, `1` |
| `$avatar-sm`, `$avatar-md`, `$avatar-lg`, `$avatar-xl` | `1.75rem`, `2.5rem`, `3.5rem`, `7rem` |
| `$page-workspace`, `$page-reading`, `$page-wide`, `$page-gutter` | `64rem`, `40rem`, `90rem`, `var(--space-8)` |
| `$card-padding` | `var(--space-4)` |
| `$card-padding-phone` | `null` (a card keeps `$card-padding` on a phone) |
| `$toolbar-gap` | `var(--space-1)` |
| `$button-padding`, `$button-padding-small` | `var(--space-3)`, `var(--space-2)` (a `.sockel-button`'s sides, regular and small) |
| `$button-line-height` | `1` (a button's line height; an app whose labels carry descenders in a clipped label part can use `inherit`) |
| `$button-primary-weight` | `null` (a primary button keeps the weight it inherits) |
| `$button-radius` | `var(--radius-inner, var(--radius-sm))` |
| `$button-gap`, `$button-press-shade`, `$button-transition` | `null` (no gap on the button itself, for an app's markup that puts its icon and text straight in it; no shade laid over it while pressed down; no transition) |
| `$button-colours` | `()`: per variant (`secondary`, `primary`, `ghost`, `quiet`, `danger`, `success`), a map of any of `fill`, `ink`, `edge`, `hover-edge`, `hover-fill`, `hover-ink`, `pressed-fill`, `pressed-ink`, `pressed-edge`, each in place of sockel's; a variant falls back on the plain button's |
| `$button-text`, `$button-text-small` | `null` (the parent's), `var(--text-sm)` |
| `$button-content-gap` | `var(--space-2)` (between a button's icon and label) |
| `$button-icon-offset` | `null` (a leading icon's inline-start margin beside a label; a negative one sets it into the padding) |
| `$button-disabled-cursor` | `not-allowed` (a disabled button's or pager button's cursor) |
| `$button-phone-height` | `null` (below the phone breakpoint every size but touch keeps its height; when set, a small button there is this high in the regular type) |
| `$dialog-padding`, `$dialog-padding-phone` | `var(--space-5)`, `null` (a dialog's head start, body and foot sides as `--dialog-pad`; on a sheet, `null` keeps `$dialog-padding`) |
| `$dialog-body-gap`, `$dialog-shadow` | `null` (the body is a plain block; set, a column with that gap, its paragraphs without margins), `var(--shadow-1)` |
| `$dialog-head-padding`, `$dialog-head-rule` | `var(--space-3) var(--space-3) 0 var(--dialog-pad)`, `null` (no rule under the head; set, its `border-bottom`) |
| `$dialog-body-padding` | `var(--space-3) var(--dialog-pad) var(--dialog-pad)` |
| `$dialog-title-size`, `$dialog-title-weight` | `null`, `null` (the section head's; set, the dialog title's own) |
| `$dialog-centred`, `$dialog-max-height` | `false` (hung from `--dialog-top`; `true` centres it, so growing content moves both edges), `null` (as tall as the window's room; set, at most that above `$sheet-breakpoint`) |
| `$dialog-scroll-gutter` | `stable` (the body keeps its scrollbar's room; `auto` gives it to the content until it scrolls) |
| `$dialog-sheet-edge` | `all` (a sheet keeps its side edges, its bottom transparent; `top`: only its top edge) |
| `$popover-padding`, `$popover-shadow` | `var(--space-3)`, `var(--shadow-1)` (a popover's body padding and its shadow, a hover card's included) |
| `$popover-radius`, `$popover-sheet-edge` | `var(--radius-md)`, `all` (a popover's corners; the edges its sheet keeps, as `$dialog-sheet-edge`) |
| `$popover-scroll-gutter` | `stable` (the body keeps its scrollbar's room, so a popover is as wide scrolling or not; `auto` gives it to the content until it scrolls) |
| `$confirm-gap` | `var(--space-3)` (between what a confirm says and its buttons) |
| `$confirm-scroll-gutter` | `auto` (a confirm's body has a scrollbar gutter only where it scrolls; `stable` keeps the popover's) |
| `$focus-ring-color`, `$focus-ring-width`, `$focus-ring-offset` | `var(--accent)`, `2px`, `2px` |
| `$tooltip-max-width` | `24rem` |
| `$picker-popup-width` | `var(--menu-width-wide)` (a Picker's or PickerMenu's list's least width; never measured from its text) |
| `$listbox-max-width`, `$listbox-active-fill` | `var(--popover-max-width)`, `color-mix(in srgb, var(--text) 7%, transparent)` (a listbox's widest, a Select's list included, and its active row's fill: the menu's) |
| `$listbox-max-height`, `$listbox-shadow`, `$listbox-group-name-transform`, `$listbox-group-name-letter-spacing` | `null` (a listbox's tallest, under the room beside its opener, which it always keeps within; its shadow, `null` keeping the list's `--shadow-1`; its group headings' `text-transform` and `letter-spacing`; `null` leaves each as without) |
| `$select-sheet-row-height` | `null` (a Select's rows in its phone sheet, its close button too; `null` keeps `--control-height`, `var(--touch-height)` gives a finger room) |
| `$layer-sticky`, `$layer-overlay`, `$layer-dialog`, `$layer-toast`, `$layer-tooltip` | `10`, `100`, `200`, `300`, `400` (`$layer-dialog` is unused: dialogs sit at `$layer-overlay`) |
| `$menu-width`, `$menu-width-wide`, `$menu-width-wider`, `$menu-width-panel` | `12rem`, `16rem`, `20rem`, `34rem` |
| `$menu-edge`, `$menu-ground`, `$menu-shadow`, `$menu-padding`, `$menu-text` | `null` each (a menu's border colour, background, shadow, padding and font size; `null` keeps the list's: `--border`, `--surface-raised`, `--shadow-1`, `--nest-inset`, inherited) |
| `$menu-scroll-gutter` | `null` (a menu's `scrollbar-gutter`; `null` keeps the list's `stable`, `auto` gives the room to the rows until the menu scrolls) |
| `$menu-item-height`, `$menu-item-padding`, `$menu-item-gap`, `$menu-item-radius`, `$menu-item-hover-fill` | `null` each (a menu row's least height, padding, gap, corners and hover/focus fill; `null` keeps the row's: `--control-height`, `var(--space-1) var(--space-3)`, `var(--space-2)`, the nested radius, 7% of `--text`) |
| `$menu-item-edge` | `null` (a menu row's reserved 1px border, its focus edge; `outline` reserves none and draws the focus edge as a 1px inset outline in the same place, so a row taller than its least height is its content's height) |
| `$menu-radius` | `null` (a menu's corners; `null` keeps `--radius-md`) |
| `$menu-item-phone-height` | `null` (a menu row's least height below the phone breakpoint, as `$button-phone-height` is a button's; `null` keeps the one above it) |
| `$menu-item-mark-ink` | `null` (a check's or radio's mark; `null` keeps `--accent`, `currentColor` draws it as the row's text) |
| `$menu-separator-margin`, `$menu-separator-edge` | `null` each (a [`MenuSeparator`](https://docs.rs/sockel/latest/sockel/struct.MenuSeparator.html)'s block margin and rule colour; `null` keeps `var(--space-1)` and `--border`) |
| `$menu-heading-padding`, `$menu-heading-text`, `$menu-heading-ink`, `$menu-heading-weight` | `null` each (a [`MenuHeading`](https://docs.rs/sockel/latest/sockel/struct.MenuHeading.html)'s padding, font size, colour and weight; `null` keeps `var(--space-1) var(--space-3) 0`, `--text-xs`, `--muted`, 600) |
| `$menu-note-padding`, `$menu-note-text`, `$menu-note-ink` | `null` each (a [`MenuNote`](https://docs.rs/sockel/latest/sockel/struct.MenuNote.html)'s padding, font size and colour; `null` keeps `var(--space-2) var(--space-3)`, `--text-sm`, `--muted`) |
| `$input-hover-edge` | `var(--accent)` (a text field's edge on hover: `.sockel-input`, `.sockel-textarea`, `.sockel-input-box`) |
| `$input-focus-edge` | `null` (its edge while focused; `null` keeps the edge it has) |
| `$input-ring-offset` | `null` (its focus ring's offset; `null` keeps `--focus-ring-offset`, a negative one draws the ring over the edge) |
| `$input-padding-inline` | `var(--space-2)` (its inline padding) |
| `$input-text` | `null` (its text size; `null` inherits it) |
| `$input-phone-height` | `null` (its height below the phone breakpoint, as `$button-phone-height` is a button's; `null` keeps the one above it) |
| `$notice-padding`, `$notice-gap`, `$notice-radius`, `$notice-fill`, `$notice-edge`, `$notice-align` | `var(--space-2) var(--space-3)`, `var(--space-2)`, `var(--radius-md)`, 10% of `--tone` over `--surface`, 45% of `--tone` over `--border`, `flex-start` |
| `$notice-min-height`, `$notice-body-padding`, `$notice-line-height`, `$notice-icon-height`, `$notice-icon-ink`, `$notice-danger-ink` | one small control plus its padding and edge, centring one line in a small control, `1.4`, `var(--control-height-small)`, `var(--tone)`, `null` (the text's) |
| `$empty-state-padding`, `$empty-state-icon-size`, `$empty-state-title-size`, `$empty-state-title-ink`, `$empty-state-title-weight` | `var(--space-7) var(--space-4)`, `var(--text-2xl)`, `var(--text-md)`, `var(--text)`, `600` |
| `$empty-state-edge`, `-radius`, `-compact-radius`, `-icon-fill`, `-icon-ink`, `-icon-box`, `-icon-radius`, `-icon-gap`, `-hint-max-width` | `null` each (none) |
| `$empty-state-compact-title-size`, `-compact-title-weight`, `-compact-title-ink`, `-compact-icon-box`, `-compact-icon-fill`, `-compact-icon-ink`, `-compact-justify` | `null` each (as the regular one) |
| `$error-box-padding`, `$error-box-gap`, `$error-box-fill`, `$error-box-edge`, `$error-box-shadow`, `$error-box-message-weight` | `var(--space-4)`, `var(--space-2)`, 8% of `--tone` over `--surface`, 45% of `--tone` over `--border`, `null`, `null` |
| `$error-box-cause-font`, `$error-box-cause-text`, `$error-box-text-end` | `ui-monospace, monospace`, `var(--text-xs)`, `null` (as much under a flanked box's text as above it) |
| `$popover-max-width`, `$popover-sheet-height` | `22rem`, `75dvh` |
| `$dialog-width`, `$dialog-width-wide`, `$dialog-min-height`, `$dialog-height-tall`, `$dialog-top`, `$dialog-sheet-height` | `32rem`, `48rem`, `10rem`, `40rem`, `8dvh`, `85dvh` |
| `$toast-max-width` | `28rem` |
| `$seam-grip`, `$seam-min` | `var(--space-3)`, `8rem` (a [`Seam`](https://docs.rs/sockel/latest/sockel/struct.Seam.html)'s grip width and each side's least size) |
| `$panel-ground`, `$panel-tint` | 4% of `--text` over `--bg`, 14% of `--accent` (a [`Panel`](https://docs.rs/sockel/latest/sockel/struct.Panel.html)'s ground and a tint one's) |
| `$card-grid-min`, `$card-grid-wide`, `$card-grid-narrow`, `$card-grid-narrow-phone` | `15rem`, `18rem`, `10.5rem`, `9rem` (a [`CardGrid`](https://docs.rs/sockel/latest/sockel/struct.CardGrid.html)'s least column width: regular, `wide`, `narrow`, `narrow` on a phone) |
| `$table-max-height` | `24rem` |
| `$drop-min-height`, `$drop-tint`, `$drop-tint-over` | `var(--touch-height)`, `12%`, `28%` |
| `$drop-overlay-inset`, `$drop-edge-width`, `$drop-tall-label-offset`, `$drop-busy-pulse` | `0`, `null` (the zone's own border), `40vh`, `null` (no pulse) |
| `$swatch-size`, `$color-select-width` | `1.25rem`, `10rem` |
| `$danger`, `$success`, `$warning`, `$on-accent`, `$surface-raised`, `$scrim`, `$field-border`, `$field-bg`, `$card-border`, `$card-border-width`, `$band-chrome-ground`, `$panel-border`, `$panel-tint-text`, `$menu-item-icon-ink` | see [Theming](https://docs.rs/sockel/latest/sockel/#theming) |
| `$panel-text`, `$panel-space-after`, `$panel-tint-space-after`, `$panel-tint-radius` | `1em` (its parent's), `0`, `var(--panel-space-after)`, `var(--radius-pill)` |
| `$selection-bar-ground`, `$selection-bar-rule`, `$selection-bar-padding`, `$selection-bar-min-height`, `$selection-bar-radius`, `$selection-bar-margin`, `$selection-bar-text` | `null`, `var(--space-px) solid var(--border)`, `var(--space-2) 0`, a control's height plus its padding and rule, `null`, `null`, `null` (a selection bar's background, top rule, padding, least height, corners, margin and font size: a tinted chip instead of a ruled band) |
| `$row-list-item-height`, `$row-list-item-padding`, `$row-list-hover-fill`, `$row-list-name-wrap` | `var(--control-height)`, `0 var(--space-3)`, 4% of `--text`, `false` (a row's least height, padding and hover fill; `true` wraps a name anywhere instead of cutting it to one line) |
| `$row-list-main-padding`, `$row-list-main-gap`, `$row-list-name-height` | `var(--space-1)`, `null`, `null` (a row's main part's vertical padding and the gap between its lines, and its name's least height) |
| `$pick-row-height`, `$pick-row-padding`, `$pick-row-gap`, `$pick-row-radius`, `$pick-row-text`, `$pick-row-hover-fill` | `var(--control-height)`, `0 var(--space-3)`, `var(--space-3)`, `var(--radius-sm)`, `null` (inherited), 4% of `--text` (a pick row's least height, padding, gap, corners, font size and hover fill) |
| `$pick-row-detail-ink`, `$pick-row-detail-text` | 70% of the row's colour, `var(--text-sm)` (its detail's colour and size) |
| `$selection-bar-inset-padding`, `$selection-bar-inset-margin` | `var(--space-3)`, `null` (an `inset` bar's side padding and margin) |
| `$selection-bar-count-ink`, `$selection-bar-count-weight`, `$selection-bar-count-padding` | `null` each (its count's colour, weight and start padding) |
| `$selection-bar-count-wrap` | `false` (the count is cut to one line; `true` wraps a count that does not fit, so the bar grows a line and its actions never shrink) |
| `$band-chrome-gap`, `$band-search-text` | `$toolbar-gap`, `var(--text-sm)` |
| `$action-overlay-radius`, `$action-overlay-padding` | `var(--radius-sm)`, `var(--space-3) var(--space-2)` (an overlay [`ActionRow`](https://docs.rs/sockel/latest/sockel/struct.ActionRow.html)'s corners and its start and end padding; an app whose rows are rounded gives it the row's end corners, e.g. `0 8px 8px 0`) |
| `$visibility-breakpoint` | `"phone"` (where `phone-only` and `desktop-only` switch) |
| `$breakpoints` | `()`, merged over sockel's: `narrow` 30rem, `small` 36rem, `phone` 40rem, `medium` 48rem, `compact` 52rem, `regular` 56rem, `large` 60rem, `tablet` 64rem, `wide` 84rem, `broad` 90rem, `desktop` 100rem |
| `$omit` | `()` (every part emitted; see [Leaving parts out](https://docs.rs/sockel/latest/sockel/#leaving-parts-out)) |
| `$features` | `null` (with [trug], `pkg:sockel/features`'s `$features`; see [Leaving parts out](https://docs.rs/sockel/latest/sockel/#leaving-parts-out)) |

`$breakpoints` moves or adds named breakpoints without restating the rest:
`$breakpoints: ("phone": 42rem)`.

<br>

### Cascade layers

An app whose Sass uses cascade layers puts sockel's rules in one too, or
they win over every layer of the app's. `layer`
wraps sockel's CSS in `@layer sockel { ... }`, and `layer_order` declares
the app's layers before it, so a layer meant to sit below sockel's (a reset)
comes first:

```rust
sockel_build::Config::new()
    .local("style/")
    .prefix("app-")
    .layer("sockel")
    .layer_order(["reset", "sockel", "components"])
    .compile();
```

The app's partials then put their own rules in their layers
(`@layer components { ... }`). Without either call the stylesheet is
unchanged.

<br>

### Leaving parts out

`$omit` names the optional parts an app never renders, and sockel leaves them
out of its CSS: a modifier no markup sets, a variant no button takes, the
tokens of a partial the app does not load. It is empty by default; a name
sockel does not have is an error. It trims components an app does render; one
it does not render is left out by its feature (see
[Choosing features](https://docs.rs/sockel/latest/sockel/#choosing-features)), with the `tokens.*` part only it
reads, and Loadable's `skeleton.loadable`.

A part its feature already leaves out is an error too, in `sockel-build` and
in the Sass given `$features`. `sockel_build::Usage` reports an omitted class
the app's own sources write.

Sass compiled with [trug] instead loads the tokens with the build's
features, and the same parts follow them:

```scss
@use "pkg:sockel/features" as f;
@use "pkg:sockel/tokens" with ($features: f.$features, $omit: ("layout.toolbar",));
```

<!-- check-variant: build.rs -->
```rust
fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
        .with("omit", r#"("layout.toolbar", "buttons.ghost")"#)
        .compile();
}
```

| Partial | Parts |
| --- | --- |
| tokens | `tokens.actions`, `tokens.card-grid`, `tokens.code-editor`, `tokens.color-select`, `tokens.drop`, `tokens.fields`, `tokens.filter-picker`, `tokens.media`, `tokens.panel`, `tokens.picker`, `tokens.seam`, `tokens.table`, `tokens.toast`, `tokens.toggles`, `tokens.tooltip`: the `:root` tokens only that partial reads (`--avatar-*` are `tokens.media`'s, `--action-overlay-*` are `tokens.actions`'s) |
| layout | `layout.toolbar` (the toolbar, its parts, the library toolbar and the `--band-*` tokens), `layout.grow` (with its bases), `layout.pages` (the workspace and library pages), `layout.plain-list`, `layout.lines`, `layout.flush` (a stack's or cluster's), `layout.gap-none`, `-hair`, `-snug`, `-tight`, `-medium`, `-loose`, `-wide`, `layout.card-compact`, `layout.card-dense`, `layout.card-link` |
| fields | `fields.field` ([`Field`](https://docs.rs/sockel/latest/sockel/struct.Field.html) itself, not a [`FormRow`](https://docs.rs/sockel/latest/sockel/struct.FormRow.html)), `fields.required`, `fields.help`, `fields.error` (with `reserve_error`), `fields.disabled`, `fields.busy`, `fields.inputs`, `fields.fieldset` (with the legend), `fields.check` |
| heads | `heads.trail` (`crumbs`), `heads.crumbs` (`back`, `meta`, `.sockel-compact`, `.sockel-one-row`), `heads.label`, `heads.list` |
| buttons | `buttons.bare`, `buttons.ghost`, `buttons.touch`, `buttons.fold`, `buttons.fold-wide`, `buttons.busy` (the spinner and `aria-busy`) |
| menu | `menu.widths` (`wide`, `wider`, `panel` and their tokens but `--menu-width-wide`, which Picker and SearchSelect read), `menu.tight`, `menu.danger`, `menu.current`, `menu.detail` (a row's detail and hint), `menu.heading`, `menu.note`, `menu.separator`, `menu.row-actions`, `menu.split-button`, `menu.context` (ContextMenu's target and point), `menu.marks` (a check or radio item's mark), `menu.disabled` (a disabled item) |
| skeleton | `skeleton.spinner`, `skeleton.loadable`, `skeleton.row`, `skeleton.group` (`count` above one), `skeleton.short` |
| inputs | `inputs.input` (`.sockel-input`), `inputs.textarea`, `inputs.mark` (an input box's mark and unit), `inputs.group` ([`InputGroup`](https://docs.rs/sockel/latest/sockel/struct.InputGroup.html) and `group_classes`), `inputs.native-select` (`native_select`), `inputs.small` and `inputs.touch` (the sizes), `inputs.busy`, `inputs.invalid` (`invalid` and `aria-invalid`), `inputs.disabled` (a disabled input box; a bare input's `:disabled` is kept) |
| notices | `notices.notice` ([`Notice`](https://docs.rs/sockel/latest/sockel/struct.Notice.html)), `notices.empty-state` ([`EmptyState`](https://docs.rs/sockel/latest/sockel/struct.EmptyState.html)), `notices.flanked` (an ErrorBox's icon and dismiss), `notices.causes` (an ErrorBox's `causes`) |
| pager | `pager.edges` (the first and last page), `pager.summary` (the summary pager), `pager.states` (`disabled` and `busy`), `pager.ranged` (the range text and button group), `pager.compact` (the compact pager's "3 / 35"), `pager.drop` (a page button as a drop target, `drop_page`) |
| popover | `popover.match-anchor` |
| badges | `badges.pill` (`Pill`, `StatusLabel`'s pill, `pill-label`), `badges.dot`, `badges.count`, `badges.status` (`StatusLabel`'s label, spinner and text) |
| tabs | `tabs.segmented`, `tabs.fill`, `tabs.stacked` (the stacked tabs and their icon and label), `tabs.nav` (`NavList`) |
| table | `table.sort-header` (`SortHeader`) |
| shell | `shell.bottom-nav` (`BottomNav`) |
| property-list | `property-list.inline-edit` (`InlineEdit`) |
| rows | `rows.pick-row` (`PickRow`), `rows.selection-bar` (`SelectionBar`), `rows.action-row` (`ActionRow`) |
| select | `select.filter` (the filter field and no-match text, `filter_from`), `select.sheet` (the phone sheet, `sheet`) |
| disclosure | `disclosure.heading` (the heading, `level`), `disclosure.detail` (the muted `summary`) |
| confirm | `confirm.inline` |
| drop | `drop.tall` |
| utilities | `utilities.focus-ring`, `utilities.numeric`, `utilities.scroll-area`, `utilities.truncate`, `utilities.clamp`, `utilities.phone-only`, `utilities.desktop-only` (Pager's `phone_steps: false` uses it) |

<br>

## The test-id attribute

Every component takes a `testid` prop and writes it to `data-testid`. An
app whose tests read another attribute sets `SOCKEL_TEST_ID_ATTR` when it
builds, in `.cargo/config.toml`:

```toml
[env]
SOCKEL_TEST_ID_ATTR = "data-test"
```

sockel reads it at compile time, so one app builds with one name, and a
change to it rebuilds sockel. The name is checked at compile time: letters,
digits, `_`, `:`, `.` and `-`, not starting with a digit, `.` or `-`. Unset,
the attribute is `data-testid`, as before (its place among a tag's attributes may differ). The
attribute is on the element when it mounts, and the props and their names
(`testid`, `opener_testid`, ...) do not change. Only the components'
elements move; an element the app writes itself keeps what it writes. See
[`OverlayHost`](https://docs.rs/sockel/latest/sockel/struct.OverlayHost.html) for
the host element, which takes it too.

<br>

## Against jitter: mixins and utilities

The shared rules every component builds on, so nothing changes size as its
state or content changes. Each is a mixin (`@use "sockel" as p;` then
`@include p.numeric;`) and, where an element needs it directly, a utility
class. Utilities load after the components, so one wins over a component's
rule of the same weight; none uses `!important`.

| Mixin | Utility (`sockel::class::`) | What it guarantees |
| --- | --- | --- |
| `focus-ring` | `.sockel-focus-ring` (`FOCUS_RING`) | A `:focus-visible` outline from the `--focus-ring-*` tokens: never a border, so focus adds nothing to the box. |
| `reserved-border($color: transparent, $width: var(--space-px))` | | The border is always there; a state changes only its colour. |
| `numeric` | `.sockel-numeric` (`NUMERIC`) | Tabular figures: a count, readout, pager or clock keeps its width as it changes. |
| `scroll-area` | `.sockel-scroll-area` (`SCROLL_AREA`) | Scrolls, with `scrollbar-gutter: stable`: content does not jump when it starts to scroll. Contains overscroll and may shrink in a flex column. |
| | `.sockel-toolbar` | Scrolls sideways with its scrollbar hidden, so its height does not change when its controls start to overflow; the edge with hidden controls fades (a CSS scroll-driven animation: browsers without `animation-timeline` show no fade). Wheel, touch and keyboard still scroll it. |
| `truncate` | `.sockel-truncate` (`TRUNCATE`) | One line, cut with an ellipsis. |
| `clamp($lines)` | `.sockel-clamp-2`, `-3`, `-4` (`CLAMP_2`...) | At most that many lines, the last cut with an ellipsis. |
| `visually-hidden` | `.sockel-visually-hidden` (`VISUALLY_HIDDEN`) | Read by a screen reader, takes no room (a hidden h1). |
| `phone-only`, `desktop-only` | `.sockel-phone-only`, `.sockel-desktop-only` | Shown only below `$visibility-breakpoint`, or only from it up. |
| `current($by: background)` | | Marks `[aria-current]` (not `"false"`) and `[aria-selected="true"]` by `background` (`--accent` under `--on-accent`), `color` or `underline`; never by weight, so the item keeps its width. |
| `scroll-fade` | | Scrolls sideways with the scrollbar hidden and the edge that has hidden content faded ([`Toolbar`](https://docs.rs/sockel/latest/sockel/struct.Toolbar.html), [`Tabs`](https://docs.rs/sockel/latest/sockel/struct.Tabs.html)). |

<br>

### Recipes for markup sockel does not render

A dependency's markup (a sign-in form, a user table) cannot wear sockel's
classes. The mixins below are the rules of the classes they name, which
include them, so such markup styled with them looks and behaves as
sockel's own: `.auth-card { @include p.card; }`.

| Mixin | The rule of |
| --- | --- |
| `stack($gap: var(--space-3))` | `.sockel-stack` (a gap of the scale for its modifiers) |
| `cluster($gap: var(--space-2), $wrap: true)` | `.sockel-cluster` (`$wrap: false` for `.sockel-nowrap`) |
| `card` | `.sockel-card` |
| `field`, `field-label` | `.sockel-field`, `.sockel-field-label` |
| `input`, `input-box` | `.sockel-input`; `input-box` is the box every text field shares (border, fill, sizes) |
| `native-select` | `.sockel-native-select` (with `input`, a native `<select>`) |
| `button($variant: secondary)` | `.sockel-button` with its states; `button-variant($variant)` is a variant's colours alone (`primary`, `ghost`, `quiet`, `danger`, `success`) |
| `table-base`, `table-head-cell`, `table-cell`, `table-row-hover` | a [`Table`](https://docs.rs/sockel/latest/sockel/struct.Table.html)'s |

A button styled this way lays out its own content (sockel's puts it in
`.sockel-button-content`): give the markup a gap of its own if it holds an
icon and a label.

Layers: `--layer-sticky`, `--layer-overlay`, `--layer-toast` and
`--layer-tooltip` are the one `z-index` scale for what floats, lowest first.
Popovers, menus, lists and dialogs all sit at `--layer-overlay`.

<br>

## Translations

sockel has no English in its code. Every word a component shows or announces
(button names, `aria-label`s, empty states, reorder announcements) is a
message in `locales/en-US.ftl` and `locales/sv-SE.ftl`, every id under the
`sockel-` prefix, exported as `sockel::phrasebookcatalog01::L10N` (a
[phrasebook-catalog] 0.1 `Catalog`). Components read the `Phrases` an app
provides with `ContextProvider<Phrases>` (phrasebook 0.2, `yew023`, the same
major version as sockel's). With no provider, or one without sockel's
catalog, they render the catalog's English and phrasebook warns once.

An app adds the catalog after its own:

```rust
let book = Phrasebook::builder()
    .language(Lang::EN_US, "English")
    .language(Lang::new("sv-SE"), "Svenska")
    .catalog(&APP)
    .catalog(&sockel::phrasebookcatalog01::L10N)
    .build();
```

- Later catalogs fill the ids earlier ones lack, so an app translates or
  rewords a sockel word by defining the same `sockel-*` id in its own
  `.ftl`. A language sockel does not ship (and the app does not translate)
  falls back as phrasebook says.
- Check the result with `phrasebook::check` (syntax, prefixes, duplicates,
  coverage) as for any catalog of the app's.
- Label props (`close_label`, `less_label`, `empty_label`, ...) are per-use
  overrides: `Option<AttrValue>`, `None` reading the catalog. `label="..."`
  in `html!` works as is; a struct literal or a field read needs
  `Some(..)` or `.as_deref()`. [`ConfirmRequest`](https://docs.rs/sockel/latest/sockel/struct.ConfirmRequest.html)'s `accept` and `cancel_label`
  are `Option` too, and `ConfirmRequest::new` leaves them `None`.
- [`Reorder`](https://docs.rs/sockel/latest/sockel/struct.Reorder.html)'s announcements are `sockel-reorder-*` messages with the
  position and count as numbers; `text` overrides them, [`use_reorder_text`](https://docs.rs/sockel/latest/sockel/fn.use_reorder_text.html)
  returns them in the app's language and `reorder_text` the catalog's English.

sockel's own rule: no user-visible text as a literal in a component. Every
word comes from the catalog, with numbers passed as numbers so the language
formats them. `tests/english_literals.rs` fails on a capitalised literal in
`src/` outside a short list of key names, and the catalog tests check
`en-US`/`sv-SE` parity and the ids the code asks for against the catalog,
both ways. The showcase provides the catalog and switches language (the
Translations page).

<br>

### Writing a library on sockel

A component library built on sockel ships its own catalog the way sockel
does: every id under its own prefix, read in each component with
`use_phrases_with(&L10N)`, and label props resolved with
[`label`](https://docs.rs/sockel/latest/sockel/phrasebookcatalog01/fn.label.html):

```rust
use phrasebook02::Phrases;
use phrasebookcatalog01::Catalog;
use yew::AttrValue;

pub static L10N: Catalog = Catalog::new("mylib", &[]).with_prefix("mylib-");

fn close(phrases: &Phrases, close_label: &Option<AttrValue>) -> AttrValue {
    sockel::phrasebookcatalog01::label(phrases, close_label, "mylib-close")
}
```

Apps add the library's catalog between their own and sockel's, so each
can reword the ones after it:
`.catalog(&APP).catalog(&mylib::L10N).catalog(&sockel::phrasebookcatalog01::L10N)`.
Apps that prune sockel's stylesheet also add `Usage::dependency("mylib")`
to their `Usage`, so classes the library uses are kept.

[phrasebook-catalog]: https://docs.rs/phrasebook-catalog

<br>

## Building props

Build a component's props with `html!` or `yew::props!`, which fill in
every prop you leave out. Nearly every props struct also implements
`Default`, so a struct literal written as `Props { label: .., ..Default::default() }`
keeps compiling when a release adds a prop. A required prop is the empty
value under `Default` (an empty label, a no-op callback), so name the ones
the component needs. `ImageProps` has no `Default`: its `ratio` has no
sensible default.

<br>

## Showcase

The [showcase](https://udoprog.github.io/sockel/) is a page per component,
with controls for its props and content, the code that renders it as set,
and a narrow frame (drag its corner to widen it) beside a wide one. It is
built from `showcase/` in the repository; run it locally with [trug]:

```sh
cd showcase
trug serve --port 8095
```

The panel in the window's corner runs the stability probe: it steps every
control through its values, focuses and clicks what is in the frames, and
measures the boxes the page declares must hold. It reads `probe: stable`
when none moved or resized by more than half a pixel, and lists each change
otherwise. Its `data-probe` attribute carries the verdict, and `?probe`
before a page's `#id` runs it on load. Hover stays a manual check.

A new component lands with its page: add it to `PAGES` in
`showcase/src/pages/mod.rs` with the names it covers, the code it shows
(`showcase/src/code.rs`) and the boxes it `holds`, or `cargo test
--workspace` fails. `showcase/scripts/trug-css.sh` checks that `trug css`
builds the same stylesheet as `showcase/build.rs`.

<br>

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --workspace -- -D warnings
cargo test --workspace
bash scripts/check-features.sh
bash scripts/check-packaged.sh
(cd showcase && trug build && sh scripts/trug-css.sh)
```

[Yew]: https://yew.rs
[sasso]: https://crates.io/crates/sasso
[trug]: https://crates.io/crates/trug
