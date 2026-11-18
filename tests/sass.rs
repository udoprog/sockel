use std::path::{Path, PathBuf};
use std::process::Command;

const SASS: &str = ".cache/trunk/sass-1.104.1/sass";

fn sass() -> Option<PathBuf> {
    let path = Path::new(&std::env::var_os("HOME")?).join(SASS);
    if path.exists() {
        Some(path)
    } else {
        eprintln!("skipping: no sass at {}", path.display());
        None
    }
}

fn compile(sass: &Path, file: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    let output = Command::new(sass)
        .arg("--no-source-map")
        .arg(&file)
        .output()
        .expect("run sass");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf-8 css")
}

fn fails(sass: &Path, file: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    let output = Command::new(sass)
        .arg("--no-source-map")
        .arg(&file)
        .output()
        .expect("run sass");
    assert!(!output.status.success(), "{file:?} compiled");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn defaults_compile() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "style/_index.scss");
    assert!(css.contains("--text-md: 0.875rem;"));
    assert!(css.contains("--radius-lg: 16px;"));
    assert!(css.contains("@media (width < 40rem)"));
}

#[test]
fn configured_through_index() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "tests/fixtures/configured.scss");
    assert!(css.contains("--text-md: 1.0625rem;"));
    assert!(css.contains("--radius-lg: 12px;"));
    assert!(css.contains("padding: var(--space-4) var(--space-5);"));
    assert!(
        css.contains("@media (width < 42rem) {\n  .sockel-card {\n    padding: var(--space-3);")
    );
    assert!(css.contains("gap: var(--space-2);\n  min-width: 0;\n  overflow-x: auto;"));
    assert!(!css.contains("@media (width < 40rem)"));
}

#[test]
fn configured_through_tokens() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "tests/fixtures/configured_tokens.scss");
    assert!(css.contains("--text-md: 1.0625rem;"));
    assert!(css.contains("--radius-lg: 12px;"));
}

#[test]
fn dialog_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("--dialog-room: calc(100dvh - var(--dialog-top) - var(--space-4));"));
    let css = compile(&sass, "tests/fixtures/dialog_centred.scss");
    assert!(css.contains(".sockel-dialog-layer {\n  align-items: center;"));
    assert!(css.contains("--dialog-room: min(85vh, 100dvh - 2 * var(--space-4));"));
    assert!(
        css.contains(
            "padding: 1rem var(--dialog-pad) var(--dialog-pad);\n  scrollbar-gutter: auto;"
        )
    );
    assert!(css.contains("--section-title-size: 1.125rem;"));
    assert!(
        css.contains("border-bottom-color: transparent;\n    border-width: var(--space-px) 0 0;")
    );
    assert!(!defaults.contains("border-width: var(--space-px) 0 0;"));
}

#[test]
fn popover_and_confirm_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(
        defaults.contains("border-radius: var(--radius-md);\n  background: var(--surface-raised);")
    );
    assert!(defaults.contains("flex-direction: column;\n  gap: var(--space-3);\n  min-width: 0;"));
    let css = compile(&sass, "tests/fixtures/popover_settings.scss");
    assert!(css.contains("border-radius: 12px;\n  background: var(--surface-raised);"));
    assert!(css.contains("gap: 1rem;"));
    assert!(css.contains("scrollbar-gutter: stable;\n  scrollbar-gutter: auto;"));
    assert!(!defaults.contains("scrollbar-gutter: auto;\n  flex: 1 1 auto;"));
    assert!(
        css.contains("border-bottom-color: transparent;\n    border-width: var(--space-px) 0 0;")
    );
    assert!(css.contains(
        ".sockel-confirm-popover.sockel-compact {\n    --popover-max-width: 100vw;\n    width: max-content;"
    ));
}

#[test]
fn error_box_settings() {
    let Some(sass) = sass() else { return };
    let centred = "padding-block: calc((var(--control-height) - 1lh) / 2);";
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(centred));
    let css = compile(&sass, "tests/fixtures/error_box_settings.scss");
    assert!(css.contains("padding-block: calc((var(--control-height) - 1lh) / 2) 0;"));
    assert!(!css.contains(centred));
}

#[test]
fn notice_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("min-height: calc(var(--control-height-small) + 2 * var(--space-2)"));
    assert!(defaults.contains("line-height: 1.4;"));
    let css = compile(&sass, "tests/fixtures/notice_settings.scss");
    assert!(!css.contains("min-height: calc(var(--control-height-small) + 2 * var(--space-2)"));
    assert!(!css.contains("line-height: 1.4;"));
    assert!(css.contains("min-height: 0;"));
}

#[test]
fn drag_handle_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(
        defaults.contains(
            "width: var(--control-height-small);\n  height: var(--control-height-small);"
        )
    );
    assert!(defaults.contains("border-radius: var(--radius-xs);"));
    let css = compile(&sass, "tests/fixtures/drag_handle_settings.scss");
    assert!(css.contains("width: 32px;\n  height: 32px;"));
    assert!(css.contains("border-radius: 6px;"));
    assert!(css.contains("background: rgba(1, 2, 3, 0.08);"));
    assert!(css.contains("width: 44px;\n    height: 44px;"));
    assert!(!css.contains("width: var(--control-height-small)"));
}

#[test]
fn a_dialogs_scrim_reaches_over_the_kept_gutter() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(
        ".sockel-dialog-layer {\n  position: fixed;\n  inset: 0 calc(-1 * var(--scroll-gutter, 0px)) 0 0;"
    ));
}

