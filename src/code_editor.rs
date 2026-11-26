//! CodeEditor: a code editor that knows no language. An uncontrolled
//! textarea, so the browser keeps its undo, under an `aria-hidden` `<pre>`
//! repeating its text in highlighted spans, beside a gutter of line numbers
//! marking the lines with diagnostics (`style/_code-editor.scss`). The
//! highlighting, the diagnostics and the completions come in through props.
//! Offsets are UTF-8 byte offsets into the text; an offset past the end is
//! clamped to the end and one inside a character floors to its start, so
//! none panics. An app whose analyser counts characters converts with
//! [`char_to_byte`](crate::char_to_byte).
//!
//! Colours are the `--code-*` custom properties, see
//! [`CodeToken`](crate::CodeToken).

use crate::testid::with_testid;
use crate::use_field;
use std::ops::Range;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlDocument, HtmlTextAreaElement};
use yew::prelude::*;

use crate::code::{CodeSpan, Diagnostic, Piece, Severity, floor, pieces};
use crate::phrasebookcatalog01::{L10N, label};
use crate::tooltip::next_id;
use crate::{
    Dismiss, PopoverAlign, Side, class, listbox, listbox_option, option_id, use_overlays,
    use_popover,
};

/// One completion: what the list shows, and the whole text and the
/// selection accepting it leaves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub label: AttrValue,
    pub detail: Option<AttrValue>,
    pub text: String,
    /// The selection after accepting, as UTF-8 byte offsets into `text`;
    /// past the end clamps, off a char boundary floors.
    pub select: Range<usize>,
}

/// The completions at the cursor, the list anchored at `start`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completions {
    /// A UTF-8 byte offset into the text; past the end clamps, off a char
    /// boundary floors.
    pub start: usize,
    pub items: Vec<Completion>,
}

/// What completions are asked for: the text, the cursor, and whether they
/// were asked for (Ctrl+Space) rather than offered while typing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionQuery {
    pub text: String,
    /// The cursor as a UTF-8 byte offset into `text`, always on a char
    /// boundary.
    pub cursor: usize,
    pub forced: bool,
}

/// A code editor's classes: `code-editor`, `invalid` while a diagnostic is
/// an error.
pub fn code_editor(invalid: bool) -> Classes {
    classes!(class::CODE_EDITOR, invalid.then_some(class::INVALID))
}

#[derive(Properties, PartialEq)]
pub struct CodeEditorProps {
    pub value: AttrValue,
    /// Each edit, with the new text.
    pub on_input: Callback<String>,
    /// The highlighted spans of `value`, in order and not overlapping.
    #[prop_or_default]
    pub spans: Rc<[CodeSpan]>,
    #[prop_or_default]
    pub diagnostics: Rc<[Diagnostic]>,
    /// Asked for the completions as the text or cursor changes, and on
    /// Ctrl+Space; none, no list.
    #[prop_or_default]
    pub complete: Option<Callback<CompletionQuery, Option<Rc<Completions>>>>,
    /// The lines it is tall at least.
    #[prop_or(4)]
    pub rows: u32,
    #[prop_or_default]
    pub readonly: bool,
    /// The accessible name, when there is no label for `id`.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The completion list's name, else the catalog's.
    #[prop_or_default]
    pub completions_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    /// `field_describedby`, the ids of the field's hint and error.
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Set to the textarea, for [`code_editor_insert`]. The ref given on
    /// the first render is the one kept.
    #[prop_or_default]
    pub node_ref: NodeRef,
}

