# sockel

Opinionated layout primitives and components for [Yew] apps, built so
components keep their size and don't jitter as their content changes.

Each component comes with its Sass under `style/`, and its variants are
modifier classes set by props, so the markup and the styles stay in step.

sockel was called plinth until 2026-11-17, and its history starts as
`crates/plinth` in characterize.

[`Styles`] adds the compiled stylesheet ([`STYLESHEET`]) to the page. An
app with Sass of its own builds it with sockel-build after sockel's, into
one stylesheet, and passes that to [`Styles`] (see
[Getting started](#getting-started)).
The colours are the app's: sockel reads `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`, and derives
overridable defaults for `--danger`, `--success`, `--warning`,
`--on-accent`, `--surface-raised` and `--scrim` from them.

Each component (or small family) is a Cargo feature that gates its Rust,
its class constants and its CSS together; `full`, the default, has all.

The utilities against jitter (`.sockel-numeric`, `.sockel-scroll-area`,
`.sockel-truncate`, `.sockel-clamp-*`, `.sockel-visually-hidden`,
`.sockel-phone-only`, `.sockel-desktop-only`, `.sockel-focus-ring`) are
classes only, in [`class`]; their mixins are in
[Against jitter](#against-jitter-mixins-and-utilities).

<br>

## Getting started

An app depends on `sockel` and builds its one stylesheet, sockel's CSS and
the app's own Sass, with `sockel-build` in its `build.rs`. No Sass step
runs anywhere else, and no path leads into sockel's checkout. The blocks
below are the files of a complete app (`scripts/check-packaged.sh` builds
them against sockel's packages).

<br>

### Dependencies

In the app's `Cargo.toml`, sockel with the features the app renders (see
[Choosing features](#choosing-features); leave out `default-features` and
`features` for all of them), and `sockel-build` to build and to test:

<!-- check: Cargo.toml -->
```toml
[dependencies]
sockel = { version = "0.1", default-features = false, features = ["button", "layout"] }
yew = { version = "0.23", features = ["csr"] }

[build-dependencies]
sockel-build = "0.1"

[dev-dependencies]
sockel-build = "0.1"
```

<br>

### Developing against a checkout

sockel is not on a registry yet, so point both crates at a checkout once, in
the `Cargo.toml` at the workspace root (the app's own, when it is not in a
workspace):

```toml
[patch.crates-io]
sockel = { path = "../sockel" }
sockel-build = { path = "../sockel/sockel-build" }
```

The dependency lines stay as above, and dropping the patch later switches to
the registry. Nothing else names the checkout.

<br>

### Styles

The app's Sass lives in a directory, say `style/`, with an `_index.scss`
(or `index.scss`) that `@use`s its partials. It loads sockel's tokens and
mixins as `"sockel"`:

<!-- check: style/_index.scss -->
```scss
@use "sockel" as p;

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

`"sockel"` is the `style/` of the sockel package the app builds against.
It emits no second copy of sockel's CSS: sockel is compiled first in the
same build. Configure sockel's tokens through the build instead of `@use
"sockel" with (...)` (see [Configuration](#configuration)).

Every class the app's Sass defines that is not sockel's must start with the
app's prefix (`app-`); one that does not is a build error, so the app's
classes never collide with sockel's.
Classes a dependency's markup writes (a sign-in form the app styles) are
the exception: `Config::foreign(["auth-*"])` lets the app style them
unprefixed (a trailing `*` is a prefix), and they get no constant.

<br>

### `build.rs`

At the package root:

<!-- check: build.rs -->
```rust
fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
        .compile();
}
```

It writes `style.css` and `classes.rs` to `OUT_DIR`, and reruns when
`build.rs`, anything under `style/` (partials and new files included) or
sockel's Sass changes.

<br>

### The class constants and the stylesheet

`classes.rs` holds the app's classes, named without the prefix
(`.app-sidebar` is `class::SIDEBAR`); sockel's stay `sockel::class::...`.
Every sockel component takes a `class` prop, so pass the constants there,
and a class the Sass does not define does not compile. The stylesheet is
mounted once at the root with [`Styles`]'s `css`:

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

fn main() {
    yew::Renderer::<App>::new().render();
}
```

<br>

### No Sass elsewhere

`index.html` links no stylesheet: no `<link data-trunk rel="scss">`, and no
`[tools] sass` in `Trunk.toml`.

```html
<link data-trunk rel="rust" href="Cargo.toml" />
```

<br>

### A guard against dead classes

A class the Sass defines and no Rust writes is dead weight.
`sockel_build::Usage` finds them in a test, so an app needs no guard of its
own:

<!-- check: tests/style_classes.rs -->
```rust
use sockel_build::{Config, Usage};

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .assert_clean(
            &Config::new()
                .style(env!("SOCKEL_STYLE"))
                .local("style/")
                .prefix("app-"),
        );
}
```

`SOCKEL_STYLE` is sockel's style directory, which `compile()` hands the
package's crates. `.dynamic(["app-tone-*"])` allows classes built at runtime
(a trailing `*` is a prefix) and `.foreign([...])` ones a dependency writes.
A class counts as written when a source names its constant
(`class::SIDEBAR`) or the class in a string literal; comments and each
item marked `#[cfg(test)]` (a test module, function or `use`) do not count. The test fails on a dead class
and on an allowed pattern that matches no class, and prints the class,
single-use and dead counts. `STYLE_INVENTORY=PATH` also writes
`class<TAB>uses` lines to `PATH`. sockel runs it on itself and on the
showcase.

<br>

### How sockel-build finds sockel

sockel's own `build.rs` passes two values to the `build.rs` of every package
depending on it directly (Cargo's `links` metadata): `DEP_SOCKEL_FEATURES`,
its enabled features, and `DEP_SOCKEL_STYLE`, the `style/` directory inside
its package. `Config` builds those features from those sources, so the
app's stylesheet always matches the sockel it links. A package that reaches
sockel only through another crate is told neither: it names them with
`Config::features([...])` and `Config::style(dir)`.

<br>

### One stylesheet per app

Both `Styles` fill the same element, `<style id="sockel-styles">`
(`sockel::STYLE_ID`): with `css` it sets the element's text to the app's
stylesheet, which holds sockel's; without it, it adds sockel's only when the
element is missing. So whichever mounts first (the app's, or a
`<sockel::Styles/>` inside a component), the page ends up with the app's
stylesheet, once. Never link or inject sockel's CSS another way beside it.

An app without Sass of its own needs no `build.rs`: it mounts
`<sockel::Styles/>` once at its root, which adds `sockel::STYLESHEET`, and
uses `sockel::class`.

Every class sockel styles starts with `sockel-`, modifiers included
(`.sockel-stack.sockel-tight`), so none collides with an app's classes. A
constant is named after its class without the prefix, as an app's are:
`.sockel-page-head-title` is `sockel::class::PAGE_HEAD_TITLE`,
`.sockel-tight` is `sockel::class::TIGHT`.

The showcase (`showcase/build.rs`, `showcase/style/`) is set up this way.

<br>

## Migrating from trunk's Sass step

An app that compiled its styles with trunk (`<link data-trunk rel="scss">`
and `@use "../../sockel/style/..."`) moves to the getting-started setup:

1. **Dependencies.** Replace `sockel = { path = ... }` with the lines in
   [Dependencies](#dependencies) and the patch in
   [Developing against a checkout](#developing-against-a-checkout).
2. **`@use` lines.** Delete every `@use` of sockel's checkout (`"../…/sockel/style"`,
   `"../…/sockel/style/tokens"`, a partial): `sockel-build` compiles sockel's
   CSS first, with the app's features. A partial that needs sockel's
   mixins or token functions writes `@use "sockel" as p;` and calls
   `p.below(...)` and the like. A `with (...)` on the old `@use` moves to
   `Config::with` in `build.rs` (see [Configuration](#configuration)).
3. **The prefix.** Every class the app's Sass defines must start with one
   prefix (`army-`), given to `Config::prefix`. Rename the app's classes in
   the Sass and the markup together; a rule that refines a sockel class
   (`.sockel-card.army-sidebar`) keeps sockel's part as it is.
4. **Class literals.** Add the `mod class` above and replace the app's
   class strings in Rust with its constants (`"army-sidebar"` becomes
   `class::SIDEBAR`, `classes!("sockel-card", "army-sidebar")` becomes
   `classes!(sockel::class::CARD, class::SIDEBAR)`). A misspelt or removed
   class then fails to compile, and the dead-class test finds the rest.
5. **trunk.** Remove `<link data-trunk rel="scss">` from `index.html` and
   `[tools] sass` from `Trunk.toml`, and mount `<sockel::Styles
   css={STYLESHEET} />` once at the root.

`sockel-build` compiles with [sasso], a Rust port of dart-sass, and its
CSS matches dart-sass's. A block's declarations written after a nested
rule or an `@include` of rules come out in a second block after those
rules, so a `padding` after `@include p.below("phone") { padding: ... }`
wins over the media query. A dart-sass feature sasso lacks fails in
`build.rs` with the error and its line.

Including sockel's Sass from a checkout (`@use "../sockel/style"`) does not
work once sockel comes from a registry, and is not supported for apps.

<br>

## Choosing features

sockel has a Cargo feature per component, or per family of components that
only work together. `full`, the default, is all of them. An app that
renders a few turns the default off and names those; each feature brings in
the ones it needs:

```toml
[dependencies]
sockel = { version = "0.1", default-features = false, features = ["field", "input", "layout"] }
```

A feature gates a component's Rust and its CSS together. [`STYLESHEET`] and
`sockel::class` hold only the enabled features' rules and classes, so a
component or class whose feature is off does not compile, rather than
rendering unstyled. The tokens, hooks, utilities and the menus' shared
surface are always in, and so is the `:root` tokens' part of a feature
(`tokens.table`, ...) only while a feature that reads it is on.

An app compiling its own Sass through `sockel-build` gets the same subset
without asking (see [How sockel-build finds sockel](#how-sockel-build-finds-sockel)).
A package that reaches sockel only through another
crate is not told, and gets every feature's CSS, the whole stylesheet;
`Config::features([...])` names its features instead. Cargo builds one set
of features for everything built together, so a workspace whose other
member wants more gets the union.

| Feature | Brings in | Gives | CSS alone |
| --- | --- | --- | --- |
| `action-group` | | [`ActionGroup`] | 3.7 KB |
| `badge` | `icons` | [`Pill`], [`Badge`], [`Dot`], [`Count`], [`StatusLabel`] | 9.6 KB |
| `button` | `icons`, `skeleton` | [`Button`], [`IconButton`] | 11.9 KB |
| `card-grid` | | [`CardGrid`] | 4.1 KB |
| `color-select` | `select` | [`ColorSelect`] | 48.1 KB |
| `confirm` | `action-group`, `badge`, `button`, `notice`, `overlay` | [`Confirm`], [`ConfirmPopover`], [`ConfirmButton`], [`ConfirmInline`], [`ConfirmHost`] | 37.0 KB |
| `disclosure` | `head`, `icons` | [`Disclosure`] | 15.2 KB |
| `drop-zone` | `button` | [`DropZone`] | 14.3 KB |
| `field` | | [`Field`], [`FormRow`], [`FormRows`], [`Fieldset`], [`FieldCheck`], [`FieldInputs`] | 6.3 KB |
| `head` | `layout` | [`Head`], [`PageHead`], [`SectionHead`] | 12.8 KB |
| `icons` | | [`Icon`], [`IconSet`], [`Glyph`] | 4.0 KB |
| `input` | `icons` | [`TextInput`], [`Textarea`], [`SearchField`], [`InputGroup`], [`SecretInput`] | 10.2 KB |
| `layout` | | [`Stack`], [`Cluster`], [`Card`], [`Inset`], [`Toolbar`] | 9.0 KB |
| `library-toolbar` | `input`, `layout` | [`LibraryToolbar`] | 15.8 KB |
| `loadable` | `notice`, `skeleton` | [`Loadable`] | 22.2 KB |
| `media` | `icons` | [`Image`], [`Avatar`] | 5.6 KB |
| `menu` | `button`, `icons`, `overlay` | [`Menu`], [`RowActions`], [`SplitButton`], [`ContextMenu`] | 31.1 KB |
| `notice` | `badge`, `button` | [`Notice`], [`ErrorBox`], [`EmptyState`] | 21.9 KB |
| `overlay` | `action-group`, `button`, `head`, `icons` | [`OverlayHost`], [`Popover`], [`Dialog`] | 25.7 KB |
| `pager` | `icons` | [`Pager`], [`PagerSummary`] | 6.5 KB |
| `panel` | | [`Panel`] | 4.8 KB |
| `picker` | `overlay`, `pager`, `select` | [`Picker`], [`PickerMenu`], [`PickerDialog`], [`ChoiceList`], [`FilterPicker`] | 50.9 KB |
| `reorder` | `icons`, `rows`, `tooltip` | [`Reorder`], [`DragHandle`] | 33.3 KB |
| `rows` | `button` | [`RowList`], [`PickRow`], [`ActionRow`], [`SelectionBar`] | 16.9 KB |
| `search-select` | `select` | [`SearchSelect`] | 45.1 KB |
| `seam` | | [`Seam`] | 5.2 KB |
| `select` | `input`, `menu`, `tooltip` | [`Select`], [`Listbox`] | 43.9 KB |
| `skeleton` | `icons`, `media` | [`Skeleton`], [`Spinner`] | 7.4 KB |
| `stepper` | `input` | [`Stepper`] | 11.2 KB |
| `table` | `icons` | [`Table`], [`SortHeader`] | 7.1 KB |
| `tabs` | | [`Tabs`], [`SubTabs`], [`NavList`] | 6.9 KB |
| `toast` | `badge`, `button`, `overlay` | [`ToastHost`], [`Toast`], [`SaveBar`] | 34.1 KB |
| `toggles` | `icons` | [`Checkbox`], [`Switch`], [`Choice`], [`ChipList`], [`ChipToggle`] | 10.7 KB |
| `tooltip` | `overlay` | [`Tooltip`], [`HoverCard`], [`ReasonButton`] | 26.8 KB |

"CSS alone" is `STYLESHEET` with only that feature and what it brings in;
with none it is 3.4 KB, and with `full` 103 KB. `scripts/check-features.sh`
checks that each feature builds alone and that its CSS holds no other
feature's rules.

<br>

## Theming

The app supplies seven colour tokens: `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`. sockel derives eight more
from them, each a default the app overrides by name (or with `Config::with`,
as `$danger` and the rest):

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
| `--panel-border` | `--border` | a [`Panel`]'s ring |
| `--panel-tint-text` | `--text` | a tint `Panel`'s text |
| `--band-chrome-ground` | `--surface` | a chrome `ToolBand`'s ground |
| `--menu-item-icon-ink` | `currentColor` | a menu row's icon (an accent beside two-line rows, say) |

The status hues are mixed toward `--text`, so they keep their contrast on a
light and a dark background alike. The defaults are computed on `:root`: an
app that sets its colours on a narrower element (a dark panel) sets these
there too. An app whose theme defines one of `--danger` to
`--card-border-width` and `--menu-item-icon-ink` itself sets its variable to
`false` (`$danger: false`), and sockel leaves its default off `:root`.
`--focus-ring-color` (default `--accent`) colours the focus ring.

sockel's own scales are custom
properties on `:root` too (`--text-*`, `--space-*`, `--radius-*`,
`--control-height`, `--page-*` and the rest, listed under Configuration), and
every rule reads them, so an app retunes the embedded stylesheet by
overriding them in its own CSS, loaded after it or with a more specific
selector:

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

Breakpoints and the component settings (`$card-padding`,
`$card-padding-phone`, `$toolbar-gap`, `$button-padding`, `$button-padding-small`,
`$button-primary-weight`, `$button-radius`, `$button-gap`, `$button-press-shade`,
`$button-transition`, `$button-colours`, `$button-text`, `$button-text-small`,
`$button-content-gap`, `$button-icon-offset`, `$button-phone-height`, `$input-hover-edge`,
`$input-focus-edge`, `$input-ring-offset`, `$input-padding-inline`, `$input-text`,
`$input-phone-height`, `$dialog-padding`, `$dialog-padding-phone`,
`$dialog-body-gap`, `$dialog-shadow`, `$dialog-head-padding`, `$dialog-head-rule`,
`$dialog-body-padding`, `$dialog-title-size`, `$dialog-title-weight`, `$dialog-centred`,
`$dialog-max-height`, `$dialog-scroll-gutter`, `$dialog-sheet-edge`, `$popover-padding`, `$popover-shadow`,
`$popover-radius`, `$popover-sheet-edge`, `$popover-scroll-gutter`, `$confirm-gap`,
`$visibility-breakpoint`) are Sass-only: an app built with
`sockel-build` sets them with `Config::with`.

<br>

## Configuration

sockel's values are defaults. Every token is a `!default` variable of
`style/_tokens.scss`, emitted as its custom property on `:root`. An app sets
its own in `build.rs`, before `compile()`; only values change, never
classes:

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
the custom properties, so a page can still override them at runtime.

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
| `$glyph-size` | `1em` |
| `$glyph-phone-size` | `null` (`--glyph-size` below the phone breakpoint; `null` keeps `$glyph-size`) |
| `$skeleton-duration` | `1.4s` |
| `$pager-gap`, `$pager-button-radius`, `$pager-button-ink`, `$pager-button-text` | `var(--space-1)`, `var(--radius-sm)`, `var(--text)`, `null` (inherited) (a pager's gap and its buttons' corners, ink and size) |
| `$pager-button-hover-fill`, `$pager-button-hover-ink`, `$pager-current-fill`, `$pager-current-ink`, `$pager-step-ink`, `$pager-phone-height` | 12% of `--accent`, `null` (as at rest), `var(--accent)`, `var(--on-accent)`, `null` (the button's), `null` (a button's hover, the current page's fill and ink, the chevrons' ink, and the controls' height below the phone breakpoint) |
| `$skeleton-fill`, `$skeleton-edge`, `$skeleton-radius` | 8% of `--text` over `--surface`, `null` (none), `var(--radius-sm)` (a skeleton's fill, the 1px line inside its edge and its corners) |
| `$skeleton-line-height`, `$skeleton-line-inset`, `$skeleton-phone-height` | `1lh`, `null` (a fifth of a line), `null` (a line's height and the transparent inset that thins it; a block's height below the phone breakpoint) |
| `$check-size`, `$switch-width`, `$switch-height`, `$choice-width` | `1.25rem`, `2.5rem`, `1.5rem`, `6rem` |
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
| `$button-disabled-cursor` | `not-allowed` (a disabled button's cursor) |
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
| `$focus-ring-color`, `$focus-ring-width`, `$focus-ring-offset` | `var(--accent)`, `2px`, `2px` |
| `$tooltip-max-width` | `24rem` |
| `$picker-popup-width` | `var(--menu-width-wide)` (a Picker's or PickerMenu's list's least width; never measured from its text) |
| `$listbox-max-width`, `$listbox-active-fill` | `var(--popover-max-width)`, `color-mix(in srgb, var(--text) 7%, transparent)` (a listbox's widest, a Select's list included, and its active row's fill: the menu's) |
| `$listbox-max-height`, `$listbox-shadow`, `$listbox-group-name-transform`, `$listbox-group-name-letter-spacing` | `null` (a listbox's tallest, under the room beside its opener, which it always keeps within; its shadow, `null` keeping the list's `--shadow-1`; its group headings' `text-transform` and `letter-spacing`; `null` leaves each as without) |
| `$select-sheet-row-height` | `null` (a Select's rows in its phone sheet, its close button too; `null` keeps `--control-height`, `var(--touch-height)` gives a finger room) |
| `$layer-sticky`, `$layer-overlay`, `$layer-dialog`, `$layer-toast`, `$layer-tooltip` | `10`, `100`, `200`, `300`, `400` (`$layer-dialog` is kept for compatibility and unused: dialogs sit at `$layer-overlay`) |
| `$menu-width`, `$menu-width-wide`, `$menu-width-wider`, `$menu-width-panel` | `12rem`, `16rem`, `20rem`, `34rem` |
| `$menu-edge`, `$menu-ground`, `$menu-shadow`, `$menu-padding`, `$menu-text` | `null` each (a menu's border colour, background, shadow, padding and font size; `null` keeps the list's: `--border`, `--surface-raised`, `--shadow-1`, `--nest-inset`, inherited) |
| `$menu-scroll-gutter` | `null` (a menu's `scrollbar-gutter`; `null` keeps the list's `stable`, `auto` gives the room to the rows until the menu scrolls) |
| `$menu-item-height`, `$menu-item-padding`, `$menu-item-gap`, `$menu-item-radius`, `$menu-item-hover-fill` | `null` each (a menu row's least height, padding, gap, corners and hover/focus fill; `null` keeps the row's: `--control-height`, `var(--space-1) var(--space-3)`, `var(--space-2)`, the nested radius, 7% of `--text`) |
| `$menu-item-edge` | `null` (a menu row's reserved 1px border, its focus edge; `outline` reserves none and draws the focus edge as a 1px inset outline in the same place, so a row taller than its least height is its content's height) |
| `$menu-radius` | `null` (a menu's corners; `null` keeps `--radius-md`) |
| `$menu-item-phone-height` | `null` (a menu row's least height below the phone breakpoint, as `$button-phone-height` is a button's; `null` keeps the one above it) |
| `$menu-item-mark-ink` | `null` (a check's or radio's mark; `null` keeps `--accent`, `currentColor` draws it as the row's text) |
| `$input-hover-edge` | `var(--accent)` (a text field's edge on hover: `.sockel-input`, `.sockel-textarea`, `.sockel-input-box`) |
| `$input-focus-edge` | `null` (its edge while focused; `null` keeps the edge it has) |
| `$input-ring-offset` | `null` (its focus ring's offset; `null` keeps `--focus-ring-offset`, a negative one draws the ring over the edge) |
| `$input-padding-inline` | `var(--space-2)` (its inline padding) |
| `$input-text` | `null` (its text size; `null` inherits it) |
| `$input-phone-height` | `null` (its height below the phone breakpoint, as `$button-phone-height` is a button's; `null` keeps the one above it) |
| `$danger`, `$success`, `$warning`, `$on-accent`, `$surface-raised`, `$scrim`, `$field-border`, `$field-bg`, `$card-border`, `$card-border-width`, `$band-chrome-ground`, `$panel-border`, `$panel-tint-text`, `$menu-item-icon-ink` | see [Theming](#theming) |
| `$panel-text`, `$panel-space-after`, `$panel-tint-space-after`, `$panel-tint-radius` | `1em` (its parent's), `0`, `var(--panel-space-after)`, `var(--radius-pill)` |
| `$selection-bar-ground`, `$selection-bar-rule`, `$selection-bar-padding`, `$selection-bar-min-height`, `$selection-bar-radius`, `$selection-bar-margin`, `$selection-bar-text` | `null`, `var(--space-px) solid var(--border)`, `var(--space-2) 0`, a control's height plus its padding and rule, `null`, `null`, `null` (a selection bar's background, top rule, padding, least height, corners, margin and font size: a tinted chip instead of a ruled band) |
| `$pick-row-height`, `$pick-row-padding`, `$pick-row-gap`, `$pick-row-radius`, `$pick-row-text`, `$pick-row-hover-fill` | `var(--control-height)`, `0 var(--space-3)`, `var(--space-3)`, `var(--radius-sm)`, `null` (inherited), 4% of `--text` (a pick row's least height, padding, gap, corners, font size and hover fill) |
| `$pick-row-detail-ink`, `$pick-row-detail-text` | 70% of the row's colour, `var(--text-sm)` (its detail's colour and size) |
| `$selection-bar-inset-padding`, `$selection-bar-inset-margin` | `var(--space-3)`, `null` (an `inset` bar's side padding and margin) |
| `$selection-bar-count-ink`, `$selection-bar-count-weight`, `$selection-bar-count-padding` | `null` each (its count's colour, weight and start padding) |
| `$selection-bar-count-wrap` | `false` (the count is cut to one line; `true` wraps a count that does not fit, so the bar grows a line and its actions never shrink) |
| `$band-chrome-gap`, `$band-search-text` | `$toolbar-gap`, `var(--text-sm)` |
| `$action-overlay-radius`, `$action-overlay-padding` | `var(--radius-sm)`, `var(--space-3) var(--space-2)` (an overlay [`ActionRow`]'s corners and its start and end padding; an app whose rows are rounded gives it the row's end corners, e.g. `0 8px 8px 0`) |
| `$visibility-breakpoint` | `"phone"` (where `phone-only` and `desktop-only` switch) |
| `$breakpoints` | `()`, merged over sockel's: `narrow` 30rem, `small` 36rem, `phone` 40rem, `medium` 48rem, `compact` 52rem, `regular` 56rem, `large` 60rem, `tablet` 64rem, `wide` 84rem, `broad` 90rem, `desktop` 100rem |
| `$omit` | `()` (every part emitted; see [Leaving parts out](#leaving-parts-out)) |

`$breakpoints` moves or adds named breakpoints without restating the rest:
`$breakpoints: ("phone": 42rem)`.

<br>

### Cascade layers

An app whose Sass puts its rules in cascade layers puts sockel's in one too:
unlayered, sockel's rules would win over every layer of the app's. `layer`
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
tokens of a partial the app does not load. It is empty by default, and every
part is emitted; a name sockel does not have is an error. It is for the
finer parts of components an app does render; a component it does not
render is left out by its feature (see [Choosing features](#choosing-features)),
which also leaves out the `tokens.*` part only it reads, and Loadable's
`skeleton.loadable`.

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
| tokens | `tokens.actions`, `tokens.card-grid`, `tokens.color-select`, `tokens.drop`, `tokens.fields`, `tokens.filter-picker`, `tokens.media`, `tokens.panel`, `tokens.picker`, `tokens.seam`, `tokens.table`, `tokens.toast`, `tokens.toggles`, `tokens.tooltip`: the `:root` tokens only that partial reads (`--avatar-*` are `tokens.media`'s, `--action-overlay-*` are `tokens.actions`'s) |
| layout | `layout.toolbar` (the toolbar, its parts, the library toolbar and the `--band-*` tokens), `layout.grow` (with its bases), `layout.pages` (the workspace and library pages), `layout.plain-list`, `layout.lines`, `layout.flush` (a stack's or cluster's), `layout.gap-none`, `-hair`, `-snug`, `-tight`, `-medium`, `-loose`, `-wide`, `layout.card-compact`, `layout.card-dense`, `layout.card-link` |
| fields | `fields.field` ([`Field`] itself, not a [`FormRow`]), `fields.required`, `fields.help`, `fields.error` (with `reserve_error`), `fields.disabled`, `fields.busy`, `fields.inputs`, `fields.fieldset` (with the legend), `fields.check` |
| heads | `heads.trail` (`crumbs`), `heads.crumbs` (`back`, `meta`, `.sockel-compact`, `.sockel-one-row`), `heads.label`, `heads.list` |
| buttons | `buttons.bare`, `buttons.ghost`, `buttons.touch`, `buttons.fold`, `buttons.fold-wide`, `buttons.busy` (the spinner and `aria-busy`) |
| menu | `menu.widths` (`wide`, `wider`, `panel` and their tokens but `--menu-width-wide`, which Picker and SearchSelect read), `menu.tight`, `menu.danger`, `menu.current`, `menu.detail` (a row's detail and hint), `menu.heading`, `menu.note`, `menu.separator`, `menu.row-actions`, `menu.split-button`, `menu.context` (ContextMenu's target and point), `menu.marks` (a check or radio item's mark), `menu.disabled` (a disabled item) |
| skeleton | `skeleton.spinner`, `skeleton.loadable`, `skeleton.row`, `skeleton.group` (`count` above one), `skeleton.short` |
| inputs | `inputs.input` (`.sockel-input`), `inputs.textarea`, `inputs.mark` (an input box's mark and unit), `inputs.group` ([`InputGroup`] and `group_classes`), `inputs.native-select` (`native_select`), `inputs.small` and `inputs.touch` (the sizes), `inputs.busy`, `inputs.invalid` (`invalid` and `aria-invalid`), `inputs.disabled` (a disabled input box; a bare input's `:disabled` is kept) |
| notices | `notices.notice` ([`Notice`]), `notices.empty-state` ([`EmptyState`]), `notices.flanked` (an ErrorBox's icon and dismiss), `notices.causes` (an ErrorBox's `causes`) |
| pager | `pager.edges` (the first and last page), `pager.summary` (the summary pager), `pager.states` (`disabled` and `busy`) |
| popover | `popover.match-anchor` |
| confirm | `confirm.inline` |
| drop | `drop.tall` |
| utilities | `utilities.focus-ring`, `utilities.numeric`, `utilities.scroll-area`, `utilities.truncate`, `utilities.clamp`, `utilities.phone-only`, `utilities.desktop-only` (Pager's `phone_steps: false` uses it) |

<br>

## Components

- [`Stack`] (`.stack`): a column, its blocks a gap apart.
- [`Cluster`] (`.cluster`): a row that wraps, its items centred.
- [`ActionGroup`] (`.action-group`): the buttons ending a form, a panel
  or a section.
- [`Button`] (`.button`): a button or a link styled as one, its
  variants and sizes, an icon slot, icon-only and fold, pressed and busy,
  all in the same box.
- [`Checkbox`] (`.checkbox`) and [`Switch`] (`.switch`): an on or off
  toggle in a box of a fixed size.
- [`Choice`] (`.choice`): a segmented pick-one group, its options of one
  width; [`ChipToggle`] (`.chip`) and [`ChipList`] (`.chip-list`): filter
  chips with a count.
- [`Field`] (`.field`), [`FormRows`] (`.form-rows`) with [`FormRow`],
  [`Fieldset`] and [`FieldCheck`]: a label, control, hint and an error
  line that can keep its room.
- [`TextInput`] (`.input`), [`Textarea`] (`.textarea`), [`SearchField`]
  (`.input-box`) and [`InputGroup`] (`.input-group`): text controls at a
  control's height, a unit or a clear button inside the field, controls
  joined in a row; [`native_select`] (`.native-select`) for a native
  `<select>` in a field's look.
- [`Stepper`] (`.stepper`): a number between a minus and a plus button,
  its room that of the longest bound; [`SecretInput`] (`.input-box`): a
  password with show, copy and clear slots that keep their room.
- [`IconButton`] (`.iconed`, `.trailing`): a button with an icon, which
  the app supplies and styles.
- [`Icon`] (`.glyph` or `.name`), [`IconSet`], [`use_icon`]: sockel's and the app's icons, from
  the set an app provides or a built-in one, in a fixed box.
- [`Image`] (`.image`): a picture in a box of a fixed ratio, with its
  placeholder; [`Avatar`] (`.avatar`): a portrait of a fixed size, its
  initials behind it.
- [`Notice`] (`.notice`): an inline message in a tone, its action and
  close; [`ErrorBox`] (`.error-box`): an error, its cause and a retry;
  [`EmptyState`] (`.empty-state`): what stands where there is nothing,
  compact for a row.
- [`DropZone`] (`.drop-zone`): a target for dropped or pasted files, or
  picked with its button, with over, refused, busy and error states drawn
  by an overlay, so its box never changes; [`accepts`] and [`refusal`]
  filter by type and size.
- [`Pill`] (`.pill`), [`Badge`] (`.badge`), [`Dot`] (`.dot`) and [`Count`]
  (`.count`): short marks of a fixed box, in a [`Tone`];
  [`StatusLabel`] (`.status-label`): a status as wide as its widest state.
- [`RowList`] (`.row-list`) with [`RowListItem`]: rows of a name, a detail
  and what ends them, divided by lines that add no height; [`PickRow`]
  (`.pick-row`): a row that is one press; [`SelectionBar`]
  (`.selection-bar`): the picked count and its actions under a list;
  [`ActionRow`] (`.action-row`): actions on what stands around them, over
  a row's end as an overlay; [`CardGrid`] (`.card-grid`): cards in as many
  columns as fit.
- [`Reorder`] (`.reorder`) with [`ReorderRow`]s or [`ReorderItem`] children
  (their grip a [`ReorderHandle`]): rows moved by a drag on
  their [`DragHandle`] (`.drag-handle`) or by keyboard, nothing reflowing
  until the drop, each step announced.
- [`Spinner`] (`.spinner`): a spinning glyph in the icon box;
  [`Skeleton`] (`.skeleton`): a placeholder of the box its content will
  have; [`Loadable`] (`.loadable`): loading, empty, error and ready in
  one box that keeps its height.
- [`Table`] (`.table`) with [`TableRow`] and [`TableCell`]: a table in a
  scroll area, its bulk actions' room reserved; [`SortHeader`]
  (`.sort-header`): a column heading whose sort mark keeps its box.
- [`Pager`] (`.pager`): page buttons in a fixed number of slots, first,
  previous, numbers with gaps, next and last, so it keeps its width from
  the first page to the last; [`Disclosure`] (`.disclosure`): a summary
  row that toggles its body, one height shut or open.
- [`ToastHost`] (`.toast-host`) and [`Toast`] (`.toast`): short messages
  with an undo, fixed over the window's bottom so they move nothing, their
  clock held under the pointer or focus; [`SaveBar`] (`.save-bar`): a
  form's unsaved changes, save and discard, in the flow at one height,
  with [`UnloadGuard`] for the browser's own prompt.
- [`Toolbar`] (`.toolbar`): one line of controls; `band` wraps them
  instead (`.band`, with `.chrome` and `.search`), in [`ToolbarPart`]s
  (`.toolbar-part`, `.ruled`) that each keep to one line.
- [`Panel`] (`.panel-box`): a ringed ground over a list or a view holding
  its filters or a short form, as a column or a line, tinted or bare.
- [`Seam`] (`.seam`): two panes and the divider dragged between them, each
  pane keeping its least size.
- [`use_width`]: an element's width, at most once a frame.
- [`Tabs`] (`.tabs`) and [`SubTabs`]: a tab list, the selected tab marked by
  an underline inside its box; [`NavList`] (`.nav-list`): a side or settings
  navigation, the current entry marked by `aria-current`.
- [`Card`] (`.card`): a surface lifted off the page.
- [`PageHead`] (`.page-head`) and [`SectionHead`] (`.section-head`): a
  page's or a section's title, its help and its actions.
- [`OverlayHost`] (`.overlay-host`): the optional root of the overlay
  layer, which [`use_overlays`] finds ([`overlay`] without one);
  [`use_overlay`] puts an app's own overlay on its stack.
- [`Popover`] (`.popover`): a surface beside its anchor in the overlay
  layer, placed before it shows ([`place`], [`place_fit`], [`use_popover`]),
  dismissed by a press outside or Escape, a sheet on a phone.
- [`Dialog`] (`.dialog`): a modal box over a scrim, its head, a body that
  scrolls and pinned actions, in three sizes; a sheet on a phone. The page
  behind holds still; Escape closes the innermost overlay only.
- [`Confirm`] (`.confirm`): a dialog asking before an action, with
  [`ask`] and [`ConfirmHost`] for promise-style confirms;
  [`ConfirmPopover`] and [`ConfirmButton`]: the same beside a button;
  [`ConfirmInline`] (`.confirm-inline`): No and Yes replacing a button in
  its box.
- [`Tooltip`] (`.tooltip`) and [`DataTips`] (`data-tip`): a short tip in
  the overlay layer after a pointer's rest or on keyboard focus;
  [`HoverCard`] (`.hover-card`): a richer card on hover or focus;
  [`ReasonButton`]: a [`Button`] that says why it cannot act, in one node.
- [`Menu`] (`.menu`): a button opening a list of actions ([`MenuItem`],
  [`MenuCheck`], [`MenuRadio`], separators, headings, notes), with
  keyboard navigation and type-ahead; [`RowActions`] (`.row-actions`), a
  row's icon buttons and overflow menu; [`SplitButton`]
  (`.split-button`); [`ContextMenu`], opened at the pointer.
- [`Select`] (`.select`): a field's box showing what is chosen, as wide
  as its widest option, opening a [`Listbox`] (`.listbox`) of its options
  while focus stays on it; [`Listbox`] is the list's keyboard state.
- [`ColorSelect`] (`.color-select`): a swatch face of one width showing the
  chosen colour, opening a grid of the app's palette (data) and an optional
  custom colour, on the select pattern.
- [`SearchSelect`] (`.search-select`): a text field filtering a
  [`Listbox`] of options in groups, loaded here or elsewhere, the list
  one width and `--picker-rows` high, loading or not.
- [`FilterPicker`] (`.filter-picker`): a list's filter button,
  `--filter-width` wide, the first choice and a reserved count of the
  rest, opening a multi-select listbox.
- [`Picker`] (`.picker`): a field of one width showing the current
  option, opening a [`ChoiceList`] (`.choice-list`), options with icon,
  mark, detail and badge slots under a search field, `--picker-rows`
  high; [`PickerMenu`], the same from an action's button;
  [`PickerDialog`], the searchable shell for a large
  set, single or multi-select, in a [`Dialog`]; [`ChoicePopup`], the
  list's popup below any opener the app renders.
- [`LibraryToolbar`] (`.library-toolbar`): a list's search field and its
  filter buttons, one row.

Where a component cannot be used (the element is another component's),
the class functions ([`stack`], [`cluster`], [`toolbar`], [`card`], ...) give
the same classes.

<br>

## Button

[`Button`] is a `<button>`, or with `href` an `<a>` styled as one. Its label
is its children; `icon` is the app's icon (`trailing` puts it after the
label); `chevron` ends it in a chevron after the label, beside a leading
icon too, as a menu's opener. sockel styles only `.sockel-button`, never a bare `<button>`.

- `variant` ([`ButtonVariant`]): `Secondary` (the plain button, the
  default), `Primary` (`sockel::class::PRIMARY`), `Ghost` (`GHOST`, no
  fill), `Quiet` (`QUIET`, no fill or border until hovered), `Danger`
  (`DANGER`), `Success` (`SUCCESS`), `Bare` (`BARE`, no fill, edge,
  padding or height of its own, for an opener whose content is its face,
  such as a swatch; the focus outline kept).
- `size` ([`ButtonSize`]): `Regular` (`--control-height`), `Small` (`SMALL`,
  `--control-height-small`), `Touch` (`TOUCH`, `--touch-height`).
- An icon without a label is the icon alone (`ICON`), square, named by
  `label` (`aria-label`). `fold` (`FOLD`) makes a button with both the icon
  alone below the phone breakpoint, its label kept for a screen reader;
  `fold_wide` (`FOLD_WIDE`) the other way round, the icon alone from the
  phone breakpoint up.
- `pressed: Some(on)` is a toggle (`aria-pressed`), marked by its fill and
  border colour. `busy` sets `aria-busy`, keeps the label in the DOM and
  lays the IconSet's spinner over it; a busy button ignores clicks.
  `disabled` disables a button; a disabled link loses its `href` and gets
  `aria-disabled="true"`.
- `submit`, `onclick`, `onkeydown`, `title`, `testid` as on [`IconButton`]; `tip` is its
  tooltip for [`DataTips`] (`data-tip`); `class` adds the app's classes. `button(variant, size, icon_only, fold)` gives the
  classes for an element that cannot be the component.
- `role` gives it another role (`menuitemradio`, `switch`), `checked` its
  `aria-checked`, and `current` its `aria-current` (`page`, `step`).

Size guarantees: the height is the size's token, whatever the label (one
line, cut with an ellipsis; the button never grows past its container) and
whatever `min-height` an app gives its own `button` elements.
The border is always there (`reserved-border`), so hover, pressed and the
variants change colours only; focus is an outline. Hover lights the border
(`hover-edge`) and, when `$button-colours` gives them, the fill
(`hover-fill`) and ink (`hover-ink`); pressed takes `pressed-fill`, `pressed-ink` and
`pressed-edge`, by default the accent's border over a tint of the fill. Pressed never changes
the font weight. Busy hides the icon and label with `visibility`, so the
box keeps their width. Fold swaps at a fixed breakpoint, and the icon's
box does not change with it.

`IconButton` stays as it was, for an app that styles its own buttons (its
children are the icon and label together); a new button should be a
`Button`.
Besides `Button`'s `submit`, `onclick`, `disabled`, `title`, `label` and
`testid`, it takes `dialog: Some(open)` (`aria-haspopup="dialog"` and
`aria-expanded`), `haspopup` (the popup alone), `pressed: Some(on)`
(`aria-pressed`), `expanded: Some(open)` with `controls` (a disclosure's
`aria-expanded` and `aria-controls`), `current` (`aria-current`) and
`node_ref`.

<br>

## Stack, Cluster and Card

[`Stack`] and [`Cluster`] (and [`Card`]) take a `tag` (`div` by default). A list
tag, `ul` or `ol`, renders with no bullets, margin or padding
(`sockel::class::PLAIN_LIST` is added for it), so it lays out exactly as a
`div` would; put `li`s in it. An app that adds `PLAIN_LIST` itself keeps
working. `stack(gap)` and `stack_with(gap, flush)` give a stack's
classes to an element that cannot be the component (a `<form>` with its
own handlers); they return the same classes for any tag, so a list built
from them adds `PLAIN_LIST` itself.

A page's column is `WORKSPACE_PAGE` or `LIBRARY_PAGE` (an index, whose
blocks lose their own margins); `LIBRARY_PAGE` + `READING` keeps its blocks
to the reading column (`--page-reading`), as a long entry's text.

<br>

## Checkbox, Switch, Choice and chips

All four mark a state by colour and background, never by weight, and
draw it in a box of a fixed size. A [`Field`] supplies a label, hint and
error; give the control `label` (an `aria-label`) when the field names it
and the control has no text of its own. Every component takes `class`.

- [`Checkbox`] (`.sockel-checkbox`): a native checkbox drawn in a
  `--check-size` box with the IconSet's `Check` (or `Minus`) over it.
  `checked`, `indeterminate` (set as the input's DOM property, drawn as a
  minus), `disabled`, `invalid` (`aria-invalid`), `on_change(bool)`,
  children as its text. `checkbox(disabled)` gives its classes.
- [`Switch`] (`.sockel-switch`): a `<button role="switch">`, its track
  `--switch-width` by `--switch-height` and its thumb moved by transform.
  `on` (`ON`), `busy` (a spinner in the thumb, clicks ignored),
  `disabled`, `invalid` (`INVALID`), `on_change(bool)`; `switch(on,
  invalid)` gives its classes.
- [`Choice`] (`.sockel-choice`): a segmented pick-one `radiogroup` of
  [`ChoiceOption`]s (`new(value, label)`, `.icon(html)`, `.icon_only(html)`,
  `.disabled(bool)`). `value`, `on_change(value)`, `label` (the group's
  name), `fit` ([`ChoiceFit`]: `Equal`, `Stretch` to fill the container
  (`STRETCH`), `Fixed` at `--choice-width` (`FIXED`)), `small` (`SMALL`),
  `invalid`, `disabled`; `choice(fit, small, invalid, disabled)`. Roving
  tabindex: the selected option, or the first enabled, is the one tab
  stop; the arrows (with wrap), Home and End select and focus the next
  enabled option.
- [`ChipToggle`] (`.sockel-chip`) and [`ChipList`] (`.sockel-chip-list`):
  filter chips. `on` (`aria-pressed`), `count` (`chip_count(n)` marks a
  zero with `ZERO`), `disabled`, `on_toggle(bool)`, children as the
  label; a list takes `label` and the chips as children.

Size guarantees: the checkbox box, the switch's track and thumb, and a
chip's height are tokens (`--check-size`, `--switch-width` and
`--switch-height`, `--control-height-small`), so no state, hover or
focus changes them (borders are reserved, focus is an outline). A
choice's options all take the width of the widest (or the container's, or
`--choice-width`), whichever is selected, and its height is the control
height; the selected one changes background and colour only, through the
`current` mixin, which now also matches `aria-checked="true"` and
`aria-pressed="true"`. A chip's count has room for two figures (tabular);
a list is at least one chip high, so an empty list does not collapse and
grow when chips arrive. New tokens: `$check-size`, `$switch-width`,
`$switch-height`, `$choice-width`.

<br>

## Tabs, SubTabs and NavList

The current tab or entry is marked by colour, background or an inset
shadow, never by weight, through the `current` mixin, so selecting one
changes no box. Every component takes `class`.

- [`Tabs`] (`.sockel-tabs`): a `tablist` of [`TabItem`]s (`new(value, label)`,
  `.icon(html)`, `.disabled(bool)`). `value`, `on_change(value)`, `label`
  (the list's name); `tabs(small)` gives its classes. Roving tabindex: the
  selected tab, or the first enabled, is the one tab stop; Left and Right
  (with wrap), Home and End select and focus the next enabled tab, from
  the focused one. Up and Down are left to the page. The selected
  tab takes the accent colour and an underline of `--tab-underline` drawn
  as an inset shadow. Icons are hidden below the phone breakpoint. Tabs
  past the width scroll sideways with the scrollbar hidden and the edge
  that has hidden tabs faded (the `scroll-fade` mixin, shared with
  [`Toolbar`]); no tab is hidden or reflowed.
- [`SubTabs`]: the same props, at the small control height (`SMALL`).
- [`NavList`] (`.sockel-nav-list`): a `nav` of a column of [`NavItem`]s
  (`new(value, label)`, `.icon(html)`, `.href(url)`, `.disabled(bool)`), a
  link with an `href` and a button without. `value` is the current entry
  (`aria-current="page"`; colour and a tint), `on_select(value)`, `label`;
  `nav_list()` gives its classes.

Size guarantees: a tab is `--tab-height` high (the control height, or the
small one) and as wide as its content whichever is selected; the underline
is a shadow, hover and focus add no border (focus is an inset outline). An
empty tab list keeps its height. A nav entry is at least the control
height with a reserved border, so the current one keeps its box. New
tokens: `--tab-height` and `--tab-underline` (custom properties on
`.sockel-tabs`); new mixin `scroll-fade`.

<br>

## Field and FormRows

[`Field`] is a label, the control (its `children`), a hint and an error,
stacked. `id` is the control's `id`, which the label's `for` names; the
control takes `aria-describedby={field_describedby(&id, has_hint,
has_error)}`, which names the hint's and the error's ids (`{id}-hint`,
`{id}-error`). sockel has no input of its own here, so the control is the
app's.

- `label`, `hint`, `error` and `help` are Html (`help` ends the label's
  row); `required` adds a `*` after the label (`aria-hidden`; the control
  sets `required` itself).
- `disabled` mutes the field (`sockel::class::DISABLED`) and `busy` sets
  `aria-busy` with a progress cursor (`BUSY`); neither changes the box.
  `class`, `testid` as elsewhere. `field(disabled, busy)` gives the classes.
- `group` makes the field a group (`role="group"`) named by its label,
  which is then text with the id `{id}-label` rather than a `<label for>`:
  for several controls, or one with no `id` (a list, an editor, a value
  shown as text). `id` is then the field's own.
- `inputs` wraps the controls: `FieldInputs::Row` (`FIELD_INPUTS`) in a
  wrapping row centred on `--control-height`, `FieldInputs::Stack`
  (`STACKED`) in a column, each as wide as the field; `Bare` (the default)
  leaves them as given. `field_inputs(inputs)` gives the classes.
- `reserve_error` (`RESERVE`, on `FIELD_ERROR`) keeps the error line's room
  while there is no error. The trade-off: the field is one line taller
  (`--field-message-line`) when it has no error, and an error longer than
  `--field-error-lines` (1) is cut with an ellipsis. Without it the error
  appears under the control and pushes what follows down.

[`FormRows`] is a grid of [`FormRow`]s (the same props as `Field`) sharing one
label column: as wide as the widest label up to half the grid, or with
`wide_labels` (`WIDE_LABELS`) `--field-label-width` (12rem) so every section's
controls start on one line. Below the phone breakpoint the label sits above
its control. [`Fieldset`] is a `<fieldset>` with a `legend` and an optional
`hint`; [`FieldCheck`] is a checkbox (its `children`) with its `label` after it,
as high as an input.

Size guarantees: with `reserve_error` the field's height does not change
when an error appears, nor does anything else (disabled, busy, focus,
required). A form row's label cell is at least `--control-height` high, so
the label is centred on a one-line control. The hint wraps (long words
break) and never widens the field.

<br>

## Pill, Badge, Dot, Count and StatusLabel

Short marks whose box is fixed, so a state, a figure or a word changing does
not move what is around them. Every component takes `class` and `testid`.
[`Tone`] (`Neutral`, `Accent`, `Success`, `Warning`, `Danger`, `Busy`, `Unset`)
is the one tone vocabulary, also [`Notice`]'s: a modifier (`ACCENT`, `SUCCESS`,
`WARNING`, `DANGER`, `BUSY`, `UNSET`; neutral has none) that only sets
`--tone`, from `--accent`, `--success`, `--warning` and `--danger`.

- [`Pill`] (`.sockel-pill`): a tinted label `--mark-height` tall, led by a dot
  of `--dot-size` in its tone. `tone`, `plain` (`PLAIN`, no dot), `on_click`
  (makes it a `<button>`, `PILL_BUTTON`; hover changes its fill only),
  `disabled`, children as the label, cut with an ellipsis in a narrow
  container. `pill(tone, plain, button)` gives its classes.
- [`Badge`] (`.sockel-badge`): a bordered short text, `--mark-height` tall and
  at least as wide. `tone`, `solid` (`SOLID`, filled, the accent by
  default), `corner` (`CORNER`, `--corner-size`, over the top end corner of
  its positioned parent by transform, so it takes no room);
  `badge(tone, solid, corner)`.
- [`Dot`] (`.sockel-dot`): a `--dot-size` circle. `tone`, `on` (`ON`; off is an
  empty ring in the same box), `label` (its accessible name; without one it
  is decoration); `dot(tone, on)`.
- [`Count`] (`.sockel-count`): a figure on a soft fill. `value`, `max` (99:
  above it the count reads `99+`), `hide_zero` (`HIDDEN`: `visibility`, so
  the box stays; a zero shown is dimmed by `ZERO`). Its `min-width` is the
  digits of `max` and one more for the `+`, in tabular figures, so the
  count holds across `0..=max+1` and a hidden one does not move its
  neighbours. The figure is `value.to_string()`; koine has no number
  formatting. `count(value, hide_zero)` and `count_text(value, max)`.
- [`StatusLabel`] (`.sockel-status-label`, `role="status"`): `states` (the
  text of every state), `current` (an index, or none), `tone`, `spinner`
  (reserves the slot), `busy` (spins in it), `pill`. Every state's text sits
  in one grid cell and the others are `HIDDEN` (and `aria-hidden`), so the
  label is as wide as its widest state and "Loading..." to "Saved" does not
  resize it. The spinner slot is a `--glyph-size` square that is there
  whether busy or not. `status_label(tone, pill)`.

Sizes: the height is `--mark-height` whatever the tone, hover or text; only
a pill's, badge's or status label's own text sets its width. A busy dot
pulses by opacity (still under `prefers-reduced-motion`).

<br>

## TextInput, Textarea, SearchField and InputGroup

Text controls at a control's height. All take `class`, `size`
([`InputSize`]: `Regular` is `--control-height`, `Small` (`SMALL`) is
`--control-height-small`, `Touch` (`TOUCH`) is `--touch-height`; the height
never changes with the viewport), `invalid` (`INVALID` and `aria-invalid`),
`disabled`, `busy` (`aria-busy`, progress cursor) and `testid`. Inside a
[`Field`], pass the field's `id` and `describedby={field_describedby(..)}`.

- [`TextInput`] (`.sockel-input`): `value` and `on_input` (the new text), `on_change`,
  `on_blur`, `on_keydown`, `kind` ([`InputKind`]: text, email, url, tel),
  `maxlength`, `list` (a `<datalist>`'s id), `inputmode` and `enterkeyhint`
  (a touch keyboard's kind and enter key),
  `placeholder`, `label` (the accessible name without a `Field`), `readonly`,
  `required`, `autofocus`, `node_ref`. With a `unit` (Html, `aria-hidden`)
  the input sits in a box (`INPUT_BOX`) with the unit after the text inside
  the field; the box takes the border and the focus ring, so the unit is
  part of the field's box and the text keeps its start.
- `native_select(size, invalid, fill)` (`.sockel-input.sockel-native-select`):
  a native `<select>` in a text field's look, the browser's arrow replaced
  by a chevron in `--muted` and its options on `--surface`. It is as wide as
  its widest option, cut at its container, and in a row it shrinks to give
  way to the items beside it; `fill` makes it its container's
  width. Set `aria-invalid` with `invalid`, as on a bare input.
- [`Textarea`] (`.sockel-textarea`): `rows` (default 3), at least `--input-height`
  high, resized vertically by the user only.
- [`SearchField`] (`.sockel-input-box`): a search icon, the text and a clear
  button. `label` is the placeholder and the accessible name, `clear_label`
  names the button, `busy` swaps the icon for a spinner in the same box.
  Clearing (the button or escape) calls `on_input("")` and returns the focus
  to the text. `input_ref` is the text input's ref; `on_keydown` sees each
  key first (preventing escape's default keeps the text); `escape_clears`
  (default `true`) set to `false` hands escape to `on_keydown` alone, its
  default prevented before `on_keydown` runs (a component's message is
  handled after the field has read the key, too late to prevent it), for a
  combobox that closes its list with escape. `controls` (a
  listbox's id) makes it that list's combobox (`role="combobox"`,
  `aria-controls`, autocomplete off), `activedescendant` naming its active
  option, and `expanded` (unset is shown) its `aria-expanded`. `placeholder`
  replaces the label as the placeholder, `title` is the text's tooltip,
  `keyshortcuts` its `aria-keyshortcuts`, and `on_focus`/`on_blur` see the
  text's focus. [`LibraryToolbar`] is a `SearchField` (class `LIBRARY_SEARCH`)
  and its filters, passing `input_ref` and `on_keydown` on; its `icon` prop is gone, the icon is the [`IconSet`]'s
  search glyph.
- [`InputGroup`] (`.sockel-input-group`): controls joined in a row. Give each
  member `group_classes(&detached)[i]` as its `class`: `GROUP_FIRST`,
  `GROUP_MIDDLE` or `GROUP_LAST` square the corners that join (a lone member
  takes none), and `detached[i]` starts a new run after a `--space-2` gap
  (`GROUP_DETACHED`). The members overlap by their one-pixel border, the
  hovered or focused one on top. No rule uses `:has()`, a sibling selector
  or `!important`. A `TextInput` in a group takes the room the buttons leave.
- Bare helpers: `input(size, invalid, busy)`, `input_box(size, invalid,
  disabled, busy)`, `textarea(size, invalid, busy)`.

Size guarantees: a field's box is the same empty or filled, long text or
short (the text is cut with an ellipsis, never wrapped or widened); hover,
focus and invalid change colours only (the border is reserved, the focus ring
is an outline); the SearchField's icon and clear slot are always laid out, the
button only hidden while empty, so typing moves nothing; a group's members of
one size are one height. New tokens: none. New partial: `_inputs.scss`.

<br>

## Stepper and SecretInput

- [`Stepper`] (`.sockel-stepper`, worn with `.sockel-input-box`): a number
  between a minus and a plus button. `value`, `on_change(i64)`, `min`, `max`,
  `step` (default 1), `label` (the number's accessible name), `less_label`
  and `more_label` (overrides of the catalog's words), `id`, `size` ([`InputSize`]),
  `invalid`, `disabled`, `busy`, `live` (report each number typed within the
  bounds at once, not only when committed; off by default), `class`,
  `testid`. The buttons step onto
  multiples of `step` (7 by 5 goes to 10 or 5), from the number typed if it
  is not yet committed, and stop at the bounds; the
  arrow keys do the same, Page Up and Down ten steps, Home and End go to the
  bounds, Enter or leaving the field commits what is typed (clamped; text that
  is no number goes back). What is typed outside `min..=max` is marked invalid
  until committed. The buttons are out of the tab order. Helpers: `stepper(size,
  invalid, disabled, busy)`, `step_from`, `key_value`, `parse_number`.
- [`SecretInput`] (`.sockel-input-box`): a password field with show or hide,
  copy and clear slots (`.sockel-input-action`, the `INPUT_CLEAR` button's
  look). `value`, `on_input`, `label`, `on_copy` (also writes the clipboard),
  `show_label`, `hide_label`, `copy_label`,
  `clear_label` (each an override of the catalog's word), `commit` (`on_input`
  only for a finished edit, on leaving the field or Enter, and on clear: for a
  value saved as it changes; the field and the slots follow what is typed
  meanwhile), `titles` (each slot's name as its tooltip too),
  `placeholder`, `reveal_ms` ([`REVEAL_MS`], 3000: a
  shown secret hides itself, showing again restarts the time; clearing
  hides it), `id`, `size`, `invalid`, `disabled`, `busy`, `autofocus`,
  `class`, `testid`. The strings are props; an app passes its own words.

Size guarantees: the Stepper's number has the room of the longest of `min`
and `max` in `ch` with `numeric` figures, so its box is the same from the
least value to the most (and its own width in a flex column or a grid cell,
not stretched to it), the buttons only disabled at a bound; the number wears `.sockel-zero`
(`ZERO`) at 0, and the settings `$stepper-number-weight`,
`$stepper-zero-ink` and `$stepper-zero-weight` (all `null`: the field's
text) restyle the number and the zero, and `$stepper-touch-number-size`,
`$stepper-touch-number-weight` and `$stepper-touch-glyph-size` (`null`: as the
other sizes') the touch size's number and glyphs, and `$stepper-edge`,
`$stepper-ground` and `$stepper-shadow` (`null`: the field's edge and
ground, no shadow) the box at rest, for a stepper that reads as a raised
control; the
SecretInput's box is the container's width, dots or text, and its three slots
are always laid out (hidden, not removed, while empty). New classes:
`STEPPER`, `STEPPER_CONTROL`, `STEPPER_BUTTON`, `INPUT_ACTION`. New glyphs
`Eye`, `EyeOff` and `Copy`. New partial: `_stepper.scss`. New tokens: none (`--stepper-chars` is
set per Stepper).

<br>

## Heads

[`PageHead`] and [`SectionHead`] take one set of props ([`HeadProps`]):

- `title` is Html; `icon` goes before its text in the heading, `subtitle` is a
  muted line under the row, `crumbs` (a list of links) stack above it, and
  `back` is the inline "back / title" crumb. Each is its own prop.
- `help` follows the heading as its sibling, in `.sockel-head-heading` with
  the `suffix`, so the heading's text and test id exclude the help button.
  The group stays on one line: a long title wraps inside its own box, with
  the help beside it.
- `SectionHead` takes the modifiers `sockel::class::LABEL` (small-caps label),
  `LIST` and `LIST` + `NESTED` (smaller titles) in `class`; `HEAD_END` pushes a
  child to the row's end.
- A head is at least `--control-height` high, so actions coming and going
  never move what follows; a `LABEL` head holds none and is its line's
  height. Neither has outer margins, also when its class is on a heading
  element (`<h3 class="sockel-section-head sockel-label">`).
- `PageHead` takes `COMPACT` (actions as icons, one row on a phone) and
  `ONE_ROW` (a crumb head that stays one row on a phone, the back link's
  words, `PAGE_HEAD_BACK_LABEL`, hidden) in `class`.

<br>

## Icons

sockel's own chrome (a dialog's close, a select's chevron, a busy spinner)
draws its icons from an [`IconSet`], and the app keeps its own icons in the same
set. The set is opaque, built with a builder, so adding a glyph to sockel never
breaks an adopter. [`Glyph`] (`Close`, `ChevronDown`, `ChevronUp`, `ChevronRight`,
`Check`, `Plus`, `Minus`, [`Spinner`], `Grip`, `Ellipsis`, `Search`, `Eye`,
`EyeOff`, `Copy`) is `#[non_exhaustive]`: a `match` on it needs a wildcard. It is
the one place sockel uses a context for something other than a runtime setting,
and it carries icons only: classes never go through it.

`IconSet::default()` holds nothing of its own: a glyph the app did not set is
the built-in inline svg, built when it is first drawn, so an app that
overrides a glyph does not build the built-in one. Without a provider sockel
uses the default set, so it renders standalone. An app supplies its set once at
its root:

```rust
let set = IconSet::default()
    .with(Glyph::Close, html! { <MyIcon name="x" /> })
    .with_named("trash", html! { <MyIcon name="bin" /> });

html! {
    <ContextProvider<IconSet> context={set}>
        <App/>
    </ContextProvider<IconSet>>
}
```

`<Icon glyph={Glyph::Close} class=.../>` renders a glyph and `<Icon
name="trash"/>` an app icon; `use_icon(glyph)`, `use_icon_named(name)` and
`use_icons()` (`get`, `get_named`) return icons for a component of your own,
and `icon(class, glyph)` gives the box's classes (`sockel::class::GLYPH`, plus
`SPIN` for `Spinner`). A name the set lacks renders the same empty box, and a
debug build warns in the console with the name. Cloning a set shares its icons.

Size guarantees: the glyph sits in a `.sockel-glyph` box of `--glyph-size`
(`1em`, so it follows the text) square, `overflow: hidden`, and what fills it
is stretched to it. A swapped set, whatever its icons' own width and height,
cannot change a control's size. Icons should draw in `currentColor`. The
spinner's box turns (slower under `prefers-reduced-motion`); the set's
spinner need not animate itself.

<br>

## Notice, ErrorBox and EmptyState

[`Notice`] is an inline message. `tone` ([`Tone`], the accent by default;
see above), coloured from `--accent`, `--success`, `--warning` or
`--danger`; `Neutral` is muted. `icon` is the app's icon, before the text; `action` is Html
(a link or button) after it; `onclose: Some(callback)` shows a close button
(`close_label`, an override of the catalog's word, names it). The notice does not remove itself: the
app stops rendering it, and the space it held collapses at once, which is
intended. A `Warning` or `Danger` notice has `role="alert"`, the others
`role="status"`. `notice(tone)` gives the classes.

[`ErrorBox`] is an error, in `--tone: var(--danger)`: `title`, the message (its children), `cause` (the
technical text, cut to three lines, whole in its `title` attribute) or
`causes` (a chain of them, most specific last: `ERROR_BOX_CAUSES`, a list
of a line each, never cut) and `onretry` (a small button, `retry_label`; `retrying` makes it
busy and ignore clicks). `icon` (the app's, before the text, in the tone) and
`ondismiss: Some(callback)` (a quiet close button after it, named and
tooltipped by `dismiss_label`; the box does not remove itself)
flank the text (`FLANKED`, the text in `ERROR_BOX_BODY`). `error_box(flanked)`
gives the classes. Its look is configured by `$error-box-padding`, `-gap`,
`-fill`, `-edge`, `-shadow`, `-message-weight`, `-cause-font`,
`-cause-text` and `-text-end` (the space under a flanked box's text)
(README, "Configuration").

[`EmptyState`] is what stands where there is nothing: `title`, `icon`, `hint`
and actions (its children). `compact` (`COMPACT`) is one row for a list or a
table. `empty_state(compact)` gives the classes.

Every one takes `class` and `testid`. Parts: `NOTICE_ICON`, `NOTICE_BODY`,
`NOTICE_ACTION`, `NOTICE_CLOSE`, `ERROR_BOX_TITLE`, `ERROR_BOX_MESSAGE`,
`ERROR_BOX_CAUSE`, `ERROR_BOX_ACTIONS`, `ERROR_BOX_ICON`, `ERROR_BOX_BODY`,
`ERROR_BOX_CLOSE`, `EMPTY_STATE_ICON`,
`EMPTY_STATE_TITLE`, `EMPTY_STATE_HINT`, `EMPTY_STATE_ACTIONS`.

Size guarantees: a notice is at least one small control plus its padding
high, its text centred in that, so one line is as high with a close button
and an action as without; its text wraps, it never widens. An error box
keeps its retry button's box while retrying. A compact empty state is
exactly `--control-height` high whatever it says (title and hint cut to one
line, its actions kept at the end), so it replaces a row of that height
without moving what follows; the regular one grows with its text.

<br>

## Toast, ToastHost and SaveBar

[`ToastHost`] is where toasts stand: fixed over the window's bottom
(`--layer-toast`), centred, newest last, in an always-present polite live
region. It renders in the overlay layer (see [`OverlayHost`]), wherever the
app places it, so an open dialog never puts it out of reach. Fixed, it takes no room: a toast coming or going moves nothing.
`raised` lifts it by `--toast-tray`, the height of a tray or bar the app
keeps at the bottom (0 unless the app sets it on `:root`). `toast_host(raised)`
gives the classes.

[`Toast`] is one message: `tone` ([`Tone`], the accent by default), `icon`, the
text (its children), `action` (a label such as "Undo") with `onaction`,
`action_icon` and `action_testid` (both none by default),
`ondismiss` (closed or timed out), `duration` (milliseconds, [`TOAST_MS`] =
8000; 0 keeps it until dismissed), `shortcut` (Ctrl or Cmd+Z takes the action
while it shows, unless typing in a field) and `close_label`. The clock stops
while the pointer is over the toast or focus is in it and starts afresh
after; the action and the dismissal happen once between them. Give each
toast its own `key`. `toast(tone)` gives the classes. The app owns the list:
it stops rendering a toast when `ondismiss` or `onaction` is called.

[`SaveBar`] is a form's bar of unsaved changes: `dirty`, `busy` (the save
button's spinner, clicks ignored), `disabled`, `label` (an override of the catalog's words),
`status` (how the last save went), `onsave`, `ondiscard` (shows Discard),
`save_label`, `discard_label`, `submit` (Save submits the form it stands in),
`sticky`, and children (further actions). While `dirty` it holds an
[`UnloadGuard`], the browser's own prompt on closing or reloading the tab
(`guard=false` leaves that to the app, which can call `use_unload_guard(active)`
itself; `unsaved()` says whether any guard is held). `save_bar(idle, sticky)`
gives the classes.

Every one takes `class` and `testid`. Parts: `TOAST_ICON`, `TOAST_BODY`,
`TOAST_ACTION`, `TOAST_CLOSE`, `SAVE_BAR_NOTE`, `SAVE_BAR_STATUS`,
`SAVE_BAR_ACTIONS`. Modifiers: `RAISED`, `IDLE`, `STICKY`, the tones. Token:
`--toast-max-width` (28rem); `--toast-tray` is the app's to set.

Size guarantees: a toast is at least one small control plus its padding
high, its text wrapping within `--toast-max-width` and the window. The
`SaveBar` choice is to keep its room: it is always in the flow at one control
plus padding high, hidden (`visibility`) while clean and not removed, so the
form's first edit moves nothing; `sticky` keeps it at the bottom of the scroll
container, still in the flow. Its save button is busy in its own box.

<br>

## Spinner, Skeleton and Loadable

[`Spinner`] is a spinning glyph (`Glyph::Spinner` from the icon set) in the
icon box: inline, `1em`, in the colour of the text around it. `label` makes
it a `role="status"` with that name; without one it is decoration. [`Button`]'s
busy mark is a `Spinner`.

[`Skeleton`] is a placeholder of the box its content will have. `shape`
([`SkeletonShape`]): `Line` (`LINE`, one line of the text around it, `1lh`),
`Block` (the default, `--control-height`, or `--skeleton-height`), `Row`
(`ROW`, one [`RowListItem`] high) or `Image(Ratio)` (an [`Image`]'s ratio
classes, `POSTER` and the rest). `count` stacks several in a column
(`SKELETON_GROUP`); the last of several lines is `SHORT`. A group reserves
the 1px border a [`RowList`] does, so `count` skeleton rows are exactly as high
as `count` one-line rows. It is `aria-hidden` and shimmers, still under
`prefers-reduced-motion` (`--skeleton-duration`). `width` sets its width (each
one's, with `count`), such as the length of the text it stands for.
`skeleton(shape, short)` gives the classes.

[`Loadable`] is one box for content that loads. `state` ([`LoadState`]):
`Loading` shows the placeholder (`placeholder`, or `rows` skeleton rows),
`Ready` the children, `Empty` an [`EmptyState`] (`empty` replaces it,
`empty_title` names it) and `Error` an [`ErrorBox`] (`error`, `error_cause`,
`onretry`, `retrying`). The empty state waits `empty_delay` (250 ms) before
it shows, the placeholder standing in, so it never flashes before data.
While it loads it is `aria-busy` and a visually hidden `role="status"` reads
`label` (an override of the catalog's word). Every one takes `class` and `testid`.

Size guarantees: while loading, a `Loadable` is at least `rows` rows high the
first time and at least the height it last had settled (ready, empty or
error) afterwards, so loading to ready and ready to loading again move
nothing when the placeholder has the content's rows. Once settled it is as
high as its content, so a shorter list shrinks. An empty state or an error
taller than the box grows it; use `EmptyState` `compact` to stay inside.

<br>

## Image and Avatar

[`Image`] is a picture in a box whose ratio is fixed before it loads, so the
page does not shift as pictures arrive. Props:

- `ratio` (required; there is no default, so no image lacks a box):
  `Ratio::Poster` (2/3), `Screenshot` (16/9), `Banner` (21/4), `Square`, or
  `Explicit(width, height)`, which also sets the `<img>`'s `width` and
  `height` attributes.
- `src` (`Option`), `alt`, `lazy` (default on: `loading="lazy"`), `class`,
  `onload`, `onerror` and `testid`.

Its modifiers are the ratio's classes (`POSTER`, `SCREENSHOT`, `BANNER`,
`SQUARE`, `RATIO`) and its status's: `LOADING`, `ERROR`, `EMPTY` (no `src`) and
`LOADED`. The status follows the `<img>`'s `load` and `error` events and starts
over when `src` changes.

Size guarantees: the box is `aspect-ratio` at the width it is given (the
container's, unless `class` sets one), and the picture is laid over it with
`object-fit: cover`, so the loading, error and empty placeholders (a glyph
from the icon set, centred) and the loaded picture all fill the same box.
A picture of another shape is cropped, never stretched; the box never
resizes.

[`Avatar`] is a round portrait of a fixed size: `name` (its initials, the first
letters of its first and last word, show behind the picture and are its
accessible name), `src`, `size` (`AvatarSize::Small`, `Medium` by default,
`Large`, `Huge`: the `--avatar-sm`, `-md`, `-lg` and `-xl` tokens), `lazy`,
`class` and `testid`. The box is `--avatar-*` square whether the picture is
loading, failed, missing or loaded. `initials(name)` is the same function for
a place of your own.

<br>

## OverlayHost

Popovers, menus, selects, tooltips, dialogs and toasts render in the overlay
layer, a portal out of whatever scrolls or clips their owner. Its events
still bubble through the owner. [`OverlayHost`] is optional and gives the app
its own layer:

```rust
html! {
    <div class="my-app" data-theme="dark">
        <OverlayHost>
            <App/>
        </OverlayHost>
    </div>
}
```

- Overlays render in a `.sockel-overlay-host` element after its children,
  so they inherit what the element around it sets (colour variables, a
  theme scoped to a sub-element). It adds no box of its own. Place it where
  no ancestor has `transform`, `filter`, `contain` or `will-change`, or
  fixed overlays are placed against that ancestor.
- It keeps the app's stack of open overlays, so Escape and a press outside
  close the innermost first. Two hosts in two separate roots keep separate
  stacks. A host inside another gets its own layer, for a sub-tree with
  its own colours, and shares the outer one's stack. The colours derived
  from the app's (`--surface-raised`, `--on-accent`, `--scrim`, the status
  hues) resolve on `:root`, so a scope that sets its own colours sets
  those there too, as `--surface-raised: var(--surface)`.
- Props: `class` (on the host element) and `testid`. `overlay_host(class)`
  gives the classes.

Without one, nothing changes from before: overlays go to `#sockel-overlay`
(`sockel::OVERLAY_ID`), made on first use as the last child of `<body>`,
with one stack per thread. Because it is a body child, it does not inherit
colours an app sets below `:root`. On the server there is no layer, and
overlays render in place.

For a component of your own, `use_overlays()` returns the nearest host's
[`Overlays`] (or the fallback), and `.portal(html)` renders into it.
`use_popover` already takes its place on the stack from it. An overlay the
app draws itself (a full-screen panel, a drawer) joins the stack with
`use_overlay(open, on_escape)`: while it is open, Escape calls `on_escape`
only when nothing opened after it is still open, so a menu or dialog opened
from it closes first, and not when an element in it acted on Escape itself
(by preventing the key's default). `Overlays::any_open()` says whether
anything but a tip is open, for an app's own keyboard shortcuts to stand
aside meanwhile. The free
`overlay(html)` always uses the fallback layer, since a plain function
cannot see a context. Prefer `use_overlays().portal(html)` in components.

Every stacked overlay (popover, menu, list, hover card, dialog) is at
`--layer-overlay`, and the one opened later is later in the host, so on top.
Toasts and tooltips (`--layer-toast`, `--layer-tooltip`) stay above. Within
this, ordering is by opening, not kind. An open dialog puts the page and the overlays opened before it out of
reach (`inert`), but never the toast region: a toast's Undo over a dialog
still works, and it is still announced.

<br>

## Popover

[`Popover`] is a surface beside its anchor, rendered while `open` in the
overlay layer (see [`OverlayHost`]), so no scrolling or clipping box around
the owner cuts it, and its events still bubble through the owner. Props:

- `anchor` (a `NodeRef` on the element it opens beside, where focus returns),
  `open`, and `on_dismiss: Callback<Dismiss>`: the owner closes it.
  `anchor_element` (a `web_sys::Element`) stands in for `anchor` when the
  owner did not render what it opens beside, such as the button just
  pressed or the button of the menu an item was picked in (a `NodeRef`
  cannot be pointed at an element from outside Yew).
- `side` (`Side::Below` by default, `Above`, `Start`, `End`) and `align`
  (`PopoverAlign::Start`, `Center`, `End`).
- `fit`: what a popover too tall for the room both above and below its
  anchor does. `Fit::Scroll` (the default) takes the roomier side and
  scrolls there; `Fit::Slide` stays on its side and slides along it, over
  the anchor, to fit the viewport (less its edges), scrolling only when
  taller than that: a long menu at phone height shown whole.
- `sheet` (on by default, `sockel::class::SHEET`): below `$sheet-breakpoint`
  (the phone's) it is a sheet from the bottom over a `--scrim`, with
  `title` as its heading and a close button (`close_label`, an override of the catalog's word), unless
  `head` is off (on by default).
- `title` is also its accessible name (`role="dialog"`); `class`, `testid`.
- `focus` (a `NodeRef`): the element that takes focus once it is placed.
- `focus_first`: without `focus`, focus moves once it is placed to an
  `[autofocus]` or `[data-autofocus]` element, else its body's first
  control, else the surface (then `tabindex="-1"`), as in a [`Dialog`]: for a
  popover of fields or choices opened from the keyboard.
- `within` (a selector): placed and kept inside the anchor's closest
  ancestor matching it, cut to the viewport, rather than the viewport
  alone: a popover raised in one of an app's docked windows stays over
  that window, not its neighbour. Without a match, the viewport.
- `trap`: Tab stays inside it, from its last control to its first, as in a
  `Dialog`; for a popover that asks for an answer ([`ConfirmPopover`] sets it).
- `flush`: a body without padding (`sockel::class::FLUSH` on
  `.sockel-popover-body`), for content that lays out its own edges, such as
  a list that runs to the surface's; off by default.
- `match_anchor`: at least as wide as the anchor (`sockel::class::MATCH_ANCHOR`,
  the anchor's width in `--popover-anchor-width`), as a field's list under
  it; off by default. A sheet stays the screen's width.
- `keep_focus`: a press inside it that is not on a control (a field, button,
  link, select or anything with a `tabindex`) keeps focus where it is, such
  as in a search field whose results are clicked; off by default.
- `on_pointer_enter` and `on_pointer_leave` (`Option<Callback<PointerEvent>>`):
  the pointer over the surface, for an owner that keeps it open while the
  pointer is there (it is in the overlay layer, so the owner's own
  `pointerleave` fires as the pointer moves onto it).

Dismissal ([`Dismiss`]), only while no overlay opened after it is open:
`Outside`, a `pointerdown` outside the anchor and the popover, keeps focus
where the press put it; `Escape` (unless what has the focus took it,
preventing its default, as a search field clearing its text does) and
`Close` (the sheet's button) return focus to the anchor, as does any close by its owner
that leaves focus nowhere after focus or a press was inside the popover (one
the pointer only passed over, a hover card, leaves focus alone). Focus goes
back to what had it inside the anchor when it opened, such as the button of
a row the popover lines up with, else to the anchor itself.

`use_popover(open, anchor, popover, side, align, on_dismiss)` is the hook
under it, for a component that renders its own surface (a menu, a select):
it returns `Option<Placed>` and does the measuring, following and
dismissing. `place(anchor, (width, height), viewport, side, align)` is its
pure placement: the given side unless the popover does not fit and the
opposite side has more room, then moved back inside the viewport, its
height capped (`max_height`) by the room on the side it took.
`place_fit(.., fit)` is the same with a [`Fit`] (`place` is `Fit::Scroll`).
`use_popover_within(.., within, on_dismiss)` takes `within` too, and
`placement_bounds(anchor, within)` is the box either places inside, for a
surface placed by its owner with `place` (`Rect::clip` cuts one box to
another). `popover(sheet, placed)` gives the classes (`POPOVER`, `SHEET`, `PLACED`).

Size guarantees: it is hidden (`visibility`) until it has been measured and
placed (`PLACED`), so it never shows at 0,0 first. It is `position: fixed`
in a layer that lays nothing out, and nothing locks the page's scroll, so
opening it never moves the page, its anchor or the scrollbar. Its width is
its content's up to `--popover-max-width`, whichever side it lands on, so a
flip moves it without reflowing it; what is taller than the room scrolls in
`.sockel-popover-body`, its scrollbar's room kept. It follows the anchor on
scroll, resize and a change of either's size; once open above or below, it
keeps that side while the side has 200px or all it needs, so content that
grows (a list appearing in a menu) scrolls there instead of flipping it
across the anchor. It closes (as a press
outside, focus left where it is) once the anchor is out of sight: no
longer laid out, or scrolled or clipped wholly out of a box around it
whose overflow clips. Menus, selects, pickers and tips share this placement.

<br>

## RowList, PickRow, SelectionBar, ActionRow and CardGrid

[`RowList`] is a column of rows (`tag`, `ul` by default, with no bullets, margin
or padding). `divided` (`DIVIDED`) draws a line above every row but the first;
`boxed` (`BOXED`) gives it a bordered, rounded surface. [`RowListItem`] is a
row: `name` (one line, cut), `detail` (under it, muted, cut), and its
children, which end the row (an [`ActionRow`], a [`Checkbox`]). `top` (`TOP`) is a
tall row: parts to the top, the name wraps. `row_list(divided, boxed)` and
`row_list_item(top)` give the classes. Parts: `ROW_LIST_MAIN`,
`ROW_LIST_NAME`, `ROW_LIST_DETAIL`.

[`PickRow`] is a button that is a whole row choosing one thing: `name`,
`detail`, `mark` (before the name), children (after), `current`
(`aria-current`, marked by background only), `disabled`, `onclick`.
Parts: `PICK_ROW_MARK`, `PICK_ROW_NAME`, `PICK_ROW_DETAIL`.

[`SelectionBar`] is the band under a list: `count`, `label` (the count as text,
in a `role="status"`) and the actions as children. It is always in the flow
at one height: with `count == 0` it is `IDLE` (`visibility: hidden`), so
picking never moves the list. `inset` (`INSET`) levels its sides with the
rows; `sticky` (`STICKY`) keeps it at the bottom of its scroll container.
Part: `SELECTION_BAR_COUNT`. `selection_bar(idle, inset, sticky)`.

`ActionRow` is actions that wrap; `nowrap` keeps one line and never shrinks.
`overlay` (`OVERLAY`) lies over the end of its row, taking no room, with
`opacity: 0` and `pointer-events: none` until the row (a `RowListItem`) is
hovered or has focus. Its controls stay in the tab order, so focus reveals
it. It reads `--overlay-opacity`, `--overlay-events` and `--overlay-tint`,
which a row sets; its corners and side padding are `--action-overlay-radius`
and `--action-overlay-padding`. `action_row(overlay, nowrap)`.

[`CardGrid`] lays cards in as many columns as fit (`tag`, `ul` by default).
`size` ([`GridSize`]): `Normal` (columns of at least `--card-grid-min`), `Wide`
(`--card-grid-wide`) and `Narrow` (`--card-grid-narrow`, and
`--card-grid-narrow-phone` with a closer gap on a phone). `card_grid(size)`.

Every one takes `class` and `testid` (the parts' children aside).

Size guarantees: a row is at least `--control-height` high, as a compact
[`EmptyState`] is, so one replaces the other without moving what follows. Every
row reserves its top border, so `divided`, hover, focus and a chosen
`PickRow` change no box. An overlay `ActionRow` takes no room, so revealing
it moves nothing. A selection bar keeps its height whether anything is
picked or not. A card grid's cards are as tall as their row.

<br>

## Reorder and DragHandle

[`Reorder`] is a list whose rows move by a drag on their [`DragHandle`] or by
keyboard. `rows` is a `Vec<ReorderRow>` (`ReorderRow::new(key, name)`,
`.detail(html)`, `.end(html)`; the key is stable across moves), and
`on_move((from, to))` reports a move, `to` an index in the list without the
row: apply it with `move_item(&mut list, from, to)`. `divided`, `boxed`
(as [`RowList`]'s), `disabled`, `class`, `testid`. The rows are
[`RowListItem`]s (`.sockel-reorder-row`), the handle in their `start` slot
(a new `RowListItem` prop: what begins a row).

Children mode, for rows that are the app's own (a row of fields, an
editor): in place of `rows`, give [`ReorderItem`] children (`id`, stable
across moves; `name`, for the announcements and the handle's name;
`class`; `testid`), each holding its content and a [`ReorderHandle`]
(`class`, `testid`) wherever the row wants its grip. Each item is the row
element itself (`.sockel-reorder-row` plus its `class`), a direct child of
the `.sockel-reorder` element, which `class` lays out; the hint and the
status follow the rows there. A `ReorderHandle` outside a `Reorder`
renders nothing.

- `quick_keys`: the arrows move a focused handle's row at once, as Alt and
  an arrow do, for apps whose handles did so; Space and Enter still lift.
- `titles`: each handle's name is its tooltip too (`DragHandle`'s new
  `title`).

- Pointer: press a handle and move past 4px; the row lifts (`LIFTED`) and
  follows the pointer, kept within the list. It passes a row once its top
  (going up) or its bottom (going down) crosses that row's middle, so it
  reaches either end whatever the rows' heights (`drag_target(middles,
  from, top, bottom)`). Release drops it, Escape or a cancelled pointer
  puts it back.
- Keyboard, on a focused handle: Space or Enter lifts, the arrows, Home and
  End aim, Space or Enter drops, Escape or leaving the handle cancels.
  Alt and an arrow moves the row at once. Focus follows the moved row.
- The row it lands beside is marked `DROP_BEFORE` or `DROP_AFTER`.
- `text: Callback<ReorderText, String>` gives the handle's name
  (`Handle`), its hint (`Hint`, its `aria-describedby`) and the
  announcements (`Lifted`, `Moving`, `Dropped`, `Cancelled`, positions from
  1), read from a `role="status"` region. Without it they are the
  catalog's `sockel-reorder-*` messages ("Moved Alpha to 3 of 5");
  `reorder_text` gives their English.
- `DragHandle` is the grip alone (the [`IconSet`]'s `Grip`): `label`, `lifted`,
  `disabled`, `describedby`, the pointer, key and blur callbacks, `class`,
  `testid`, `title`. Helpers: `reorder_row(index, lift)`,
  `drag_handle(lifted)`, `drag_target(middles, from, top, bottom)`,
  `landing(from, slot)`, `slot_at(middles, y)`, `key_target(key, to, count)`.

Size guarantees: nothing reflows during a drag. The rows are measured once
at the press; the lifted row moves by a transform (`--reorder-shift`) and
keeps its own room, so no gap of another height opens and the list's
height holds; the drop marks are inset box shadows, never borders or
margins; the order changes once, on the drop. A keyboard lift moves only
the marks. The handle is `--control-height-small` square with a reserved
border; hover, focus and lifted change colours only. Settings:
`$drag-handle-size`, `$drag-handle-phone-size` (`null`: as above the
phone breakpoint), `$drag-handle-radius`, `$drag-handle-hover-fill`. New
classes:
`REORDER`, `REORDER_ROW`, `LIFTED`, `DROP_BEFORE`, `DROP_AFTER`,
`DRAG_HANDLE`. New partial: `_reorder.scss`. New tokens: none.

<br>

## Dialog

[`Dialog`] is a modal box over a `--scrim`, rendered while `open` in the
overlay layer (a portal, like [`Popover`], not a native `<dialog>`: a modal
`<dialog>` makes everything outside the top layer inert, the overlay layer
included, so a popover or menu opened inside it could not be used). Props:

- `open`, `on_dismiss: Callback<Dismiss>` (the owner closes it), `title`
  (its heading and accessible name), `icon` (before the title).
- `size`: `DialogSize::Default` (`--dialog-width`), `Wide`
  (`sockel::class::WIDE`, `--dialog-width-wide`), `Tall` (`TALL`, fixed at
  `--dialog-height-tall`) or `WideTall` (both).
- `flush`: a body without padding that does not scroll (`FLUSH` on
  `.sockel-dialog-body`), for content that lays itself out and scrolls its
  own parts, such as a manual with its contents beside it.
- `dismiss`: `DialogDismiss::Any` (Escape, the close button, a press on the
  scrim), `NoOutside` (not the scrim) or `Actions` (no close button, Escape
  does nothing). `DialogDismiss::allows(Dismiss)` tells which.
- `head_actions`: tools in the head, before the close button.
- `actions` (the foot's end, the primary last) and `actions_start` (apart,
  at its start: a Delete), each an [`ActionGroup`] in `.sockel-dialog-foot`.
- `sheet` (on by default, `SHEET`): below `$sheet-breakpoint` a sheet from
  the bottom, up to `--dialog-sheet-height`. `close_label`,
  `close_size` (`ButtonSize::Small` by default) and `close_testid` (the
  close button's), `class`, `testid`.

`dialog(size, sheet)` gives the classes. Parts: `.sockel-dialog-layer` (the
scrim), `-head` (a [`SectionHead`]), `-body` (what scrolls), `-foot`.

Behaviour: opening it moves focus to an `[autofocus]` or `[data-autofocus]`
element, else the body's first control, else the box; Tab and Shift+Tab
wrap inside it; the page and the overlays open before it are `inert`. A
dialog opened from a dialog stacks over it, the one beneath dimmed by its
scrim and out of reach. Escape and a press outside close only the innermost
overlay, a popover included: overlays are kept on one stack. An Escape
whose default was prevented (a [`SearchField`] clearing its text) or that
comes from outside the dialog's own DOM (an app's own popup rendered
elsewhere, which closes alone) leaves it open. While it is the innermost
overlay, focus that falls to the page (its control removed or disabled) comes
back to the last control in it that had it, else to the box. On close focus returns to
what had it.

Size guarantees: while a dialog is open the root takes
`.sockel-scroll-locked` (no scrolling), and `.sockel-gutter-kept` when the
page had a scrollbar, so the scrollbar's room stays and the page neither
moves nor scrolls; an app whose root already has `scrollbar-gutter: stable`
keeps it either way. The kept room's width is `--scroll-gutter` on the root
while a dialog is open, and the scrim reaches over it, so it covers the
whole window. The box hangs `--dialog-top` from the window's top,
not centred unless `$dialog-centred` says so, so content that loads or grows
moves only its bottom edge;
it is at least `--dialog-min-height` and at most the window's room, beyond
which the body scrolls with its scrollbar's room kept (unless
`$dialog-scroll-gutter` says otherwise) and the foot stays
pinned below it. Its width is fixed by its size, never by its content. A
`Tall` dialog keeps one height whatever it holds; a sheet grows upward
from the bottom edge.

<br>

## Confirm, ConfirmPopover, ConfirmButton and ConfirmInline

Asking before an action, in three places. [`Confirm`] is a [`Dialog`] (rendered
while `open`): `title` (its heading and name), `explanation`, `details` (a
list of what it affects), `accept` (the accepting button's label) with `icon`,
`cancel_label` (the catalog's "Cancel" unless given), `danger` (on by default: the accepting button is
`Danger` and Cancel has the focus, so Enter never destroys by habit; off, it
is `Primary` and has the focus), `busy`, `error` (a danger [`Notice`] under the
explanation), `on_accept` and `on_cancel`. Cancel, Escape, the close button
and a press on the scrim all call `on_cancel`; the owner sets `open` to
false. While `busy` the accepting button spins and nothing closes it.

[`ConfirmHost`] and `ask` are promise-style confirms: render one `ConfirmHost`
near the app's root, then `ask(ConfirmRequest::new(title, explanation))`
from any code and `.await` the [`Answer`]: `true` when accepted, `false` when
cancelled, when another question replaced it, or when no host is mounted.
[`ConfirmRequest`] has the same fields as `Confirm`, public.

[`ConfirmPopover`] is the same beside an element: `anchor` (or
`anchor_element`), `open`, `side` and `align` as [`Popover`], `title` (its name, and the sheet's heading on a phone),
the same text, `danger`, `busy` and `error`, `on_accept` and `on_cancel`
(Cancel, Escape and a press outside; not while busy). The focus goes in once
it is placed (`Popover`'s `focus` prop), Tab stays inside it (`trap`), and
focus returns to the anchor.
With `compact`, it asks on one line, for a row's or a toolbar's own
"Remove Notes?": `explanation_icon` (an icon before it), `explanation`, then
`details` on the same line (such as the name in bold), above Cancel and the
accepting button joined at the end as their icons alone, `Glyph::Close` and
`icon` (else `Glyph::Check`), named (`aria-label` and `title`) by
`cancel_label` and `accept`, so an app passes its own words for both. It is
as wide as what it says, up to the viewport's width (not
`--popover-max-width`, so the line is not cut short), and a full-width sheet on a phone
without the head; the two are joined as an [`InputGroup`]'s members are.
[`ConfirmButton`] is a [`Button`] (`variant`, `size`, `icon`, `busy`, `disabled`)
that opens one beside itself and calls `on_accept` once accepted.

[`ConfirmInline`] replaces a button by No and Yes in the same box, for a row's
own delete: the button's `variant`, `size`, `icon`, `disabled`, children as
label, `no_label`, `yes_label`, `question` (the group's accessible name),
`danger` (Yes is `Danger`, else `Primary`), `busy` (Yes spins, No is
disabled), `on_accept`. It holds its own state, or the owner's: `asking`
(`Some`) with `on_ask`. No takes the focus; Escape and No go back and focus
returns to the button; a click does not bubble to a clickable row.

Every one takes `class`, and `testid` (`confirm-cancel` and `confirm-accept`
name the buttons). Classes: `CONFIRM` and `CONFIRM_TEXT` (the body),
`CONFIRM_POPOVER` with `COMPACT` (`confirm_popover(compact)` gives them) and
the compact parts `CONFIRM_LINE`, `CONFIRM_PAIR`, `CONFIRM_NO` and
`CONFIRM_YES`, `CONFIRM_INLINE` with
its parts `CONFIRM_IDLE` (the button) and `CONFIRM_ASK` (No and Yes), and
`AWAY` on the part not shown; `confirm_inline(part, away)` gives them.
Setting: `$confirm-gap` (between what it says and its buttons).

Size guarantees: `ConfirmInline` lays the button and the No and Yes in one
grid cell and hides (`visibility`) the one not shown, so its box is the same
asking or not: as wide as the wider of the two, as high as the button, and a
row it ends never moves. `ConfirmPopover` is `--popover-max-width` wide
whatever it says (a compact one is as wide as its line, fixed while open), and a flip moves it without reflowing; the page behind a
`Confirm` holds still, and the dialog hangs from the top, so an error line
only grows its bottom edge. A `ConfirmButton` keeps the button's box open or
closed.

<br>

## Tooltip, HoverCard and ReasonButton

All three place themselves with `use_popover` in the overlay layer, so they
inherit Popover's placement, following and Escape or press-outside dismissal.

[`Tooltip`] is a short tip beside its anchor. Props: `anchor` (a `NodeRef`),
`text` (nothing is shown, and nothing described, while it is empty), `side`
(`Side::Above` by default), `delay` ([`TOOLTIP_DELAY`], 250 ms of the pointer's
rest), `id`, `class`, `testid`. Keyboard focus (`:focus-visible`) shows it at
once; a touch pointer never does; a press, Escape, leaving or losing focus
hides it. The anchor gets `aria-describedby` naming the tip while it has
text. The tip (`.sockel-tooltip`, `sockel::class::TOOLTIP`; `tooltip(placed)`
gives its classes) is `--layer-tooltip`, `pointer-events: none`, hidden until
`PLACED`, and as wide as its text up to `--tooltip-max-width` (`$tooltip-max-width`,
24rem), wrapping beyond. [`DataTips`], mounted once, gives every element with a
`data-tip` attribute the same tip (none when it says what the element already
says, or while its menu is open, `aria-expanded="true"`); it places the tip by
a fixed stand-in (`TIP_ANCHOR`), and hides on scroll and Escape.

[`HoverCard`] is a richer card (`.sockel-hover-card` on a `.sockel-popover`,
`hover_card(placed)`): props `anchor`, `side`, `align`, `delay`
([`HOVER_CARD_DELAY`], 400 ms to open and to close), `title` (its accessible
name), `class`, `testid` and the card's children. It opens on hover or
keyboard focus and stays open while the pointer is over the anchor or the
card, so a link in it can be reached.

[`ReasonButton`] is a [`Button`] (the props `class`, `variant`, `size`, `icon`,
`submit`, `onclick`, `label`, `testid`, children) with `reason: Option<AttrValue>`
and `busy` with `working: Option<AttrValue>`: its tooltip says the reason, else
what it is doing (`reason_text`). A button with a reason is `unavailable`
(`aria-disabled`, not `disabled`), so it keeps focus and hover and a click does
nothing. `Button` has `unavailable` and `node_ref` props for this.

Size guarantees: the tip and card are `position: fixed` in the overlay layer,
hidden until measured, so showing them never moves the page, the anchor or the
scrollbar. A `ReasonButton` is one `<button>` whether it is available,
unavailable or busy: the same node, box and focus across the states (the busy
spinner and the aria state change, the width does not).

<br>

## Menu, RowActions, SplitButton and ContextMenu

[`Menu`] is a [`Button`] (`aria-haspopup="menu"`, `aria-expanded`) opening a
list of actions (`role="menu"`, `sockel::class::MENU`) beside it in the
overlay layer, through `use_popover`. Props:

- `text` (the opener's visible content; without it the opener is its icon
  alone), `icon` (the app's; a text opener without one ends in the
  IconSet's chevron, an icon-only one shows its ellipsis), `label` (the
  accessible name of the opener and the list), `variant` and `size` (the
  opener's, `Regular` so it matches the buttons beside it), `opener_class`,
  `tip` (the opener's `data-tip`), `title` (the opener's native tooltip).
- `on_click` and `pressed`: an opener that is a tool of its own. A click
  runs `on_click` (closing the menu if open) and a right-click (a long
  press on a touch screen) opens the menu; a second right-click while it
  is open reaches the browser's menu. `pressed: Some(on)` makes the opener
  a toggle (`aria-pressed`), which a right-click opens the same way, so an
  opener whose `on_click` comes and goes with its state stays one element.
- `width` ([`MenuWidth`]): `Regular` (`--menu-width`), `Wide` (`WIDE`,
  `--menu-width-wide`), `Wider` (`WIDER`, `--menu-width-wider`), [`Panel`]
  (`PANEL`, fixed at `--menu-width-panel`); `tight` (`TIGHT`) drops the gap
  between rows; `sheet` (`SHEET`, off by default): below `$sheet-breakpoint`
  a sheet from the bottom over a scrim (`.sockel-menu-scrim`), as a
  [`Popover`] is.
- `side`, `align` (`End` by default), `within` (as `Popover`'s),
  `disabled`; `open` and `on_toggle` for a menu its owner also opens another way; `class` (the list's),
  `testid` (the list's; the opener's is it suffixed `-toggle`, or
  `opener_testid` when set); `keep_mounted` (off by default) keeps the
  list rendered, `hidden`, while closed, for an item that owns what it
  opens (a dialog) and must not be dropped when the menu closes.

Its children are the items: [`MenuItem`] (an action; `icon`, `detail` under
the label and `detail_html` (markup) after it, `hint` at the end, `danger`,
`disabled`, `onclick`; with `href` a link row, an `<a>` that loses its
`href` when disabled, `new_tab` opening it in a new tab (`target="_blank"`,
`rel="noopener noreferrer"`), `download` a link that downloads; `title` the
row's native tooltip; `dialog: Some(open)` a button row opening a dialog
(`aria-haspopup="dialog"`, `aria-expanded`); `current` marks the page shown,
`aria-current="page"`, by the accent's ground and ink; `tip` a styled tip,
`data-tip`, naming what it acts on), [`MenuCheck`]
(`menuitemcheckbox`: `checked`, `on_change`; the menu stays open),
[`MenuRadio`] (`menuitemradio`: `checked`, `on_pick`), both with `icon`,
`detail` and `hint`; `stay` on a `MenuItem` or `MenuRadio` (`data-stay`,
which an app's own item can wear too) keeps the menu open on a pick, for
a step to another list of the same menu or a row of choices it holds; [`MenuSeparator`], [`MenuHeading`] and [`MenuNote`] (a line
such as "nothing here"). Each takes `class` and `testid`. `menu(width, tight, placed)` and `menu_item(danger)` give the
classes; `menu_step` and `typeahead` are the keyboard's pure parts.

Keyboard: focus moves to the checked radio once the list is placed, else
to the first item, else (a menu of fields alone) to its first control; Up
and Down (wrapping), Home and End move between enabled items, Left and
Right between the items of a focused item's `role="group"` (a row of
radios; a `stay` radio is picked as it is reached, as a radio group's
arrows pick), and typing
moves to the next item starting with what was typed (a pause of 0.7 s
starts over; one letter repeated cycles). In a menu that holds fields (a
small form), Tab and Shift+Tab move between its own fields and buttons and
close it only past either end, and a focused field keeps its arrows and
letters. A pick of an item or radio (or of an option of a single-select
[`ChoiceList`] the menu holds), Tab, Escape or a press outside closes
it; all but the press return focus
to the opener. A pick gives it back before the menu goes, so a dialog the
pick opens returns focus to the opener when it closes; Tab gives it back
and moves on from there (Shift+Tab to what comes before it). A menu is on the one overlay stack, so Escape and a press
outside close a menu opened from a popover or a dialog before what it
opened from. Where focus stays on the opener (a panel of text alone),
Escape and Tab there close it too, and Escape goes no further.

[`RowActions`] is a row's actions: `actions: Vec<RowAction>`
(`RowAction::new(icon, label, onclick)`, `.danger()`, `.pressed(on)`,
`.disabled(..)`, `.testid(..)`), shown as small quiet icon buttons, the
danger ones last (`ROW_ACTION_DANGER`, muted until hovered or focused),
then `children`, then the overflow `Menu` behind its ellipsis when `more`
holds its items (`more_label`; `more_disabled` keeps the
opener, disabled, so every row ends the same way; `more_testid` its test
id). The actions are named by `title`, or with `tips` by a styled tip
(`data-tip`, for [`DataTips`]), the opener's too. `disabled`, `open`,
`on_toggle`, `class`, `testid`.

[`SplitButton`] is a main action (`text`, `onclick`, `icon`, `busy`) joined
to a `Menu` of its variants (its children; `menu_label`;
`width`), `Primary` unless `variant` says otherwise, `size`, `disabled`,
`class`, `testid` (the menu's is it suffixed `-menu`).

[`ContextMenu`] wraps its children (`.sockel-context-target`,
`display: contents`, so it lays nothing out) and opens a menu of `items`
at the pointer on a right-click, or a press held for half a second on a
touch screen. Closed by Escape, a pick or Tab, it gives focus back to what
had it when it opened (Tab then moves on from there). `width`, `tight`, `within`, `label`, `disabled` (the browser's
menu instead), `on_toggle`, `class`, `testid`.

Size guarantees: the opener is a `Button`, so open and closed are the same
box (only `aria-expanded` changes). The list is fixed in a layer that lays
nothing out, hidden until placed. It is at least its width token wide and
never narrower than it opened (`--menu-open-width`, measured once on
open), so it does not follow its longest item as items change; past the
room on its side it scrolls (`scroll-area`), its scrollbar's room kept. Its
rows are `--control-height` at least, their border reserved, so hover and
focus change only the fill and border colour; a check's or radio's mark
keeps its box when off. Labels are one line, cut with an ellipsis.

<br>

## Table and SortHeader

[`Table`] is a table in a scroll area (`scrollbar-gutter: stable`, so rows
starting to scroll move nothing). `head` is the heading cells ([`TableCell`]
with `head`, or [`SortHeader`]s); the children are [`TableRow`]s of `TableCell`s.
`columns` is the count the `empty` row spans: `empty` (an [`EmptyState`] with
`compact`, or an [`ErrorBox`]) stands in the rows' place. `fixed` (`FIXED`)
sizes columns by the first row alone, so data arriving or a sort moves no
column; give a cell `width`, and `cut` to cut its text with an ellipsis.
`sticky` (`STICKY`, on the wrap) holds the head while the rows scroll past
`--table-max-height`. `bulk` is the bulk actions above the table: given,
its room (`TABLE_BULK`, one height) is always kept, and `bulk_active` shows
it, so selecting rows moves nothing; `IDLE` hides it. `numeric` on a cell
(`NUMERIC`) sets figures of one width, to the end. `table(fixed)`,
`table_wrap(sticky)`, `table_bulk(idle)` and `table_cell(head, numeric, cut)`
give the classes. Parts: `TABLE_FRAME`, `TABLE_WRAP`, `TABLE_ROW`,
`TABLE_CELL`, `TABLE_HEAD_CELL`, `TABLE_EMPTY`.

`SortHeader` is a heading cell with a button: `label`, `direction`
(`Option<SortDirection>`; `aria-sort` is `ascending`, `descending` or
`none`), `first` (the way a first press runs: names from A, sizes largest
first), `numeric`, `disabled` and `onsort`, called with the direction asked
for. `next_sort(current, first)` is the cycle: unsorted, `first`, the other
way, unsorted. The mark (`SORT_HEADER_MARK`, a chevron from the [`IconSet`])
has its box whether sorted or not (`IDLE` hides it), so sorting never changes
the heading's width. `sort_header(numeric)` and `sort_mark(idle)`.

Every one takes `class` and `testid`. Size guarantees: a row and a heading
are at least `--control-height` high, every line is a reserved border, and
hover tints the row without changing a box; the mark and the bulk bar keep
their room. For markup sockel does not render, the mixins `table-base`
(`$fixed`), `table-head-cell`, `table-cell` and `table-row-hover` give the
same rules. New token: `--table-max-height`.

<br>

## Pager and Disclosure

[`Pager`] is page buttons in a fixed number of slots: first, previous, the
numbers with gaps (`…`), next and last. `page` (from 0), `pages`, `on_page`
(called with the page asked for), `slots` (default 7, at least [`MIN_SLOTS`],
5), `edges` (default true: the first and last page buttons; hidden below the
phone breakpoint, and worth turning off in a narrow container), `phone_steps`
(default true: the previous and next page buttons below the phone breakpoint;
false hides them there with `DESKTOP_ONLY`, so `_utilities.scss` must be
loaded), `titles` (default false: every button's accessible name as its
`title` too, a tooltip), `label`,
`first_label`, `previous_label`, `next_label`, `last_label`, `page_label`
(given, the number is appended) and `disabled` and `busy` (the buttons keep
their room and ignore presses). `pager_slots(pages, page, slots)` is the pure
layout: every page when they fit, otherwise exactly `slots` entries, the
first and last page always among them and each gap hiding at least two
pages, so the count does not change with the page. `page_count(total, size)`
rounds up, at least one. The current page is marked by `aria-current`, its
colour and background, never its weight. `pager(disabled, busy)` and
`pager_button(edge)` give the classes. Parts: `PAGER_BUTTON`, `PAGER_GAP`,
`PAGER_MARK`, `PAGER_FLIP`, `PAGER_ECHO`, `PAGER_EDGE`.

`summary: Option<PagerSummary>` makes it a summary pager (`SUMMARY`): in
place of the numbers and the first and last page, which items the page shows
over where it stands (`PagerSummary::new(shown, position)`, the app's words,
such as "1–50 of 312" and "Page 1 of 7"), in a polite live region, then previous and next at the row's
end. A page past the last stays as given there, so previous leads back to the
last page. The setting `$pager-summary-padding` (`null`: none) pads it, for
one that ends a list or table and lines up with its rows. `nav_ref` is the
`nav` element, for an app that scrolls to what it
pages. Parts: `PAGER_STATUS`, `PAGER_SHOWN`, `PAGER_POSITION`, `PAGER_STEPS`.

[`Disclosure`] is a summary row that toggles its body: `title`, `summary` (what
it holds, beside the title while shut), `actions` (controls at the row's end,
outside the toggle), `open`, `on_toggle` (the state a press asks for), `id`
(the body's, named by `aria-controls` while open), `disabled`, `level` (a
[`HeadLevel`]: the toggle sits in that heading, `DISCLOSURE_HEADING`, its title
at a list head's size, for a section whose title is a heading), `class`,
`testid`, and the children as the body, rendered only while open.
`disclosure(open)` gives the classes (`DISCLOSURE`, `OPEN`),
`disclosure_summary(heading)` the toggle's (`HEADING`). Parts:
`DISCLOSURE_HEAD`, `DISCLOSURE_SUMMARY`, `DISCLOSURE_MARK`, `DISCLOSURE_TITLE`,
`DISCLOSURE_DETAIL`, `DISCLOSURE_ACTIONS`, `DISCLOSURE_BODY`.

Size guarantees: the Pager's slots are as many whatever the page, and each
button and gap is as wide as the widest page number (`--pager-chars`, the
digits of `pages`) with `numeric` figures, at least `--control-height`, so
its width holds from the first page to the last (in a container too narrow
the buttons shrink together, the same at every page); the current page adds
no weight. A summary pager's counts are `numeric` and cut to one line each,
its two buttons at its end, so only the counts' text changes with the page.
The Disclosure's row is `--control-height` high shut or open
(the setting `$disclosure-row-height` sets another, such as
`var(--touch-height)`; `$disclosure-title-ink`, `null` by default, colours
the title), the
mark turns inside its own box, and the title and summary are cut to one line;
only the body adds height. New classes: `PAGER`, `PAGER_BUTTON`, `PAGER_GAP`,
`PAGER_MARK`, `PAGER_FLIP`, `PAGER_ECHO`, `PAGER_EDGE`, `DISCLOSURE`,
`DISCLOSURE_HEAD`, `DISCLOSURE_SUMMARY`, `DISCLOSURE_MARK`,
`DISCLOSURE_TITLE`, `DISCLOSURE_DETAIL`, `DISCLOSURE_ACTIONS`,
`DISCLOSURE_BODY` (`OPEN` is new too). New partials: `_pager.scss`,
`_disclosure.scss`. New tokens: none (`--pager-chars` is set per Pager).

<br>

## Select and Listbox

`Select<T>` is the WAI-ARIA select-only combobox: a face in the tab order
(a `<button>`, `role="combobox"`, `aria-haspopup="listbox"`) showing what is
chosen, opening a listbox (`role="listbox"`, `sockel::class::LISTBOX`) under
it in the overlay layer through `use_popover`, while focus stays on the face
(`aria-activedescendant` names the active option). Props:

- `options: Vec<SelectOption<T>>` (`SelectOption::new(value, label)`,
  `.icon(html)` before its label in the list, `.disabled(..)`,
  `.description(html)` a muted line under its label, such as why it cannot
  be picked, `.group(name)` the heading it is listed under: a group's
  options are given together, under a `role="group"` named by its
  heading, `.testid(id)` its row's test id), `value:
  Option<T>`, `on_change` (told a pick that is not the one chosen),
  `placeholder` (muted, while nothing listed is chosen).
- `width` ([`SelectWidth`]): `Widest` (the widest option or placeholder) or
  `Fill` (`FILL`, its container's width).
- `size` ([`InputSize`], as a [`TextInput`]'s), `invalid`, `disabled`, `busy`
  (`aria-busy`; it still opens on what it has).
- `filter_from: Option<usize>`: from how many options the list has a
  filter field above it (`None`, the default: never). It takes focus on
  opening, typing on the face (closed, or open with focus back on it) goes into it, and it narrows the list to the
  options whose label or group holds its text, ignoring case
  (`filter_matches`); the arrows, PageUp, PageDown, Enter and Alt+ArrowUp
  still move and pick, Home, End and Space are the text's, and Tab closes
  it from the face. `filter_label` (its placeholder and name) and
  `no_match` are its words, the catalog's unless given. The list is then
  `SELECT_POPUP`: the field (`SELECT_FILTER`, staying at the top as the
  options scroll), the `role="listbox"` (`SELECT_OPTIONS`) and
  `SELECT_EMPTY`; the field's test id is the face's suffixed `-filter`.
- `multiple`: picks several, `values: Vec<T>` in place of `value`, told
  through `on_change_values` (the values picked after a check or uncheck,
  in the options' order). A pick leaves the list open; each picked option
  shows its check, the list is `aria-multiselectable`. The face names the
  first picked, or while several are, `summary` (such as how many rows they
  match) or by default the first and how many more (`Small +2`); that text
  joins the labels' cell, so a summary longer than every label widens it.
  `picked(options, multiple, value, values)` gives the picked indexes.
- `sheet` (off by default): below the phone breakpoint the list is a
  sheet from the bottom over a scrim (`SELECT_SHEET`, `SELECT_SCRIM`),
  headed by `label` and a close button (`SELECT_HEAD`, `SELECT_TITLE`;
  `close_label`) above what scrolls (`SELECT_BODY`); its filter
  field is not focused on opening there, so the keyboard does not cover
  the sheet. Above the breakpoint the list is as without it.
- `id` and `describedby` inside a [`Field`] (its label names and opens it),
  `label` (the accessible name without one), `class` (the face's), `testid`
  (the face's; the list's is it suffixed `-options`).

In an [`InputGroup`], give it `group_classes(..)[i]` as its `class` like any
member. `select(size, width, invalid, disabled, busy)` gives the face's
classes (the input box's, `SELECT`, `FILL`), `chosen(options, value)` the
chosen option's index.

Keyboard: closed, the arrows, Enter, Space, Home and End open it (on the
chosen option, else the first or, for ArrowUp and End, the last enabled
one), and typing opens it on the option starting with what was typed.
Open, the arrows (stopping at the ends), PageUp and PageDown (10), Home and
End move over the enabled options, typing moves as a [`Menu`]'s type-ahead
does, Enter, Space and Alt+ArrowUp pick, and Escape, Tab or a press outside
close it. A pointer over an option makes it active; a press on the list
keeps focus on the face.

[`Listbox`] is the list's keyboard state, for anything listing options under
what keeps focus (P12's searching pickers): `active`, `hover`, `set`,
`step`, `go_to_end`, `settle` (after filtering), `typeahead(pickable,
labels, char, now)`, `key(key, alt, pickable) -> ListKey` (`Moved`,
`Pick(i)`, `Ignored`) and `take_scroll` (scroll the active option into
view), or `scroll_active(base)` doing that scroll from `rendered`.
`printable(event)` is the character a key types (none with Ctrl, Meta or
Alt), for a filter or type-ahead beside the list. `option_id(base, i)`, `listbox(placed)` and `listbox_option(active)`
(`LISTBOX_OPTION`, `ACTIVE`) give the ids and classes; an option's
`aria-selected` and `aria-disabled` carry the rest. Its parts are
`LISTBOX_OPTION_MARK` (the selected one's check, its box kept when off,
`OFF`), `LISTBOX_OPTION_ICON` and `LISTBOX_OPTION_LABEL`, and with a
description `LISTBOX_OPTION_TEXT` holding the label and
`LISTBOX_OPTION_DESCRIPTION`; a group is `LISTBOX_GROUP` with its
`LISTBOX_GROUP_NAME`.

Size guarantees: the face is one width whatever is chosen. Every option's
label and the placeholder are in one grid cell, all but the shown one
`HIDDEN`, so `Widest` is the widest of them. Past its container it is cut
with an ellipsis, never wrapped. `Fill` is the container's width. It is
the input box's height for its size, and open, focus, hover and invalid
change colours only. The face shows no option icon, which would change its
width. The list is the menu's surface (fixed in a layer that lays nothing
out, hidden until placed), at least as wide as the face
(`--listbox-anchor-width`) and never narrower than it opened
(`--listbox-open-width`). Its rows are `--control-height` at least, their
border reserved, and the active row changes only its fill. Labels are one
line; a description under one wraps, and makes its row taller. New
tokens: none (the two custom properties are set by the component). New partials: `_listbox.scss`, `_select.scss`. The menu's
surface, row and mark are the mixins `list`, `item`, `item-active` and
`mark` of `_menu-surface.scss` (forwarded by `_menu.scss`), which emits no
rules, so the listbox and the Select load none of the menu's. `.sockel-hidden`
is `_hidden.scss`, which `_badges.scss` and `_select.scss` both load.

<br>

## DropZone

A target for files dropped on it, pasted, or picked with its button. While a
file is dragged over the page the zone is outlined, over it tinted; a file of
a type it does not take is refused. All of that, and the busy and error
messages, is drawn by an overlay inside the zone, so the zone never changes
size.

```rust
html! {
    <DropZone accept="image/*" max_size={Some(5_000_000)} paste=true
        pick="Choose a file" {on_files} {on_refused}>
        <p>{ "Drop images here" }</p>
    </DropZone>
}
```

Props: `accept` (as a file input's: media types, `image/*`, extensions; empty
takes anything), `max_size` (bytes), `multiple` (otherwise the first file
only), `label`, `reject` and `too_large` (what the overlay says), `busy`
(`Some(text)` while a file is worked on; the zone takes nothing then),
`error` (text, until the app clears it), `disabled`, `paste` (files pasted
outside a text field go the same way), `tall` (a zone that may be taller
than the screen: its label, larger, stays on screen while the zone
scrolls; omittable part `drop.tall`), `pick` (a button with this label
opens the file picker through a hidden input), `on_files` (the files taken,
in order, from a drop, paste or pick), `on_refused` (a [`Refusal`]: `Type` or
`TooLarge`), `testid`, `class`. `accepts(accept, name, kind)` and
`refusal(accept, max_size, name, kind, size)` are the same filter as plain
functions. A refusal the zone found itself is said until the next press in
it or the next drag; a drag that carries no files passes through.

Modifiers, set from [`DropState`] by `drop_zone(state, disabled)`:
`.sockel-dragging` (a file is dragged on the page), `.sockel-over`,
`.sockel-refused`, `.sockel-busy`, `.sockel-error` and `.sockel-disabled`.
Parts: `.sockel-drop-zone-overlay`, `-label` and `-pick`.

Size guarantees: the zone has a border at all times (`reserved-border`,
transparent at rest), and every state changes only colours, so its box is the
same at rest, dragged over, refused, busy and in error, with a short message
or a long one (the label is cut at two lines inside the overlay, which
clips). The overlay is absolutely placed and lets the drag through. The
zone's least height is `--drop-min-height`.

Tokens: `--drop-min-height`, `--drop-tint` and `--drop-tint-over`.

Settings: `$drop-overlay-inset` (`0`; a negative one lays the overlay past
the zone's edge) and `$drop-edge-width` (`null`: the zone's own one-pixel
border; set, the zone has no border and the overlay draws the edge at this
width, dashed while dragged near), for an app whose outline sits outside
the zone without moving anything; `$drop-tall-label-offset` (`40vh`: how
far inside the viewport's top and bottom a tall zone's label stays);
`$drop-busy-pulse` (`null`; a duration, and the busy label, which carries
`.sockel-busy`, pulses at it, still under reduced motion).

<br>

## ColorSelect

[`ColorSelect`] picks a colour from a palette the app passes in as data; sockel
owns no palette. It is the select-only combobox pattern of [`Select`]: a face
(a `<button>`, `role="combobox"`) with the chosen colour's swatch and name,
opening a grid (`role="listbox"`) of swatches in the overlay layer through
`use_popover`, while focus stays on the face. Props:

- `options: Vec<ColorOption>` (`ColorOption::new(value, label)`; the value is
  any CSS colour, and an empty one is "no colour", drawn struck through),
  `value: Option<AttrValue>`, `on_change` (told a pick that is not the one
  chosen), `placeholder` (muted, while `value` is `None`).
- `custom: Option<AttrValue>`: the label of a row under the grid holding the
  browser's native colour input (`#rrggbb`); a colour picked there is told
  to `on_change` like any other, and a value that is not in the palette is
  shown by its own text.
- `columns` (6): swatches to a grid row (`--color-select-columns`).
- `width` ([`ColorSelectWidth`]): `Fixed` (`--color-select-width`), `Fill`
  (`FILL`, the container's) or `Swatch` (`SWATCH_ONLY`, the swatch alone in a
  box as wide as it is high).
- `size` ([`InputSize`]), `invalid`, `disabled`, `busy` (`aria-busy`; it still
  opens on what it has), `id` and `describedby` inside a [`Field`], `label` (the
  accessible name without one), `class` (the face's), `testid` (the face's;
  the grid's is it suffixed `-options`, the custom input's `-custom`).

In an [`InputGroup`], give it `group_classes(..)[i]` as its `class`.
`color_select(size, width, invalid, disabled, busy)`, `color_swatch(none)`,
`color_list(placed)` and `color_option(active)` give the classes
(`COLOR_SELECT`, `COLOR_SWATCH` with `NONE`, `COLOR_LIST` with `PLACED`,
`COLOR_OPTION` with `ACTIVE`; parts `COLOR_SELECT_LABEL`,
`COLOR_SELECT_PLACEHOLDER`, `COLOR_SELECT_CHEVRON`, `COLOR_GRID`,
`COLOR_OPTION_MARK`, `COLOR_CUSTOM` with `_LABEL` and `_INPUT`);
`color_index(options, value)`, `color_name(options, value)` and
`color_input_value(value)` give the chosen option, its name and the native
input's value.

Keyboard: closed, the arrows, Enter, Space, Home and End open it on the
chosen colour (else the first or last), and typing opens it on the colour
whose name starts with what was typed. Open, Left and Right move by one,
Up and Down by a row (stopping at the ends), Home, End, PageUp and PageDown
and type-ahead move as in a [`Listbox`], Enter, Space and Alt+ArrowUp pick,
Escape or a press outside close it, and Tab moves into the custom input
(with `custom`; Shift+Tab or Tab from it closes and returns to the face).

Size guarantees: the face is one width whatever is chosen or loaded, the
label cut with an ellipsis, never wrapped; it is the input box's height for
its size, and open, focus, hover and invalid change colours only. A swatch
is `--swatch-size` square, a grid cell `--control-height` square with its
border reserved, and the active and selected cells change fill and border
colour only; the selected mark keeps its box when off. The popup is the
menu's surface in a layer that lays nothing out, hidden until placed.
New tokens: `--swatch-size`, `--color-select-width`. New partial:
`_color-select.scss`; new classes as above (`SWATCH_ONLY` is new; `NONE`,
`FILL`, `PLACED`, `ACTIVE` and `OFF` are shared).

<br>

## SearchSelect and FilterPicker

`SearchSelect<T>` is the WAI-ARIA editable combobox with list
autocomplete: a field (the input box, `SEARCH_SELECT`) whose text filters a
listbox under it in the overlay layer, focus staying in the field. Props:

- `options: Vec<SearchOption<T>>` (`SearchOption::new(value, label)`,
  `.detail(..)` muted after the label and searched too, `.icon(html)`,
  `.group(..)` the heading it is listed under, adjacent options sharing
  one, `.disabled(..)`), `value: Option<T>` (shown in the field while it is
  not being edited), `on_change`.
- `filter` (on by default: `search_matches(options, query)`, label or
  detail containing the query, ignoring case), `on_search` (each edit, for
  options loaded elsewhere; turn `filter` off), `answers` (the query the
  options answer: until it is the one typed they are stale, and a loading
  row stands in their place, so a late answer to an older query is never
  shown), `busy` (a spinner in the field, `aria-busy`, and the loading row
  while there are no options).
- `label` (the placeholder and accessible name), `loading_label` and
  `empty_label` (the app's words), `id`, `describedby`, `size`, `fill`
  (`FILL`), `invalid`, `disabled`, `autofocus`, `class`, `testid` (the
  list's is it suffixed `-options`).

`listed(options, query, answers, busy, filter)` says what an open list shows
(`Listed::Loading` or `Listed::Options(indices)`), `search_select(size,
fill, invalid, disabled, busy)` gives the field's classes. Keyboard:
ArrowDown and ArrowUp open it; open, the arrows, PageUp and PageDown move
over the enabled options (Home, End and Space edit the text), Enter picks,
Escape, Tab or a press outside close it and give the field back what is
chosen. Typing makes the first match active. Parts:
`SEARCH_SELECT_LIST`, `SEARCH_SELECT_GROUP` with `SEARCH_SELECT_HEADING`,
`SEARCH_SELECT_DETAIL`, `SEARCH_SELECT_NOTE` (the loading or no-match row).

`FilterPicker<T>` is a list's filter, for a [`LibraryToolbar`]: a button
(`BUTTON`, `FILTER_PICKER`, `ON` while anything is chosen) saying `all`, or
the first choice and `+N` more, opening a multi-select listbox
(`aria-multiselectable`): an option for all, then one per choice, checked
when chosen. A pick leaves it open. Props: `choices: Vec<FilterChoice<T>>`,
`chosen: Vec<T>` (in the order chosen), `on_change` (the choices after a
pick, `toggled(chosen, value)`; empty for all), `label` (the accessible
name and tooltip before what is chosen), `all` (an override of the catalog's word), `disabled`, `class`, `testid`. Keys as a [`Select`]'s. Past
[`SEARCH_FROM`] (8) choices the pop-up is a [`ChoiceList`] (`multi`, all as
its first option) with its search field, prefix matches first and its
`--picker-rows` height whatever matches; the field takes the focus.
`filter_summary(all, labels)` gives its words and count. Parts:
`FILTER_PICKER_TEXT`, `FILTER_PICKER_COUNT` (`OFF` with none),
`FILTER_PICKER_CHEVRON`, `FILTER_PICKER_LIST`.

Size guarantees: SearchSelect's field is `--menu-width-wide` (or its
container's, `Fill`) and the input box's height, whatever is typed or
chosen. Its list is `--menu-width-wide` or the field's width if wider,
never measured from its text, and `--picker-rows` rows high (or what fits
below the field), so typing, results arriving, loading and no matches
never resize it; past that it scrolls. FilterPicker is `--filter-width`
wide from no choice to all of them: its words are cut with an ellipsis and
its count's room (`3ch`) is kept, hidden, with none. New token:
`--picker-rows` (`$picker-rows: 6`). New partials: `_search-select.scss`,
`_filter-picker.scss`.

<br>

## Picker, PickerMenu, ChoiceList and PickerDialog

`ChoiceList<T>` is the list every picker opens: a listbox (`CHOICE_LIST`,
`CHOICE_LIST_OPTIONS`) of `PickerOption<T>`s under a search field
(`CHOICE_LIST_SEARCH`, an input box) once there are more than
[`SEARCH_FROM`] (8) options, or with `always_search`. Focus stays in the
field (or on the list, without one), naming the active option with
`aria-activedescendant`: the arrows, PageUp and PageDown move over the
enabled options and Enter picks; without a field, Home, End, Space and
type-ahead too. Props: `options`, `value` (checked), `multi`
(`aria-multiselectable`, `chosen` checked, the owner toggles a pick with
`toggled`), `on_pick`, `label` (the list's name), `always_search`,
`search_label` and `empty_label` (the app's words), `autofocus`, `class`,
`clear_label` (the search field's clear button: it is a [`SearchField`], so
Escape clears a query before it closes a popup), `detail_below` (each
row's detail on a second line under its label, wrapping, in
`LISTBOX_OPTION_TEXT` and `LISTBOX_OPTION_DESCRIPTION` as a [`Select`]'s
description), `testid` (the field's is it suffixed `-search`, each
option's `-option`, the pager's `-pager`). Under a search field no option
starts active (unless `tab_cycles`): ArrowDown goes to the first, ArrowUp
to the last, Enter picks the first match, and the current option is
scrolled into view. `on_search` is told the query as it changes (and
blank when the list mounts), for options the app finds by it. `page_size`
lists that many matches at a time with a [`Pager`] under them
(`CHOICE_LIST_PAGER`, over a rule), opening on the current option's page;
a new query goes back to the first. `tab_cycles` makes Tab and Shift+Tab
move the active option, wrapping, instead of leaving (a list adding one
thing after another). `rows` sets a searching list's height in rows
(`--picker-rows`), for a list whose every option should show. A keyboard
pick is the option's click, so a [`Menu`] holding a single-select list
closes on it as on a mouse pick.

`PickerOption::new(value, label)` with `.detail(..)` (muted after the
label, searched), `.icon(html)` (a slot kept on every row once any option
has one), `.badge(html)` (at the row's end), `.keywords(..)` (searched, not
shown), `.disabled(..)`, `.marked(..)` (checked beside the current one,
such as those already chosen), `.found(..)` (the app's answer to the
query: listed whatever it reads like) and `.data_value(..)` (the value as
text, the row's `data-value`). Rows are the listbox's (`LISTBOX_OPTION` with
its `-mark`, `-icon`, `-label`), plus `CHOICE_LIST_DETAIL`,
`CHOICE_LIST_BADGE`, and `CHOICE_LIST_NOTE` for no matches.
`picker_matches(options, query)` finds every word in the label, detail or
keywords, ignoring case, labels starting with the text first, then labels
with a word starting with it, then the rest.

`Picker<T>` is a field (the input box, `PICKER`) showing the current
option's icon and label (`PICKER_ICON`, `PICKER_TEXT`), or the placeholder
(`PICKER_PLACEHOLDER`), and a chevron (`PICKER_CHEVRON`), opening a
[`ChoiceList`] below it in the overlay layer (`PICKER_POPUP`, the menu's
surface). A pick, Escape, Tab or a press outside closes it (Tab moving
on from the field, as if it had not been open); ArrowDown or ArrowUp
opens it. Props: `options`, `value`, `on_change`, `label` (the
accessible name, followed by the current option's label), `placeholder`,
`variant` (`PickerVariant::Field` or `Small`, `SMALL`), `fill` (`FILL`),
`invalid`, `disabled`, `always_search`, `search_label`, `empty_label`,
`on_search`, `page_size`, `clear_label`, `detail_below`, `id`, `describedby`, `title` (the field's
tooltip), `fold` (the current option's icon alone below the phone
breakpoint, its label kept for a screen reader), `class`, `testid` (the popup's is it suffixed `-popup`, the
list's `-list`, or `list_testid`). `picker(variant, fill, invalid, disabled)`
gives its classes. `PickerMenu<T>` is the same list opened from a [`Button`]
for an action that takes an option (Add a language): `text`, `icon`,
`chevron` (beside the icon too), `label`, `variant`, `size`, `fold` (as a `Button`'s), `tip`, `disabled`, `on_pick`,
`field` (the opener is a [`Picker`]'s field showing `icon` and `text`, not a
button: a field of several choices, such as a filter; ArrowDown and ArrowUp
open it) with `fill`, and the list's
`value`, `multi` and `chosen` (a multi pick leaves it open), `on_search`, `page_size`, `tab_cycles`, `rows` and `list_testid`; `width` (the popup's, as [`ChoicePopup`]'s); `align`
(`Start` by default), `open` and `on_toggle` for a list its owner also closes, `head` (above the list, a note) and
its children (below it, a Done button). `PickerDialog<T>` is the
searchable shell for a large set (languages, countries): a [`Dialog`]
(`open`, `on_dismiss`, `title`, `size`, `actions` such as Done) holding a
`ChoiceList` that always searches, its field focused, single (the owner
closes it on a pick) or `multi`. `ChoicePopup` is the list's popup on its own, for an
opener the app renders (a `Button` with `node_ref`, a chip, a face): props
`anchor` (the opener's ref), `open`, `on_dismiss` (Escape, Tab or a press
outside; a pick is the owner's to close on), `align` ([`PopoverAlign`],
default `Start`), `width` (its least width in place of
`$picker-popup-width`, a length such as `var(--menu-width-wide)`; the
`--picker-popup-width` custom property), `testid` (suffixed `-popup`) and its children, a
`ChoiceList` with `autofocus` and any note or action beside it, which it
focuses once placed. Tab closes it and moves on from the opener.

Size guarantees: the Picker's field is `--menu-width-wide` wide (small:
`--menu-width`; its container's with `Fill`) and the input box's height,
whatever is chosen; its label is cut with an ellipsis. In a row without
room for it, it gives way to the items beside it rather than overflowing. The popup is
`$picker-popup-width` (`--menu-width-wide`) or the field's width if wider,
never measured from its text. A ChoiceList's options are `--picker-rows`
rows high whatever matches, none included, so typing never resizes the
list, its popup or its dialog (without a search field, which never
filters, its rows' height, two-line rows included, as far as its popup allows); past that it scrolls, its scrollbar's room kept. Marks keep their
box when off. New partial: `_picker.scss`. No new tokens.

<br>

## ToolBand, Panel, Seam and use_width

**ToolBand** is a [`Toolbar`] with `band`: its controls wrap onto further
lines instead of scrolling sideways (`.sockel-toolbar.sockel-band`; no
hidden scrollbar or fade). `chrome` sets it on `--band-chrome-ground`
(`--surface`) with a rule under it (`.sockel-chrome`, an inset shadow, so
no border is added), its controls `--band-chrome-gap` apart; `search` sets
it in `--band-search-text` (`--text-sm`), as a list's filter
(`.sockel-search`). `tool_band(chrome, search)` gives a band's classes for
markup that is not the component.
Group its controls in [`ToolbarPart`]s (`.sockel-toolbar-part`, `role=group`):
a band wraps between parts, and inside a part only when that part alone is
wider than the band (it never overflows the band).
`ruled` rules a part off from the one before it (`.sockel-ruled`, a
`--border` line and `--space-3` in) from the phone breakpoint up, where a
band has the room. A band's height changes only when it wraps, so give its
controls fixed widths (a [`Button`]'s busy and disabled states keep its box);
`toolbar_part(ruled)` gives a part's classes.

**Panel** is a ground over a list or a view holding its filters or a short
form (`.sockel-panel-box`): a column `--space-2` apart, on `--panel-ground`
in a `--panel-border` ring that is always there, in `--panel-text` type,
`--panel-space-after` above what follows, as wide as what it holds and never wider than its column. `line` lays it out
as one row that wraps (`.sockel-line`); `ground` is `Sunk`, `Tint` (a
`--panel-tint-radius` `--panel-tint` chip in `--panel-tint-text`,
`--panel-tint-space-after` above what follows,
`.sockel-tint`) or `Bare` (only the spacing, `.sockel-bare`); the ring
turns transparent rather than going, so the ground never changes its box.
`centered` centres it in its column (`.sockel-center`); `small` sets it in
`--text-sm` (`.sockel-small`). With a `label` it is a named `group`.
`panel(ground, line, centered)` gives its classes; add `class::SMALL` for
`small`.

**Seam** is two panes, `start` and `end`, and the divider dragged between
them (`.sockel-seam`, `.sockel-seam-pane`, `.sockel-seam-grip`). `at` is the
first pane's share of the room beside the divider (0 to 1); the owner keeps
it and sets it from `on_change`, which a drag calls at most once an
animation frame, with the pointer captured, and `on_settle` once the drag
ends. The divider is a `separator`: the arrow keys move it by 5%, Home and
End to the ends, and a double-click returns it to `reset` (0.5). `min` and
`max` bound the share, and each pane keeps `--seam-min` (8rem) besides,
both in what it reports and in the grid, so neither pane collapses and an
`at` out of range still lays out. The divider is `--seam-grip` wide
whatever its state: hover, focus and a drag (`.sockel-active` on the seam)
light its line only. `axis: SeamAxis::Rows` stacks the panes
(`.sockel-rows`); give the seam a height. Each pane scrolls what overflows
it, so its content never widens it.

**`use_width(&node)`** is the width of `node`'s element (its border box, in
CSS pixels) from a `ResizeObserver`: `0.0` until it is first measured, then
at most one new value an animation frame, and only when it changed, so a
component laid out against its own width renders once a frame of a resize.
The observer goes with the component.

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
| | `.sockel-toolbar` | Scrolls sideways with its scrollbar hidden, so its height does not change when its controls start to overflow; the edge with hidden controls fades (a CSS scroll-driven animation: browsers without `animation-timeline` show no fade). Wheel, touch and keyboard scroll as before. |
| `truncate` | `.sockel-truncate` (`TRUNCATE`) | One line, cut with an ellipsis. |
| `clamp($lines)` | `.sockel-clamp-2`, `-3`, `-4` (`CLAMP_2`...) | At most that many lines, the last cut with an ellipsis. |
| `visually-hidden` | `.sockel-visually-hidden` (`VISUALLY_HIDDEN`) | Read by a screen reader, takes no room (a hidden h1). |
| `phone-only`, `desktop-only` | `.sockel-phone-only`, `.sockel-desktop-only` | Shown only below `$visibility-breakpoint`, or only from it up. |
| `current($by: background)` | | Marks `[aria-current]` (not `"false"`) and `[aria-selected="true"]` by `background` (`--accent` under `--on-accent`), `color` or `underline`; never by weight, so the item keeps its width. |
| `scroll-fade` | | Scrolls sideways with the scrollbar hidden and the edge that has hidden content faded ([`Toolbar`], [`Tabs`]). |

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
| `button($variant: secondary)` | `.sockel-button` with its states; `button-variant($variant)` is a variant's colours alone (`primary`, `ghost`, `quiet`, `danger`, `success`) |
| `table-base`, `table-head-cell`, `table-cell`, `table-row-hover` | a table's (see Table) |

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
  in `html!` still compiles; a struct literal or a field read needs
  `Some(..)`/`.as_deref()`. [`ConfirmRequest`]'s `accept` and `cancel_label`
  are `Option` too, and `ConfirmRequest::new` leaves them `None`.
- [`Reorder`]'s announcements are `sockel-reorder-*` messages with the
  position and count as numbers; `text` still overrides them, and
  `reorder_text` returns the catalog's English.

sockel's own rule: no user-visible text as a literal in a component. Every
word comes from the catalog, with numbers passed as numbers so the language
formats them. `tests/english_literals.rs` fails on a capitalised literal in
`src/` outside a short list of key names, and the catalog tests check
`en-US`/`sv-SE` parity and the ids the code asks for against the catalog,
both ways. The showcase provides the catalog and switches language (the
Translations page).

[phrasebook-catalog]: https://docs.rs/phrasebook-catalog

<br>

## Showcase

`showcase/` is an interactive gallery of every component: a page for
each, with controls for its props and its content, shown in a narrow
frame (drag its corner to widen it) beside a wide one. Run it with
[trunk]:

```sh
cd showcase
trunk serve --port 8095 --open
```

Pick a free port. The showcase dogfoods sockel: its page controls and
its Theme picker are sockel's own [`Select`], so every page also exercises
it. Neither the showcase nor sockel's components render a native
`<select>`, whose open list the OS draws outside sockel's tokens.

The showcase's stylesheet is also built with [trug] (`cargo install trug`,
or point `TRUG` at a binary) by a trunk `post_build` hook,
`showcase/scripts/trug-css.sh`, which builds the CSS with trug and
fails if it differs from the CSS `showcase/build.rs` wrote to
`target/showcase-css/`. Without trug on `PATH` the hook skips the check.

A new component lands with its showcase page: add it to
`PAGES` in `showcase/src/pages/mod.rs`, listing the names it covers, or
`cargo test --workspace` fails.

Under its frames, a page shows the code that renders its component as
the controls set it, highlighted, with a Copy button. A page builds it
with `showcase/src/code.rs` (`El` for each element, its props and
children, and `snippet` around them), listing only the props that differ
from their defaults, and passes it to `Demo` as `code`. A page without a
`Demo` shows a `CodeCard` itself; the test fails for a page that shows
no code.

<br>

### Stability probe

Each page also declares the boxes that must hold (`holds`, next to its
`covers`): a part's class, which of its size holds (`Size::Width`,
`Height`, `Both`, or `Box` for its size and place), and across which
states (`Across::All`, `Only` or `Except` a list of control labels, a
single value as `"label=value"`, or the probe's own `focus` and
`click`). The test fails for a page without one, unless it is listed in
`UNPROBED`.

The panel in the window's corner (fixed, so it never moves the page)
runs the probe: it sets each control to each of its values in turn,
one control at a time (opening its Select and clicking the option),
toggles each checkbox, then focuses each control in the frames and
clicks each button there (pressing Escape after each click, so nothing
it opened stays open), measuring every declared
part in both frames after each state. Its `<pre>` reads `probe: stable`
when no declared box moved or resized by more than half a pixel, and
otherwise lists each change by state, frame and part. The panel's
`data-probe` attribute carries the verdict (`idle`, `running`, `stable`,
`unstable`), and `?probe` before the page's `#id` runs it on load, so a
browser check reads the result as text. Hover cannot be scripted, so it
stays a manual check.

<br>

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --workspace -- -D warnings
cargo test --workspace
bash scripts/check-features.sh
bash scripts/check-packaged.sh
(cd showcase && trunk build)
```

[Yew]: https://yew.rs
[sasso]: https://crates.io/crates/sasso
[trunk]: https://trunkrs.dev
[trug]: https://crates.io/crates/trug
