# plinth

Opinionated layout primitives and components for [Yew] apps, built so
components keep their size and don't jitter as their content changes.

Each component comes with its Sass under `style/`, and its variants are
modifier classes set by props.

```toml
[dependencies]
plinth = { path = "../plinth" }
```

## Including the styles

plinth compiles `style/` with its defaults when the crate builds (with
[grass], through `plinth-build`), so an app needs no Sass step of its own.
An app without styles of its own mounts `<plinth::Styles/>` once at its
root:

```rust
html! {
    <>
        <plinth::Styles/>
        <App/>
    </>
}
```

It adds the stylesheet to `<head>` as `<style id="plinth-styles">`, once,
however many times it is mounted. The CSS is also `plinth::STYLESHEET`, for
an app that serves or inlines it itself. Its classes are constants in
`plinth::class`, so a misspelt class does not compile.

Every class plinth styles starts with `plinth-`, modifiers included
(`.plinth-stack.plinth-tight`), so none collides with an app's classes. A
constant is named after its class without the prefix, as an app's are:
`.plinth-page-head-title` is `plinth::class::PAGE_HEAD_TITLE`,
`.plinth-tight` is `plinth::class::TIGHT`.

## Adopting plinth with styles of your own

An app with its own Sass builds it with `plinth-build` in its `build.rs`,
beside plinth's, into **one stylesheet**: plinth's CSS first, unchanged,
then the app's. The app's classes become constants as plinth's are. Step by
step:

1. **Dependencies.** In the app's `Cargo.toml`:

   ```toml
   [dependencies]
   plinth = { path = "../plinth" }

   [build-dependencies]
   plinth-build = { path = "../plinth/plinth-build" }
   ```

