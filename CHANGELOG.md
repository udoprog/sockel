# Changelog

All notable changes to sockel and sockel-build. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