/// A code editor: the text in a monospace textarea, highlighted by
/// `spans`, with line numbers, `diagnostics` underlined and marked in the
/// gutter, Tab and Shift+Tab indenting and outdenting, Enter keeping the
/// line's indent, and a completion list at the cursor. Edits go through the
/// browser's editing commands, so they are undone as typing is. It grows
/// with the text and scrolls sideways, following the caret.
#[function_component]
pub fn CodeEditor(props: &CodeEditorProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let field = use_field().unwrap_or_default();
    let ids = use_state(|| (next_id("code-editor-list"), next_id("code-editor-notes")));
    let (list_id, notes_id) = (ids.0.clone(), ids.1.clone());
    let area = (*use_state(|| props.node_ref.clone())).clone();
    let frame = use_node_ref();
    let anchor = use_node_ref();
    let list = use_node_ref();
    let offers = use_state(|| None::<Rc<Completions>>);
    let active = use_state(|| 0usize);
    let caret = use_state(|| 0usize);
    // Set while an accepted completion is inserted, so its `input` does not
    // offer completions again.
    let accepting = use_mut_ref(|| false);
    let overlays = use_overlays();

    let open = offers.is_some();

    let close = {
        let offers = offers.clone();
        Callback::from(move |_: Dismiss| offers.set(None))
    };

    let placed = use_popover(
        open,
        anchor.clone(),
        list.clone(),
        Side::Below,
        PopoverAlign::Start,
        close,
    );

    // The textarea is uncontrolled: only a value from outside is written
    // into it, which typing never needs.
    {
        let area = area.clone();
        use_effect_with(props.value.clone(), move |value| {
            if let Some(el) = area.cast::<HtmlTextAreaElement>()
                && el.value() != value.as_str()
            {
                el.set_value(value);
            }
        });
    }

    {
        let (frame, caret) = (frame.clone(), *caret);
        use_effect_with((caret, props.value.clone()), move |_| follow_caret(&frame));
    }

    let track = {
        let (area, caret) = (area.clone(), caret.clone());
        Rc::new(move || {
            if let Some(el) = area.cast::<HtmlTextAreaElement>() {
                caret.set(cursor(&el));
            }
        })
    };

    let refresh = {
        let (area, offers, active, complete) = (
            area.clone(),
            offers.clone(),
            active.clone(),
            props.complete.clone(),
        );
        Rc::new(move |forced: bool| {
            let (Some(el), Some(complete)) = (area.cast::<HtmlTextAreaElement>(), &complete) else {
                return;
            };

            let query = CompletionQuery {
                cursor: cursor(&el),
                text: el.value(),
                forced,
            };

            active.set(0);
            offers.set(complete.emit(query).filter(|c| !c.items.is_empty()));
        })
    };

    let accept = {
        let (area, offers, accepting) = (area.clone(), offers.clone(), accepting.clone());
        Rc::new(move |index: usize| {
            let (Some(el), Some(at)) = (area.cast::<HtmlTextAreaElement>(), (*offers).clone())
            else {
                return;
            };
            let Some(item) = at.items.get(index) else {
                return;
            };

            let value = el.value();
            let (replace, text) = difference(&value, &item.text);
            *accepting.borrow_mut() = true;
            apply(
                &el,
                &value,
                &Edit {
                    replace,
                    text: text.to_owned(),
                    select: item.select.clone(),
                },
            );
            *accepting.borrow_mut() = false;
            let _ = el.focus();
            offers.set(None);
        })
    };

    let on_input = {
        let (on_input, refresh, track, accepting) = (
            props.on_input.clone(),
            refresh.clone(),
            track.clone(),
            accepting.clone(),
        );
        Callback::from(move |e: InputEvent| {
            let Some(el) = e.target_dyn_into::<HtmlTextAreaElement>() else {
                return;
            };

            on_input.emit(el.value());
            track();

            if !*accepting.borrow() {
                refresh(false);
            }
        })
    };

    let on_keydown = {
        let (area, offers, active, refresh, accept, readonly) = (
            area.clone(),
            offers.clone(),
            active.clone(),
            refresh.clone(),
            accept.clone(),
            props.readonly,
        );
        Callback::from(move |e: KeyboardEvent| {
            if readonly {
                return;
            }

            let key = e.key();

            if e.ctrl_key() && key == " " {
                e.prevent_default();
                refresh(true);
                return;
            }

            if let Some(at) = &*offers {
                let count = at.items.len().max(1);

                match key.as_str() {
                    "ArrowDown" => {
                        e.prevent_default();
                        active.set(active.saturating_add(1).checked_rem(count).unwrap_or(0));
                        return;
                    }
                    "ArrowUp" => {
                        e.prevent_default();
                        active.set(active.checked_sub(1).unwrap_or(count.saturating_sub(1)));
                        return;
                    }
                    "Enter" | "Tab" => {
                        e.prevent_default();
                        accept(*active);
                        return;
                    }
                    "Escape" => {
                        e.prevent_default();
                        offers.set(None);
                        return;
                    }
                    _ => {}
                }
            }

            if let Some(el) = area.cast::<HtmlTextAreaElement>()
                && keydown(&el, &e)
            {
                e.prevent_default();
            }
        })
    };

    let on_caret = {
        let track = track.clone();
        Callback::from(move |_: Event| track())
    };

    let on_blur = {
        let offers = offers.clone();
        Callback::from(move |_: FocusEvent| offers.set(None))
    };

    // The textarea is as large as its text and never scrolls; the frame
    // does.
    let on_scroll = Callback::from(|e: Event| {
        if let Some(el) = e.target_dyn_into::<Element>() {
            el.set_scroll_top(0);
            el.set_scroll_left(0);
        }
    });

    let popup = offers.as_ref().map(|at| {
        let style = placed.map(|p| {
            format!(
                "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
                p.left, p.top, p.max_height
            )
        });

        let rows = at.items.iter().enumerate().map(|(index, item)| {
            let onclick = {
                let accept = accept.clone();
                Callback::from(move |_: MouseEvent| accept(index))
            };
            let onmousemove = {
                let active = active.clone();
                Callback::from(move |_: MouseEvent| {
                    if *active != index {
                        active.set(index);
                    }
                })
            };

            html! {
                <div id={option_id(&list_id, index)} role="option"
                    class={listbox_option(index == *active)}
                    aria-selected={(index == *active).then_some("true")}
                    {onclick} {onmousemove}>
                    <span class={class::LISTBOX_OPTION_TEXT}>
                        <span class={classes!(class::LISTBOX_OPTION_LABEL, class::CODE_EDITOR_OPTION)}>
                            { item.label.clone() }
                        </span>
                        if let Some(detail) = &item.detail {
                            <span class={class::LISTBOX_OPTION_DESCRIPTION}>{ detail.clone() }</span>
                        }
                    </span>
                </div>
            }
        });

        // A press in the list keeps the keyboard in the textarea.
        let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());

        overlays.portal(html! {
            <div ref={list.clone()} id={list_id.clone()} role="listbox"
                aria-label={label(&phrases, &props.completions_label, "sockel-code-editor-completions")}
                class={listbox(placed.is_some())} {style} {onmousedown}>
                { for rows }
            </div>
        })
    });

    let source = props.value.as_str();
    let marks = props
        .diagnostics
        .iter()
        .map(|d| (d.range.clone(), d.severity))
        .collect::<Vec<_>>();
    let anchor_at = offers.as_ref().map(|at| at.start);
    let pieces = pieces(source, &props.spans, &marks, &[Some(*caret), anchor_at]);

    let code = pieces.into_iter().map(|piece| match piece {
        Piece::Text { range, span, mark } => {
            let text = source.get(range).unwrap_or_default().to_owned();
            let token = span
                .and_then(|i| props.spans.get(i))
                .map(|s| s.token.class());
            let html = match token {
                Some(class) => html! { <span class={class}>{ text }</span> },
                None => html! { { text } },
            };

            match mark {
                Some(severity) => html! {
                    <span class={classes!(class::CODE_EDITOR_MARK, severity.class())}>{ html }</span>
                },
                None => html,
            }
        }
        Piece::Point(0) => html! { <span class={class::CODE_EDITOR_CARET}></span> },
        Piece::Point(_) => html! { <span ref={anchor.clone()}></span> },
        Piece::EmptyMark(severity) => html! {
            <span class={classes!(class::CODE_EDITOR_MARK, class::CODE_EDITOR_EMPTY_MARK, severity.class())}></span>
        },
    });

    let lines = gutter(source, &props.diagnostics);
    let gutter = lines.iter().enumerate().map(|(line, (severity, notes))| {
        let number = line.saturating_add(1);

        match severity {
            Some(severity) => html! {
                <div class={classes!(class::CODE_EDITOR_LINE, severity.class())} title={notes.clone()}>
                    <span class={class::CODE_EDITOR_MARKER}></span>{ number }
                </div>
            },
            None => html! { <div class={class::CODE_EDITOR_LINE}>{ number }</div> },
        }
    });

    let errors = props
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error);
    let (id, describedby, invalid) = field.control(&props.id, &props.describedby, errors);
    let has_notes = !props.diagnostics.is_empty();
    let describedby = [
        describedby.map(|d| d.to_string()),
        has_notes.then(|| notes_id.to_string()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let describedby = (!describedby.is_empty()).then(|| describedby.join(" "));
    let style = format!("--code-editor-rows: {};", props.rows.max(1));

    with_testid(
        html! {
            <div ref={frame} class={classes!(code_editor(invalid), props.class.clone())} {style}>
                // First: yew matches unkeyed children from the end, so the
                // list coming and going there would rebuild the textarea.
                { for popup }
                <div class={class::CODE_EDITOR_GUTTER} aria-hidden="true">{ for gutter }</div>
                // The spare line keeps the textarea's text inside its box,
                // so that it never has anything to scroll.
                <pre class={class::CODE_EDITOR_CODE} aria-hidden="true">
                    { for code }{ "\n " }
                </pre>
                <textarea ref={area} class={class::CODE_EDITOR_INPUT}
                    id={id}
                    aria-label={props.label.clone()}
                    aria-describedby={describedby}
                    aria-invalid={invalid.then_some("true")}
                    aria-autocomplete={props.complete.is_some().then_some("list")}
                    aria-controls={open.then(|| list_id.clone())}
                    aria-activedescendant={open.then(|| option_id(&list_id, *active))}
                    readonly={props.readonly}
                    rows="1" wrap="off" spellcheck="false" autocapitalize="off" autocomplete="off"
                    oninput={on_input} onkeydown={on_keydown}
                    onkeyup={on_caret.reform(|e: KeyboardEvent| e.into())}
                    onclick={on_caret.reform(|e: MouseEvent| e.into())}
                    onselect={on_caret.clone()}
                    onblur={on_blur} onscroll={on_scroll} />
                if has_notes {
                    <ul id={notes_id.clone()} class={class::VISUALLY_HIDDEN}>
                        { for props.diagnostics.iter().map(|d| html! {
                            <li>{ format!("{}: {}", line_of(source, d.range.start).saturating_add(1), d.message) }</li>
                        }) }
                    </ul>
                }
            </div>
        },
        props.testid.clone(),
    )
}

/// Each line's worst severity and its messages, one per line of `source`.
fn gutter(source: &str, diagnostics: &[Diagnostic]) -> Vec<(Option<Severity>, Option<String>)> {
    let mut lines = vec![(None, None::<String>); source.split('\n').count()];

    for d in diagnostics {
        let Some((severity, notes)) = lines.get_mut(line_of(source, d.range.start)) else {
            continue;
        };

        *severity = (*severity).max(Some(d.severity));

        match notes {
            Some(notes) => {
                notes.push('\n');
                notes.push_str(&d.message);
            }
            None => *notes = Some(d.message.to_string()),
        }
    }

    lines
}

/// The line, from zero, byte `at` is on.
fn line_of(text: &str, at: usize) -> usize {
    text.get(..floor(text, at.min(text.len())))
        .unwrap_or_default()
        .matches('\n')
        .count()
}

/// The byte offset of `utf16` units into `text`, as the DOM counts.
fn byte_offset(text: &str, utf16: u32) -> usize {
    let mut count = 0u32;

    for (index, c) in text.char_indices() {
        if count >= utf16 {
            return index;
        }

        count = count.saturating_add(u32::try_from(c.len_utf16()).unwrap_or(2));
    }

    text.len()
}

/// The UTF-16 units of `text` up to byte `at`.
fn utf16_at(text: &str, at: usize) -> u32 {
    let units = text
        .get(..floor(text, at.min(text.len())))
        .unwrap_or_default()
        .encode_utf16()
        .count();
    u32::try_from(units).unwrap_or(u32::MAX)
}

fn cursor(area: &HtmlTextAreaElement) -> usize {
    let at = area.selection_start().ok().flatten().unwrap_or(0);
    byte_offset(&area.value(), at)
}

/// The part of `old` to replace, and the text to put there, to make `new`.
fn difference<'a>(old: &str, new: &'a str) -> (Range<usize>, &'a str) {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|&((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((i, _), _)| i);

    let suffix = old
        .get(prefix..)
        .unwrap_or_default()
        .chars()
        .rev()
        .zip(new.get(prefix..).unwrap_or_default().chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();

    let old_end = old.len().saturating_sub(suffix).max(prefix);
    let new_end = new.len().saturating_sub(suffix).max(prefix);
    (
        prefix..old_end,
        new.get(prefix..new_end).unwrap_or_default(),
    )
}

/// An edit: `replace` (bytes of the text before) becomes `text`, and then
/// `select` (bytes of the text after) is selected.
#[derive(Debug, PartialEq, Eq)]
struct Edit {
    replace: Range<usize>,
    text: String,
    select: Range<usize>,
}

/// The edit for Tab (`shift`: Shift+Tab) on the selection `start..end` of
/// `value`: a tab at the cursor, or the selected lines indented or
/// outdented. `None` when nothing changes.
fn indent(value: &str, start: usize, end: usize, shift: bool) -> Option<Edit> {
    let selected = value.get(start..end)?;
    let multiline = selected.contains('\n');

    if !shift && !multiline {
        let at = start.saturating_add(1);
        return Some(Edit {
            replace: start..end,
            text: "\t".to_owned(),
            select: at..at,
        });
    }

    let line_start = value
        .get(..start)?
        .rfind('\n')
        .map_or(0, |i| i.saturating_add(1));

    let block_end = if end > start && value.get(..end)?.ends_with('\n') {
        end.saturating_sub(1)
    } else {
        value
            .get(end..)?
            .find('\n')
            .map_or(value.len(), |i| end.saturating_add(i))
    };

    let block = value.get(line_start..block_end)?;
    let mut removed = 0usize;

    let lines = block
        .split('\n')
        .enumerate()
        .map(|(n, line)| {
            if !shift {
                return format!("\t{line}");
            }

            let strip = if line.starts_with('\t') {
                1
            } else {
                line.len()
                    .saturating_sub(line.trim_start_matches(' ').len())
                    .min(4)
            };

            if n == 0 {
                removed = strip;
            }

            line.get(strip..).unwrap_or(line).to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n");

    if lines == block {
        return None;
    }

    let select = if multiline {
        line_start..line_start.saturating_add(lines.len())
    } else {
        let at = start.saturating_sub(removed).max(line_start);
        at..at
    };

    Some(Edit {
        replace: line_start..block_end,
        text: lines,
        select,
    })
}

/// The edit for Enter at the selection `start..end`: a new line keeping
/// the indent of the one the cursor is on.
fn newline(value: &str, start: usize, end: usize) -> Option<Edit> {
    let line_start = value
        .get(..start)?
        .rfind('\n')
        .map_or(0, |i| i.saturating_add(1));
    let line = value.get(line_start..start)?;
    let indent = line
        .get(
            ..line
                .len()
                .saturating_sub(line.trim_start_matches([' ', '\t']).len()),
        )
        .unwrap_or_default();
    let text = format!("\n{indent}");
    let at = start.saturating_add(text.len());

    Some(Edit {
        replace: start..end,
        text,
        select: at..at,
    })
}

/// Handle the keys which edit beyond inserting a character. Whether the
/// browser's own action is to be prevented.
fn keydown(area: &HtmlTextAreaElement, event: &KeyboardEvent) -> bool {
    if event.is_composing() || event.ctrl_key() || event.meta_key() || event.alt_key() {
        return false;
    }

    let value = area.value();
    let s16 = area.selection_start().ok().flatten().unwrap_or(0);
    let e16 = area.selection_end().ok().flatten().unwrap_or(s16);
    let (start, end) = (byte_offset(&value, s16), byte_offset(&value, e16));

    let edit = match event.key().as_str() {
        "Tab" => {
            let Some(edit) = indent(&value, start, end, event.shift_key()) else {
                return true;
            };
            edit
        }
        "Enter" if !event.shift_key() => match newline(&value, start, end) {
            Some(edit) => edit,
            None => return false,
        },
        _ => return false,
    };

    apply(area, &value, &edit);
    true
}

/// Insert `text` into the [`CodeEditor`] whose `node_ref` is `node_ref`,
/// in place of its selection (at the cursor when nothing is selected),
/// leaving the cursor after it. It goes through the browser's editing
/// commands as typing does, so Ctrl+Z undoes it and `on_input` is called
/// with the new text. The editor takes the keyboard. `false`, and nothing
/// done, when the ref is not on a mounted editor or the editor is
/// `readonly`.
///
/// ```
/// use sockel::{CodeEditor, code_editor_insert};
/// use yew::prelude::*;
///
/// #[function_component]
/// fn Expression() -> Html {
///     let text = use_state(String::new);
///     let editor = use_node_ref();
///
///     let on_input = {
///         let text = text.clone();
///         Callback::from(move |value: String| text.set(value))
///     };
///
///     let onclick = {
///         let editor = editor.clone();
///         Callback::from(move |_: MouseEvent| {
///             code_editor_insert(&editor, "price");
///         })
///     };
///
///     html! {
///         <>
///             <CodeEditor value={(*text).clone()} {on_input} label="Expression"
///                 node_ref={editor} />
///             <button {onclick}>{ "price" }</button>
///         </>
///     }
/// }
/// ```
pub fn code_editor_insert(node_ref: &NodeRef, text: &str) -> bool {
    let Some(area) = node_ref.cast::<HtmlTextAreaElement>() else {
        return false;
    };

    if !area.is_connected() || area.read_only() {
        return false;
    }

    let _ = area.focus();
    let value = area.value();
    let s16 = area.selection_start().ok().flatten().unwrap_or(0);
    let e16 = area.selection_end().ok().flatten().unwrap_or(s16);
    let (start, end) = (byte_offset(&value, s16), byte_offset(&value, e16));
    let (start, end) = (start.min(end), start.max(end));
    let after = start.saturating_add(text.len());

    let edit = Edit {
        replace: start..end,
        text: text.to_owned(),
        select: after..after,
    };

    apply(&area, &value, &edit);
    true
}

/// Make `edit` through the browser's editing commands, so that it lands on
/// its undo stack and fires `input` as typing does.
fn apply(area: &HtmlTextAreaElement, value: &str, edit: &Edit) {
    let from = utf16_at(value, edit.replace.start);
    let to = utf16_at(value, edit.replace.end);
    let _ = area.set_selection_range(from, to);

    let document = area
        .owner_document()
        .and_then(|d| d.dyn_into::<HtmlDocument>().ok());

    let done = match document {
        Some(document) if edit.text.is_empty() => {
            from == to || document.exec_command("delete").unwrap_or(false)
        }
        Some(document) => document
            .exec_command_with_show_ui_and_value("insertText", false, &edit.text)
            .unwrap_or(false),
        None => false,
    };

    if !done {
        let _ = area.set_range_text(&edit.text);

        if let Ok(event) = web_sys::InputEvent::new("input") {
            let _ = area.dispatch_event(&event);
        }
    }

    let after = area.value();
    let _ = area.set_selection_range(
        utf16_at(&after, edit.select.start),
        utf16_at(&after, edit.select.end),
    );
}

/// Scroll the frame so that the caret can be seen: the textarea is as
/// large as its text and never scrolls itself, so the browser does not do
/// this on its own.
fn follow_caret(frame: &NodeRef) {
    let Some(frame) = frame.cast::<Element>() else {
        return;
    };

    let Ok(Some(caret)) = frame.query_selector(&format!(".{}", class::CODE_EDITOR_CARET)) else {
        return;
    };

    let gutter = frame
        .query_selector(&format!(".{}", class::CODE_EDITOR_GUTTER))
        .ok()
        .flatten()
        .map_or(0.0, |g| g.get_bounding_client_rect().width());

    let view = frame.get_bounding_client_rect();
    let at = caret.get_bounding_client_rect();
    let margin = at.height().max(16.0) * 2.0;

    let left = view.left() + gutter + margin;
    let right = view.right() - margin;

    let by = if at.left() < left {
        at.left() - left
    } else if at.left() > right {
        at.left() - right
    } else {
        return;
    };

    frame.set_scroll_left(frame.scroll_left().saturating_add(by.round() as i32));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CodeToken;

    fn span(range: Range<usize>, token: CodeToken) -> CodeSpan {
        CodeSpan::new(range, token)
    }

    #[test]
    fn utf16_round_trips() {
        let text = "a😀é\nb";
        assert_eq!(utf16_at(text, 1), 1);
        assert_eq!(utf16_at(text, 5), 3);
        assert_eq!(utf16_at(text, 7), 4);
        assert_eq!(byte_offset(text, 3), 5);
        assert_eq!(byte_offset(text, 4), 7);
        assert_eq!(byte_offset(text, 99), text.len());
    }

    #[test]
    fn lines() {
        assert_eq!(line_of("a\nb\nc", 0), 0);
        assert_eq!(line_of("a\nb\nc", 2), 1);
        assert_eq!(line_of("a\nb\nc", 99), 2);
    }

    #[test]
    fn the_gutter_takes_each_lines_worst() {
        let diagnostics = [
            Diagnostic::new(0..1, Severity::Warning, "w"),
            Diagnostic::new(2..3, Severity::Info, "i"),
            Diagnostic::new(2..2, Severity::Error, "e"),
        ];
        assert_eq!(
            gutter("a\nb\nc", &diagnostics),
            [
                (Some(Severity::Warning), Some("w".to_owned())),
                (Some(Severity::Error), Some("i\ne".to_owned())),
                (None, None),
            ]
        );
    }

    #[test]
    fn differences() {
        assert_eq!(difference("a + wi", "a + wits"), (6..6, "ts"));
        assert_eq!(difference("a.le + 1", "a.len() + 1"), (4..4, "n()"));
        assert_eq!(difference("x.[", "x[\"a b\"]"), (1..3, "[\"a b\"]"));
        assert_eq!(difference("same", "same"), (4..4, ""));
    }

    #[test]
    fn tab_inserts_a_tab() {
        assert_eq!(
            indent("ab", 1, 1, false),
            Some(Edit {
                replace: 1..1,
                text: "\t".into(),
                select: 2..2
            })
        );
    }

    #[test]
    fn tab_indents_selected_lines() {
        assert_eq!(
            indent("a\nb\nc", 0, 3, false),
            Some(Edit {
                replace: 0..3,
                text: "\ta\n\tb".into(),
                select: 0..5
            })
        );
    }

    #[test]
    fn shift_tab_outdents() {
        assert_eq!(
            indent("x\n    ab", 7, 7, true),
            Some(Edit {
                replace: 2..8,
                text: "ab".into(),
                select: 3..3
            })
        );
        assert_eq!(
            indent("\ta\n\tb", 0, 5, true),
            Some(Edit {
                replace: 0..5,
                text: "a\nb".into(),
                select: 0..3
            })
        );
        assert_eq!(indent("ab", 1, 1, true), None);
        assert_eq!(indent("ab", 3, 1, false), None);
    }

    #[test]
    fn enter_keeps_the_indent() {
        assert_eq!(
            newline("  a", 3, 3),
            Some(Edit {
                replace: 3..3,
                text: "\n  ".into(),
                select: 6..6
            })
        );
        assert_eq!(newline("é", 1, 1), None);
    }

    #[test]
    fn its_classes() {
        assert_eq!(code_editor(false).to_string(), class::CODE_EDITOR);
        assert_eq!(
            code_editor(true).to_string(),
            format!("{} {}", class::CODE_EDITOR, class::INVALID)
        );
    }

    #[test]
    fn inserting_without_an_editor_does_nothing() {
        assert!(!code_editor_insert(&NodeRef::default(), "x"));
    }

    fn render(props: CodeEditorProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<CodeEditor>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn renders_spans_marks_and_the_gutter() {
        let html = render(CodeEditorProps {
            value: "let x\n1".into(),
            on_input: Callback::noop(),
            spans: Rc::from([
                span(0..3, CodeToken::Keyword),
                span(6..7, CodeToken::Class("app-number")),
            ]),
            diagnostics: Rc::from([Diagnostic::new(6..7, Severity::Error, "bad")]),
            complete: None,
            rows: 2,
            readonly: false,
            label: Some("Code".into()),
            completions_label: None,
            id: None,
            describedby: None,
            class: Classes::new(),
            testid: Some("editor".into()),
            node_ref: NodeRef::default(),
        });
        assert!(
            html.contains("class=\"sockel-code-editor sockel-invalid\""),
            "{html}"
        );
        assert!(html.contains("--code-editor-rows: 2;"), "{html}");
        assert!(
            html.contains("<span class=\"sockel-code-token-keyword\">let</span>"),
            "{html}"
        );
        assert!(
            html.contains("<span class=\"sockel-code-editor-mark sockel-error\"><span class=\"app-number\">1</span></span>"),
            "{html}"
        );
        assert!(
            html.contains("<div title=\"bad\" class=\"sockel-code-editor-line sockel-error\">"),
            "{html}"
        );
        assert!(html.contains("aria-invalid=\"true\""), "{html}");
        assert!(html.contains(">2: bad</li>"), "{html}");
        assert!(html.contains("data-testid=\"editor\""), "{html}");
    }
}