#[test]
fn a_dialog_body_gap_spaces_its_paragraphs() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(!defaults.contains(".sockel-dialog-body > p"));
    let css = compile(&sass, "tests/fixtures/dialog_body_gap.scss");
    assert!(css.contains("gap: 1rem;"));
    assert!(css.contains(".sockel-dialog-body > p {\n  margin: 0;\n}"));
}

#[test]
fn a_native_select_gives_way_in_a_row() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(
        ".sockel-native-select {\n  width: max-content;\n  max-width: 100%;\n  flex-shrink: 1;"
    ));
}

#[test]
fn a_picker_gives_way_in_a_row() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(
        ".sockel-picker {\n  width: var(--menu-width-wide);\n  max-width: 100%;\n  flex-shrink: 1;"
    ));
}

#[test]
fn input_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("--input-edge: var(--accent);"));
    assert!(!defaults.contains("--input-height: 44px;"));
    assert!(defaults.contains(".sockel-native-select {\n  width: max-content;"));
    let css = compile(&sass, "tests/fixtures/input_settings.scss");
    assert!(css.contains("--input-edge: var(--border-strong);"));
    assert!(css.contains(":focus-within {\n  --input-edge: var(--accent);"));
    assert!(css.contains("--input-height: 44px;"));
    assert!(css.contains("padding-inline: 10px;"));
    assert!(css.contains("font-size: var(--text-sm);"));
    assert!(css.contains("outline-offset: -1px;"));
}

#[test]
fn omitted_parts_are_left_out() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    let css = compile(&sass, "tests/fixtures/omit.scss");
    for part in [
        ".sockel-toolbar",
        "--band-chrome-ground",
        ".sockel-cluster.sockel-wide",
        ".sockel-button.sockel-fold {",
        ".sockel-head-trail",
        ".sockel-split-button",
        "--table-max-height",
        "--danger:",
        ".sockel-fieldset",
        ".sockel-field-error",
        ".sockel-input.sockel-small",
        ".sockel-input-box.sockel-touch",
        ".sockel-input.sockel-busy",
        ".sockel-input[aria-invalid=true]",
        ".sockel-input-box.sockel-invalid",
        ".sockel-input-box.sockel-disabled",
        ".sockel-menu-item[aria-disabled=true]",
        ".sockel-menu-item-mark",
        ".sockel-pager.sockel-busy",
    ] {
        assert!(defaults.contains(part), "{part} in the defaults");
        assert!(!css.contains(part), "{part} left out");
    }
    for kept in [
        ".sockel-stack.sockel-loose,\n.sockel-cluster.sockel-loose {",
        ".sockel-button.sockel-fold-wide",
        ".sockel-head-subtitle {\n  flex: 0 0 100%;",
        ".sockel-menu-item {",
        "--success:",
        ".sockel-field-hint {\n  margin: 0;",
        ".sockel-field-inputs {",
        ".sockel-input:disabled",
        ".sockel-input-box:focus-within",
        ".sockel-menu-item-icon {",
        ".sockel-pager-button {",
    ] {
        assert!(css.contains(kept), "{kept} kept");
    }
}

#[test]
fn an_unknown_part_is_an_error() {
    let Some(sass) = sass() else { return };
    assert!(fails(&sass, "tests/fixtures/omit_unknown.scss").contains("no part layout.toolbars"));
}

#[test]
fn badge_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("height: var(--mark-height);"));
    assert!(!defaults.contains(":not(.sockel-solid)"));
    assert!(defaults.contains("transform: translate(50%, -50%);"));
    assert!(defaults.contains("inset-block-start: var(--corner-top, 0);"));
    let css = compile(&sass, "tests/fixtures/badge_settings.scss");
    assert!(css.contains("height: 1.375rem;"));
    assert!(css.contains("--tone: var(--text);"));
    assert!(css.contains("transform: none;"));
    assert!(css.contains("background: color-mix(in srgb, var(--tone) 12%, transparent);"));
    assert!(css.contains("calc((var(--count-digits) + 1) * 1ch + 2 * 0.25rem)"));
}

#[test]
fn choice_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("background: var(--accent);"));
    assert!(!defaults.contains("border-inline-start-color: var(--border);"));
    let css = compile(&sass, "tests/fixtures/choice_settings.scss");
    assert!(css.contains("border-inline-start-color: var(--border);"));
    assert!(css.contains("color: var(--muted);"));
    assert!(css.contains("background: color-mix(in srgb, var(--accent) 22%, transparent);"));
    assert!(css.contains("padding: 0;"));
    assert!(defaults.contains("--choice-height: var(--control-height);\n  display: inline-grid;"));
    assert!(!defaults.contains("border-block-start-color: var(--border);"));
    assert!(css.contains("--choice-height: var(--control-height-small);\n  display: inline-grid;"));
    assert!(css.contains("transparent);\n  color: var(--text);"));
    assert!(css.contains("border-block-start-color: var(--border);"));
    assert!(css.contains("@media (width < 48rem) {\n  .sockel-choice.sockel-stack-narrow {"));
    assert!(!defaults.contains(":where(.sockel-choice)"));
    assert!(css.contains(":where(.sockel-choice) {\n  align-self: center;"));
    assert!(css.contains("grid-auto-columns: auto;"));
    assert!(css.contains(".sockel-choice-option.sockel-icon {\n  min-width: var(--control-height-small);\n  padding-inline: 0;"));
    assert!(css.contains(".sockel-choice:focus-within {\n  border-color: var(--accent);"));
}