2. **Styles.** Put the app's Sass in a directory, say `style/`, with an
   `_index.scss` (or `index.scss`) that `@use`s its partials. Load plinth's
   tokens and mixins as `plinth`:

   ```scss
   @use "plinth" as p;

   .army-sidebar {
       padding: var(--space-4);

       @include p.below("phone") {
           padding: var(--space-2);
       }
   }

   // plinth's classes are stable, so the app may refine them.
   .plinth-card.army-sidebar {
       --radius-lg: 12px;
   }
   ```

   `"plinth"` is served from the sources embedded in `plinth-build` (no path
   into plinth's checkout), so they always match the plinth-build version.
   It emits no second copy of plinth's CSS: plinth is compiled first in the
   same build. plinth's Sass cannot be configured from here (`@use "plinth"
   with (...)` fails); retune it through the custom properties (see
   [Theming](#theming)).

   Every class the app's Sass defines that is not plinth's must start with
   the app's prefix (`army-`); one that does not is a build error, so the
   app's classes never collide with plinth's.

3. **`build.rs`**, at the package root:

   ```rust
   fn main() {
       plinth_build::Config::new()
           .local("style/")
           .prefix("army-")
           .compile();
   }
   ```

   It writes `style.css` and `classes.rs` to `OUT_DIR`, and reruns when
   `build.rs` or anything under `style/` changes (partials and new files
   included).

4. **The class constants.** Somewhere in the crate (`src/main.rs` or
   `src/lib.rs`):

   ```rust
   mod class {
       include!(concat!(env!("OUT_DIR"), "/classes.rs"));
   }
   ```

   `classes.rs` holds only the app's classes, named without the prefix:
   `.army-sidebar` is `class::SIDEBAR`. plinth's stay `plinth::class::...`.
   Every plinth component takes a `class` prop, so pass the constants there:

   ```rust
   html! { <Stack class={class::SIDEBAR}>{ children }</Stack> }
   ```

   A class the Sass does not define, or a misspelt one, does not compile.

5. **The stylesheet.** Include it and mount it once at the root, with
   `Styles`'s `css`:

   ```rust
   const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

   html! {
       <>
           <plinth::Styles css={STYLESHEET} />
           <App/>
       </>
   }
   ```

6. **No Sass elsewhere.** Remove the app's own Sass step (trunk's
   `<link data-trunk rel="scss">` and `[tools] sass`); `index.html` links no
   stylesheet.

7. **A guard against dead classes.** A class the Sass defines and no Rust
   writes is dead weight. `plinth_build::Usage` finds them in a test, so an
   app needs no guard of its own (`tests/style_classes.rs`):

   ```rust
   use plinth_build::{Config, Usage};

   #[test]
   fn every_styled_class_is_written() {
       Usage::new()
           .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
           .dynamic(["army-tone-*"]) // built at runtime; a trailing `*` is a prefix
           .foreign(["army-fc-row"]) // written by a dependency
           .assert_clean(&Config::new().local("style/").prefix("army-"));
   }
   ```

   Add `plinth-build` to `[dev-dependencies]`. A class counts as written when
   a source names its constant (`class::SIDEBAR`) or the class in a string
   literal; comments and code after a file's first `#[cfg(test)]` do not
   count. The test fails on a dead class and on an allowed pattern that
   matches no class, and prints the class, single-use and dead counts.
   `STYLE_INVENTORY=PATH` also writes `class<TAB>uses` lines to `PATH`.
   It is a test, not a build warning: a warning scrolls past in a build
   log, a test fails where the project already looks. plinth runs it on
   itself and on the showcase.

**One stylesheet per app.** Both `Styles` fill the same element,
`<style id="plinth-styles">` (`plinth::STYLE_ID`): with `css` it sets the
element's text to the app's stylesheet, which holds plinth's; without it, it
adds plinth's only when the element is missing. So whichever mounts first
(the app's, or a `<plinth::Styles/>` inside a component), the page ends up
with the app's stylesheet, once. Never link or inject plinth's CSS another
way beside it.

**Configuring plinth's defaults.** Local Sass cannot `@use "plinth" with
(...)`, since plinth compiles first. Pass the values to the build instead,
before `compile()`; they are plinth's `!default` tokens (see Configuration),
and only values change, never classes:

```rust
plinth_build::Config::new()
    .local("style")
    .prefix("army-")
    .with("text-md", "1.0625rem")
    .with("$radius-lg", "12px")
    .compile();
```

The `$` is optional and the value is any Sass expression. Without `.local`,
the same call configures plinth's own stylesheet.

The showcase (`showcase/build.rs`, `showcase/style/`) is set up this way.

## Theming

The app supplies seven colour tokens: `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`. plinth derives six more
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

The status hues are mixed toward `--text`, so they keep their contrast on a
light and a dark background alike. The defaults are computed on `:root`: an
app that sets its colours on a narrower element (a dark panel) sets these
there too. `--focus-ring-color` (default `--accent`) colours the focus ring.

plinth's own scales are custom
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
`$card-padding-phone`, `$toolbar-gap`, `$visibility-breakpoint`) are Sass-only: an app built with
`plinth-build` sets them with `Config::with`.

## Configuration

An app built with `plinth-build` sets these with `Config::with` (see
Adopting plinth); an app that compiles plinth's Sass itself uses `with (...)`.
plinth's values are defaults. Every token is a `!default` variable of
`style/_tokens.scss`, emitted as its custom property on `:root`, so a project
sets its own with `with (...)`, either through the index:

```scss
@use "../plinth/style" with ($text-md: 1.0625rem, $radius-lg: 12px);
@use "../plinth/style/tokens";   // tokens and mixins, as configured
```

or on the tokens module first, then the rules:

```scss
@use "../plinth/style/tokens" with ($text-md: 1.0625rem);
@use "../plinth/style";
```

Sass configures a module only on its first load, so `with` goes on whichever
of the two comes first. The rules keep reading the custom properties, so a
page can still override them at runtime.

| Variable | Default |
| --- | --- |
| `$text-3xs`, `$text-2xs`, `$text-xs`, `$text-sm` | `0.625rem`, `0.6875rem`, `0.75rem`, `0.8125rem` |
| `$text-md`, `$text-base`, `$text-heading`, `$text-lg` | `0.875rem`, `0.9375rem`, `1.125rem`, `1.25rem` |
| `$text-title`, `$text-xl`, `$text-2xl` | `1.4rem`, `1.5rem`, `1.75rem` |
| `$space-0`, `$space-px`, `$space-1`, `$space-2`, `$space-3` | `0.125rem`, `1px`, `0.25rem`, `0.5rem`, `0.75rem` |
| `$space-4`, `$space-5`, `$space-6`, `$space-7`, `$space-8`, `$space-9` | `1rem`, `1.25rem`, `1.5rem`, `2rem`, `2.5rem`, `3rem` |
| `$radius-xs`, `$radius-sm`, `$radius-md`, `$radius-lg`, `$radius-pill` | `4px`, `8px`, `10px`, `16px`, `999px` |
| `$nest-inset` | `var(--space-1)` |
| `$control-height`, `$control-height-small` | `2.25rem`, `1.75rem` |
| `$mark-height`, `$touch-height` | `1.5rem`, `2.75rem` |
| `$glyph-size` | `1em` |
| `$filter-width` | `10.5rem` |
| `$page-workspace`, `$page-reading`, `$page-wide`, `$page-gutter` | `64rem`, `40rem`, `90rem`, `var(--space-8)` |
| `$card-padding` | `var(--space-4)` |
| `$card-padding-phone` | `null` (a card keeps `$card-padding` on a phone) |
| `$toolbar-gap` | `var(--space-1)` |
| `$focus-ring-color`, `$focus-ring-width`, `$focus-ring-offset` | `var(--accent)`, `2px`, `2px` |
| `$layer-sticky`, `$layer-overlay`, `$layer-dialog`, `$layer-toast`, `$layer-tooltip` | `10`, `100`, `200`, `300`, `400` |
| `$danger`, `$success`, `$warning`, `$on-accent`, `$surface-raised`, `$scrim` | see [Theming](#theming) |
| `$visibility-breakpoint` | `"phone"` (where `phone-only` and `desktop-only` switch) |
| `$breakpoints` | `()`, merged over plinth's: `narrow` 30rem, `small` 36rem, `phone` 40rem, `medium` 48rem, `compact` 52rem, `regular` 56rem, `large` 60rem, `tablet` 64rem, `wide` 84rem, `broad` 90rem, `desktop` 100rem |

`$breakpoints` moves or adds named breakpoints without restating the rest:
`$breakpoints: ("phone": 42rem)`.

## Heads

`PageHead` and `SectionHead` take one set of props (`HeadProps`):

- `title` is Html; `icon` goes before its text in the heading, `subtitle` is a
  muted line under the row, `crumbs` (a list of links) stack above it, and
  `back` is the inline "back / title" crumb. Each is its own prop.
- `help` follows the heading as its sibling, in `.plinth-head-heading` with
  the `suffix`, so the heading's text and test id exclude the help button.
- `SectionHead` takes the modifiers `plinth::class::LABEL` (small-caps label),
  `LIST` and `LIST` + `NESTED` (smaller titles) in `class`; `HEAD_END` pushes a
  child to the row's end.

## Icons

plinth's own chrome (a dialog's close, a select's chevron, a busy spinner)
draws its icons from an `IconSet`: one `Html` per `Glyph` (`Close`,
`ChevronDown`, `ChevronUp`, `ChevronRight`, `Check`, `Plus`, `Minus`,
`Spinner`, `Grip`, `Ellipsis`, `Search`). It is the one place plinth uses a
context for something other than a runtime setting, and it carries icons
only: classes never go through it.

Without a provider plinth uses `IconSet::default()`, a built-in set of inline
svg, so it renders standalone. An app supplies its own once at its root,
mapping its icon component; a set may override only some glyphs:

```rust
let set = IconSet {
    close: html! { <MyIcon name="x" /> },
    chevron_down: html! { <MyIcon name="caret-down" /> },
    ..IconSet::default()
};

html! {
    <ContextProvider<IconSet> context={set}>
        <App/>
    </ContextProvider<IconSet>>
}
```

`<Icon glyph={Glyph::Close} class=.../>` renders a glyph; `use_icon(glyph)` and
`use_icons()` return the icon(s) for a component of your own, and `icon(class,
glyph)` gives the box's classes (`plinth::class::GLYPH`, plus `SPIN` for
`Spinner`).

Size guarantees: the glyph sits in a `.plinth-glyph` box of `--glyph-size`
(`1em`, so it follows the text) square, `overflow: hidden`, and what fills it
is stretched to it. A swapped set, whatever its icons' own width and height,
cannot change a control's size. Icons should draw in `currentColor`. The
spinner's box turns (slower under `prefers-reduced-motion`); the set's
spinner need not animate itself.

## Against jitter: mixins and utilities

The shared rules every component builds on, so nothing changes size as its
state or content changes. Each is a mixin (`@use "plinth" as p;` then
`@include p.numeric;`) and, where an element needs it directly, a utility
class. Utilities load after the components, so one wins over a component's
rule of the same weight; none uses `!important`.

| Mixin | Utility (`plinth::class::`) | What it guarantees |
| --- | --- | --- |
| `focus-ring` | `.plinth-focus-ring` (`FOCUS_RING`) | A `:focus-visible` outline from the `--focus-ring-*` tokens: never a border, so focus adds nothing to the box. |
| `reserved-border($color: transparent, $width: var(--space-px))` | | The border is always there; a state changes only its colour. |
| `numeric` | `.plinth-numeric` (`NUMERIC`) | Tabular figures: a count, readout, pager or clock keeps its width as it changes. |
| `scroll-area` | `.plinth-scroll-area` (`SCROLL_AREA`) | Scrolls, with `scrollbar-gutter: stable`: content does not jump when it starts to scroll. Contains overscroll and may shrink in a flex column. |
| `truncate` | `.plinth-truncate` (`TRUNCATE`) | One line, cut with an ellipsis. |
| `clamp($lines)` | `.plinth-clamp-2`, `-3`, `-4` (`CLAMP_2`...) | At most that many lines, the last cut with an ellipsis. |
| `visually-hidden` | `.plinth-visually-hidden` (`VISUALLY_HIDDEN`) | Read by a screen reader, takes no room (a hidden h1). |
| `phone-only`, `desktop-only` | `.plinth-phone-only`, `.plinth-desktop-only` | Shown only below `$visibility-breakpoint`, or only from it up. |
| `current($by: background)` | | Marks `[aria-current]` (not `"false"`) and `[aria-selected="true"]` by `background` (`--accent` under `--on-accent`), `color` or `underline`; never by weight, so the item keeps its width. |

Layers: `--layer-sticky`, `--layer-overlay`, `--layer-dialog`,
`--layer-toast` and `--layer-tooltip` are the one `z-index` scale for what
floats, lowest first.

## Showcase

`showcase/` is an interactive gallery of every component: a page for
each, with controls for its props and its content, shown in a narrow
frame (drag its corner to widen it) beside a wide one. Run it with
[trunk]:

```sh
cd showcase
trunk serve --port 8095 --open
```

Pick a free port. A new component lands with its showcase page: add it to
`PAGES` in `showcase/src/pages/mod.rs`, listing the names it covers, or
`cargo test --workspace` fails.

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
one control at a time, toggles each checkbox, then focuses each control
in the frames and clicks each button there, measuring every declared
part in both frames after each state. Its `<pre>` reads `probe: stable`
when no declared box moved or resized by more than half a pixel, and
otherwise lists each change by state, frame and part. The panel's
`data-probe` attribute carries the verdict (`idle`, `running`, `stable`,
`unstable`), and `?probe` before the page's `#id` runs it on load, so a
browser check reads the result as text. Hover cannot be scripted, so it
stays a manual check.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --workspace -- -D warnings
cargo test --workspace
(cd showcase && trunk build)
```

[Yew]: https://yew.rs
[grass]: https://crates.io/crates/grass
[trunk]: https://trunkrs.dev
