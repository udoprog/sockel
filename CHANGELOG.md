# Changelog

All notable changes to sockel and sockel-build. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- `Report::css_bytes`, the compiled stylesheet's plain size, and
  `Config::css_budget(bytes)`, which fails any build over that size with the
  new `Error::OverBudget { size, budget }`. No budget, no change.
- `MenuItem` passes on `pressed` (`aria-pressed`), `busy` (`aria-busy`),
  `haspopup` (`aria-haspopup`, over `dialog`'s `"dialog"`), `data` (more
  `data-*` attributes, as Button's) and `node_ref`. Unset, they render nothing.
- `SelectionBar` takes `onkeydown`, `count_testid` (the count span's test id)
  and `idle: Option<bool>`, which overrides `count == 0` when the count is a
  phrase. Unset, the markup is as before.
- `Switch` and `Checkbox` take `title`, a native hover hint (on the switch's
  button, on the checkbox's label), for apps without `DataTips`. Unset, it
  renders nothing.
- `Config::prune(usage)`: leaves out sockel's rules for classes the app never
  reaches (neither its sources nor a sockel component they use write them),
  and `Report::pruned`, `pruned_rules` and `pruned_bytes`. Off by default, the
  stylesheet unchanged. Measured: territory 107.6 KB to 104.2 KB (15.2 to
  15.0 KB gzip), characterize 370.0 KB to 364.4 KB (49.9 to 49.4 KB gzip).

### Breaking

- `sockel_build::Error` (not `#[non_exhaustive]`) has the new variant
  `OverBudget`; an exhaustive `match` on it needs an arm.
- New public prop fields break struct literals of `MenuItemProps` (`pressed`,
  `busy`, `haspopup`, `data`, `node_ref`) and `SelectionBarProps`
  (`count_testid`, `idle`, `onkeydown`), `SwitchProps` and `CheckboxProps`
  (`title`). End the literal with
  `..Default::default()`, or build the props with `yew::props!`.

### Changed

- sockel-build's docs: the short-prefix advice gives the measured cost. A
  long prefix is a large share of the raw sheet (`characterize-`, about
  34 KB of 369 KB) but gzip removes almost all of it (0.2–2% of the
  compressed sheet), so it matters for raw size, little on the wire.
- A pill button's text on hover is 45% tone and 55% `--text` (the new
  `$pill-button-hover-ink`), so it reads at 4.5:1 or more on the deeper hover
  tint too.

## [0.2.2] - 2026-11-19

### Added

- An optional app colour, `--selection`: the ground of a `NavList`'s current
  item and a menu's current page, under their accent text. An app that sets
  none keeps today's accent tint.
- `Table` takes `stacked`: with `cards`, each cell after the first is a
  field, its label over its value, two to a line (`.sockel-stacked` on the
  table).
- Every props struct except `ImageProps` implements `Default` (its `#[prop_or]`
  values, otherwise the empty value), so apps can write `..Default::default()`
  and later new props stay non-breaking. The crate docs recommend `html!` and
  `yew::props!` for building props.
- Finer `$omit` parts, so an app leaves out more of a component it renders:
  `badges.pill`, `badges.dot`, `badges.count`, `badges.status`, `tabs.segmented`,
  `tabs.fill`, `tabs.stacked`, `tabs.nav`, `table.sort-header`, `shell.bottom-nav`,
  `property-list.inline-edit`, `rows.pick-row`, `rows.selection-bar` and
  `rows.action-row`. Omitting all of them takes the compressed stylesheet from
  115.8 KB to 102.5 KB; the default output is unchanged.
- A size budget: sockel-build's test `the_full_stylesheet_is_within_its_budget`
  fails when the full stylesheet, compressed, passes 121,500 bytes.

### Changed

- A `Pill`'s text is its tone mixed 70% toward `--text` (the new `$pill-ink`),
  so every tone reads at 4.5:1 or more on the pill's tint; its dot keeps the
  tone itself.

### Fixed

- On a phone, a `Choice`'s options are at least the group's height (the group
  grows by its inset) and a label wraps instead of being cut off.
- A `.sockel-small` button is at least as wide as it is tall, through the new
  `$button-small-min-width` (`var(--button-height)`; `auto` restores the old
  width), so a short label stays a tap target.
- A control wearing `.sockel-split-view-back` shows a back arrow before its
  label.

## [0.2.1] - 2026-11-19

### Changed

- The crate docs and READMEs point to the [showcase](https://udoprog.github.io/sockel/)
  instead of describing each component, and every link in them is absolute.

### Fixed

- The getting-started and Choosing features blocks require sockel and
  sockel-build `0.2`, not `0.1`.

## [0.2.0] - 2026-11-19

### Breaking

New public prop fields break struct literals of these props (none derives
`Default`). Add the new fields with their defaults (`false`, `None`,
`Callback::noop()`, `ButtonVariant::Quiet` for `close_variant`), or build the
props with `yew::props!`, which fills them in:

- `ButtonProps`: `selected`, `link`, `onmouseenter`, `onmouseleave`,
  `onpointerdown`, `onmousedown`, `onauxclick`.
- `MenuProps`: `chevron`.
- `DialogProps`: `close_variant`, `onkeydown`.
- `ErrorBoxProps`: `id`.
- `ReorderItemProps`: `disabled`.
- `CardProps`: `gap` (`None`).
- `SearchSelectProps`: `more`, `on_more`, `more_label`, `on_create`,
  `create_label`, `free_text`, `on_text`, `text`.
- `ReorderProps`: `on_drop_page_picked`.
- `PickerProps`: `busy`, `busy_label`, `filter`, `sheet`, `close_label`;
  `ChoiceListProps`: `busy`, `busy_label`, `filter`; `ChoicePopupProps`:
  `sheet`, `title`, `close_label`. `PickerOption` gains the public field
  `trailing` (`None`), which breaks a struct literal of it; build options
  with `PickerOption::new`.
- `TabsProps`, `LinkTabsProps`: `stacked`.
- `PagerProps`: `counts`.

`ReorderText` is `#[non_exhaustive]` and has new variants (`LiftedCarrying`,
`MovingBeside`, `DroppedBeside`, `DroppedCarrying`, `DroppedOnPage`), so a
`text` callback that matches it no longer compiles: add a last arm falling
back to the app's own text: `let fallback = sockel::use_reorder_text();` in the
component, then `_ => fallback.emit(t)` (`sockel::reorder_text(&t)` is the
English only).

`Reorder`: a picked row carrying others is no longer let go on a page
(`on_drop_page`, which reported only the lifted row); set
`on_drop_page_picked` to take that drop, with every carried row.

sockel-build's `Report` has the fields `unprefixed` and `split`, and is
`#[non_exhaustive]` so later fields are not breaking. Code that built a
`Report` or destructured it without `..` no longer compiles; read its fields
instead.

### Added

- A configurable test-id attribute: set `SOCKEL_TEST_ID_ATTR` at build time
  (`.cargo/config.toml` `[env]`) and every component writes its `testid` to
  that attribute instead of `data-testid`, the default. Unset, an element
  has the same attributes as before, the test id now last among them (a
  snapshot comparing attribute order sees it move).
- sockel-build: `Report::omitted`, the sockel classes the build leaves out
  (by `$omit` or a feature) that a source writes, as a literal or
  `sockel::class::NAME`, unless the local styles define them.
- `use_reorder_text()` (hook) returns a `Callback<ReorderText, String>` saying
  any `ReorderText` in the app's language, the fallback arm of a `text`
  override. `reorder_text` stays the English catalog text.
- `Tabs`, `SubTabs` and `LinkTabs` take `stacked`: a phone's bottom bar, each
  tab's icon over its one-line, truncated label, the tabs of equal width
  across the container and none changing size when selected. Off by default;
  tokens `$tab-bar-icon-size` and `$tab-bar-gap`.
- `Reorder` and `ReorderTree` announce the fixed row a row lands next to,
  `Reorder` how many picked rows a lifted one carries and a drop on a page
  (`on_drop_page`). New catalog messages: `sockel-reorder-lifted-carrying`,
  `-moving-beside`, `-dropped-beside`, `-dropped-carrying`, `-dropped-page`,
  `-dropped-page-carrying`, and the `-in` forms of the first four.
- `Reorder`'s `on_drop_page_picked`: picked rows lifted together and let go on
  a page, reported as `(rows, value)`.
- Button: `selected` (the pressed look for a current choice, without
  `aria-pressed`), `link` (a text link look) and the mouse and pointer events
  `onmouseenter`, `onmouseleave`, `onpointerdown`, `onmousedown`, `onauxclick`.
- Menu: `chevron` ends an icon opener in a chevron, as Button's does.
- Dialog: `close_variant` (the close button's variant, `Quiet` by default) and
  `onkeydown`.
- ErrorBox: `id`.
- sockel-build: `Usage::one_file_per_component()` also fails on a class with
  a top-level rule in more than one local Sass file (`Report::split`), the
  check characterize kept beside its guard.
- sockel-build: `Usage::unprefixed([...])` allows words a class position
  writes on purpose (see Changed).
- sockel-build's docs: a short prefix keeps the stylesheet smaller (each
  local selector carries it; `characterize-` cost about 47 KB).
- `ReorderItem`'s `disabled`: a row that cannot be lifted (its handle is
  disabled, it is not carried with picked rows, no body drag), counted and
  landed beside as any other, in `Reorder` and `ReorderTree`.
- `ReorderTree` honours a `ReorderItem`'s `fixed` (no handle, stays put, not
  counted in the positions announced) and `lifted_only` (shown only while a
  row is lifted), as `Reorder` does.
- sockel-build's docs show one `Config` shared by `build.rs` and the
  `Usage` guard: a `style.rs` that both `include!`, the guard adding
  `.style(env!("SOCKEL_STYLE"))`.
- `--layer-host` (`$layer-host`): the overlay host's z-index, `auto` by
  default, so the host is no stacking context and nothing changes. An app
  whose own overlays sit at z-index 1000 or more sets it above them (say
  `--layer-host: 1100`), and every sockel popover, menu, list, dialog, toast
  and tooltip shows over the app's, in their own order.
- SearchSelect, all off by default: `more` and `on_more` (a more-results
  row ending the list, for paged server results; `more_label`),
  `on_create` (a new-value row offering what is typed while no label is it,
  `offers_new`; `create_label`), and `free_text` with `on_text` and `text`
  (Enter without an active option, Tab or a press outside give the typed
  text). New classes `.sockel-search-select-create`, `-typed`, `-more` and
  `search_select_row`; phrases `sockel-search-select-new`, `-more-results`.
- Picker and ChoiceList, all off by default: `busy` for a server search
  (`on_search`) still on its way (a spinner in the search field,
  `aria-busy` on the list, `busy_label` while nothing is listed), `filter`
  (the app's control under the search field, `.sockel-choice-list-filter`),
  `PickerOption::trailing` (muted row-end content before the badge,
  `.sockel-choice-list-trailing`), and `sheet` with `close_label` (a sheet
  from the bottom on a phone, headed by the label, as ChoicePopup's new
  `sheet`, `title` and `close_label`: `.sockel-picker-popup.sockel-sheet`,
  `-popup-head`, `-popup-title`, `.sockel-picker-scrim`). Phrases
  `sockel-picker-searching`, `sockel-picker-close`.
- Layout gaps: `--page-title-size` (a `PageHead` title's size, `$page-title-size`),
  `--page-head-crumbs-align` (a crumb head's back link, `/` and title; `baseline`),
  `$card-button-fill` (a secondary `Button`'s fill on a `Card`; `var(--surface)`),
  `Card`'s `gap` prop (the card as a `Stack`) and `.sockel-in-row` (`class::IN_ROW`):
  a `TextInput` or its box taking the room left in a row, no `sockel-grow` wrapper.
- `cluster_with(gap, justify, align, wrap, inline, lines, flush)` is exported,
  beside `stack_with`.
- Pager: `counts: Option<PagerCounts>` (`PagerCounts::new(total, size)`)
  makes a summary pager whose two lines come from the catalog,
  `sockel-pager-counts-shown` ("51–100 of 312") and
  `sockel-pager-counts-position` ("Page 2 of 7"); a `summary` given beside it
  wins.
- `MenuPopup`: a menu's list for an opener the app renders itself, open while
  `open`, placed beside `anchor` or at a viewport point `at`, closed through
  `on_dismiss`; it holds the same items, headings, separators and notes as a
  `Menu`.
- Settings for a menu's parts, `null` by default so nothing changes:
  `$menu-separator-margin`, `$menu-separator-edge`, `$menu-heading-padding`,
  `-text`, `-ink`, `-weight`, `$menu-note-padding`, `-text`, `-ink`.
- `$omit` parts `pager.ranged` (a ranged pager's range and group),
  `pager.compact` (`.sockel-pager-at`), `pager.drop` (a page button as a drop
  target), `select.filter` (a Select's filter field and no-match text),
  `select.sheet` (a Select's phone sheet), `disclosure.heading` (a
  Disclosure's `level` heading) and `disclosure.detail` (its `summary`).
- Opt-in layout modifiers and tokens: `.sockel-disclosure.sockel-wrap` (`class::WRAP`; a long
  title wraps and the row grows), `$empty-state-compact-actions-wrap`,
  `$row-list-item-edge`, `.sockel-least` (`class::LEAST`; a `Button`'s or
  `TextInput`'s size as a least height) and `.sockel-shrink` (`class::SHRINK`;
  `min-width: 0` for a column in a row).
- `.sockel-reveal-host` and `.sockel-reveal` (`class::REVEAL_HOST`,
  `class::REVEAL`): a part of any container that stays invisible and inert
  until the container is hovered or has focus inside, keeping its room (an
  overlay ActionRow lies over the row instead); shown where nothing hovers.
- `.sockel-button.sockel-menu-row` (`class::MENU_ROW`): a button as a
  start-aligned row of a dropdown, spanning its container.
- `$head-trail-separator` (`"\203A"`), `$head-trail-mark` (unset) and
  `$head-trail-mark-size` (`1em`): the mark after each crumb of a head's
  trail, a string or a mask image drawn in the muted ink.

### Changed

- An `$omit` naming a part its feature already leaves out is an error, in
  `sockel-build` and in the Sass given `$features`. `Usage::assert_clean`
  (and `Report::is_clean`) now fail on a written class the build omits, so a
  build that passed may report `omitted: .sockel-...`: keep the part, or
  stop writing the class.
- TextInput's `autofocus` focuses the input on mount. Inside a dialog or
  popover (`[role=dialog]`) it stays the `data-autofocus` mark they focus as
  they open, as before; elsewhere it used to do nothing.
- A `Button` reads its height from `--button-height` (its size's), so
  `.sockel-least` can turn it into `min-height`; the height is the same
  without it. A Disclosure's row height and title wrap, a compact
  EmptyState's actions wrap and a row's top edge each read a token or
  custom property whose default is today's value, so output is unchanged.
- A `PageHead` title has a size of its own, `--page-title-size`; unset (the
  default) it keeps the browser's heading size (`revert`), so output is the
  same until an app sets `$page-title-size`. A secondary button's fill reads
  `--button-surface` (set on `.sockel-card` and by `p.card`,
  `$card-button-fill`), the same colour by default.
- ConfirmPopover's body has a scrollbar gutter only where it scrolls: it
  never does, so it no longer shows a 10px gutter on its right. Output
  changes for every confirm; a new `$confirm-scroll-gutter` (`auto`) sets it
  and `stable` keeps the popover's. `$popover-scroll-gutter` stays `stable`.
- Smaller CSS for an app that renders only some toggles, or only Toast or
  only SaveBar: new features `checkbox` (with `OptionCard`), `switch`,
  `choice`, `chip`, `toasts` (`Toast`, `ToastHost`) and `save-bar` each load
  only their own partial (`style/_checkbox.scss`, `_switch.scss`,
  `_choice.scss`, `_chip.scss`, `_toasts.scss`, `_save-bar.scss`). `toggles`
  and `toast` stay and enable all of theirs, `@use "sockel/toggles"` and
  `"sockel/toast"` load the same rules as before, and `_index.scss` (so
  the full sheet) compiles to byte-identical CSS. No `$omit` key changed.
- sockel-build: `Config::with(token, "null")` is a build error naming the
  token. Sass treats a null configuration as unset, so the call always left
  the default in place; a build that passed it now fails. Drop the call, leave
  the part out with `$omit`, or give a real value. The error is an
  `Error::Io` of kind `InvalidInput`, so `Error` keeps its variants.
- sockel-build: `Usage` also fails on a class word written without its
  prefix (`Report::unprefixed`): a literal word in a class position (a
  `class` attribute, field or binding, `classes!`, or a parameter of the
  sources' own `fn` named for a class, through `Some(..)`, tuples and
  `match` arms) that is a styled class once prefixed, as `Some("button")`
  beside `.app-button`. A guard that passed can fail. Write the constant
  (`Some(class::BUTTON)`), or, for a word that is not a class there, allow
  it with `Usage::unprefixed(["button"])`; a word a `dynamic` pattern
  covers once prefixed is not reported.
- The pager, select, listbox, disclosure, drop, menu and toast partials fold
  their repeated rules: rules with the same declarations share one selector
  list (a pager's button and gap, a menu's and a listbox's mark and icon, a
  toast's icon, action and close, a drop zone's over, refused and error), and
  duplicate declarations are gone (a listbox's second `max-width` rule, a
  `min-width: 0` beside the truncate mixin). Same rendered styles; the full
  sheet is 651 bytes smaller (expanded), and no class changed.

### Fixed

- `use_placement` measures a popover's height with its border, so a
  bordered surface against a viewport edge keeps `EDGE` clear instead of
  ending 2px past it.
- `Reorder` with `body_drag`: a press let go outside the window before it
  became a drag no longer leaves a drag that the next pointer move over the
  row would start without a button held.
- Escape that closes a popover, menu, select, search select, color select,
  filter picker or picker list is now handled on its face and its surface
  and has its default prevented. The app's own keydown handlers on their
  ancestors (yew runs them before the document hears the key) can check
  `defaultPrevented` and skip it, so a form that cancels on Escape no
  longer cancels while closing a list inside it. Apps that place their own
  popover with `use_popover` keep the document's handling, where Escape is
  now also default-prevented.

## [0.1.0] - 2026-11-19

First release. It covers what changed for the apps that adopted plinth
(characterize, army, territory, track, livemix). Commits in parentheses.

### Renamed: plinth is sockel (153da740)

Mechanical: `s/plinth/sockel/`, `s/Plinth/Sockel/`, `s/PLINTH/SOCKEL/`:

- Cargo: `plinth` -> `sockel`, `plinth-build` -> `sockel-build` (path
  `../sockel/sockel-build`); `use sockel::...`, `sockel_build::...`.
- Rust: `plinth_classes` -> `sockel_classes`; any `plinth-*` name -> `sockel-*`.
- Sass: `@use "plinth"` -> `@use "sockel"`.
- Classes: `.plinth-*` -> `.sockel-*`, in markup, Sass and test selectors.
- Custom properties: `--plinth-*` -> `--sockel-*`.
- Build env: `DEP_PLINTH_FEATURES`, `DEP_PLINTH_STYLE`, `PLINTH_STYLE` ->
  `DEP_SOCKEL_FEATURES`, `DEP_SOCKEL_STYLE`, `SOCKEL_STYLE`; `links = "sockel"`.
- Phrase ids: `plinth-*` in an app's `.ftl` -> `sockel-*`.

### Removed tokens

- `$button-icon-width`: use Button's `wide` per button (95656588).
- `$empty-state-ink`, `$empty-state-compact-ink`: a hint reads muted (c00d95ec).
- `$pager-range-ink` (dc6dec9c).

No class was renamed or removed beyond the plinth prefix.

### Label props are `Option<AttrValue>` (d04e1206, 8d6eaa96)

`None` is the catalog's word in the app's language. `html!` uses still
compile; struct literals need `Some(..)`, and reads of the fields change.

- Confirm, ConfirmPopover, ConfirmButton, ConfirmInline: `cancel_label`,
  `question`, `no_label`, `yes_label`; `ConfirmRequest`'s `accept`, `cancel_label`.
- Dialog, Popover, Notice, Toast: `close_label`. Select: also `filter_label`, `no_match`.
- Loadable: `label`, `empty_title`, `error`. ErrorBox: `retry_label`, `dismiss_label`.
- SaveBar: `label`, `save_label`, `discard_label`. DropZone: `label`, `reject`, `too_large`.
- RowActions `more_label`, SplitButton `menu_label`, SearchField `clear_label`,
  FilterPicker `all`, Seam `label`, Stepper `less_label`, `more_label`.
- Pager: `label`, `first_label`, `previous_label`, `next_label`, `last_label`, `page_label`.
- ChoiceList, Picker, MultiPicker, PickerMenu, PickerDialog: `search_label`,
  `empty_label`. SearchSelect: `loading_label`, `empty_label`.
- SecretInput: `show_label`, `hide_label`, `copy_label`, `clear_label`.

Apps provide phrasebook 0.2 and add `sockel::phrasebookcatalog01::L10N` after
their own catalog.

### New props fields (struct literals only)

Props built with `html!` are unaffected. A struct literal needs the new fields:

- characterize: `ButtonProps`, `MenuItemProps`, `MenuCheckProps`,
  `MenuRadioProps`, `MenuPartProps`, `RowActionsProps`, `ChoiceListProps`,
  `ChoicePopupProps`, `PickerMenuProps`.
- army: `IconButtonProps`, `DialogProps`, `ToastProps`, `PopoverProps`,
  `SelectProps`, `MenuProps`, `MenuItemProps`, `StepperProps`, `PagerProps`,
  `DropZoneProps`, `SearchFieldProps`.
- territory: `TextInputProps` (`maxlength`, `list`, `inputmode`,
  `enterkeyhint`), `DisclosureProps` (`level`).
- livemix: `PanelProps` (`small`), `DialogProps` (`head_actions`),
  `PopoverProps`, `MenuProps`, `ContextMenuProps` (`within`), `PopoverProps` (`fit`).
- `CheckboxProps`, `SwitchProps` (`tip`, 916bf3a1).
- `TextareaProps` (`node_ref`, `on_keydown`, `on_paste`, `on_drop`),
  `TextInputProps` (`autocomplete`, `spellcheck`), `ConfirmProps`,
  `ConfirmPopoverProps`, `ConfirmButtonProps` (`accept_disabled`) (211120b4).

### Added

- `OptionCard` (`toggles` feature): a radio or checkbox drawn as a card with a
  label and a description line (211120b4).
- `InputKind::Number` and `InputKind::DateTime` (`datetime-local`); a `match`
  over `InputKind` needs the new arms (211120b4).

### Behaviour changes

- Label heads have min-height 0; heads have margin 0 (94b28b7c).
- A popover's close refocuses its anchor only after focus or a press inside (c62aeac5).
- A fixed anchor inside a scroller is not clipped by it (9842c777).
- A popover closes once its anchor is out of sight (60609936); it ignores an
  Escape already handled (0aa9d72d); it is `role=dialog` by default (9a2685fa).
- Menus: Tab or a pick returns focus to the opener (4f5d68e5); the opener's
  Escape or Tab closes it (344b70ef); a fields-only menu focuses its first
  control (790896ce); popovers keep their side (b8bf9e9f).
- Link menu items no longer get the button base (3ec9ebd3).
- Stepper: parts min-height 0 (5c37e11a), width fit-content (8fc6c366), a step
  starts from the typed number (e8277b94); a width with decimals counts every
  decimal place, trailing zeros included (65762cf9).
- Select with `filter_from`: typing on the open face goes into the filter (6c1e087e).
- ChoiceList: no row is active under a search field (cd924f67).
- `.sockel-grow` and `.sockel-push-end` beat a one-class app rule (918170b1).
- Field faces and Button keep their height under an app's button min-height
  (2d1a31f9, 58dd5a97); an app dropping a blanket button rule sets
  `$button-part-*` instead (355023d9).
- `use_width` measures portal-drawn elements (62eb5e67).

### Build

- sockel-build compiles with sasso, not grass: CSS matches dart-sass, and
  `Error::Sass` carries `sasso::Error` (169bcc02). Apps' `Cargo.lock` loses grass.
- Apps build their stylesheet with sockel-build in `build.rs`, not trunk's
  Sass step; sockel's Sass is read from its package, not a checkout.
- An app may style a sockel class its `$omit` leaves out (031d46fc).
- `_inputs.scss`'s private `box` mixin is `base.input-box` (c0e8b6cc).
