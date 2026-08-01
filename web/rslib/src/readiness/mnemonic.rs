// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Whimsical mnemonic cues.
//!
//! A whimsy cue is a concept-relevant, deliberately silly hook that appears
//! while teaching and while practising retrieval, and disappears during formal
//! testing. Because the `whimsy_enabled` flag doubles as an ablation control
//! for measuring whether whimsy actually helps, removal has to be exact: with
//! the flag off the rendered card must look exactly like a card that never had
//! a cue at all.
//!
//! Removal happens at render time. Turning the flag off never edits stored note
//! content.

/// Class on the element wrapping a card's whimsy cue in rendered output.
/// Elements carrying this class are removed, wrapper included, when whimsy is
/// disabled or the card is a neutral test item.
pub const WHIMSY_CUE_CLASS: &str = "whimsy-cue";

/// Class on the element wrapping a card's concept map. A separate field from
/// the whimsy cue, and never removed with it.
pub const CONCEPT_MAP_CLASS: &str = "concept-map";

/// Note field holding the whimsical cue.
pub const WHIMSY_FIELD: &str = "Whimsy";

/// Note field holding the concept map.
pub const CONCEPT_MAP_FIELD: &str = "ConceptMap";

/// A note carrying this tag is a neutral test item: its whimsy cue is never
/// rendered, whatever the config flag says.
pub const NEUTRAL_TEST_TAG: &str = "neutral-test";

/// Remove every whimsy cue element, wrapper and all, from rendered card html.
///
/// This is the strip path applied to rendered question/answer html when the
/// cue must not be shown. It must leave no trace: no wrapper, no emptied
/// container, no whitespace where the cue used to be. Html containing no cue
/// comes back unchanged.
pub(crate) fn strip_whimsy_cue(rendered: &str) -> String {
    // The class name has to appear somewhere for there to be anything to do,
    // so the overwhelmingly common case costs one substring search.
    if !rendered.contains(WHIMSY_CUE_CLASS) {
        return rendered.to_string();
    }

    let mut out = String::with_capacity(rendered.len());
    let mut idx = 0;
    while idx < rendered.len() {
        let Some(offset) = rendered[idx..].find('<') else {
            out.push_str(&rendered[idx..]);
            return out;
        };
        let tag_start = idx + offset;
        match parse_start_tag(rendered, tag_start) {
            Some(tag) if tag.has_whimsy_class => {
                out.push_str(&rendered[idx..tag_start]);
                idx = if tag.self_closing {
                    tag.after_tag
                } else {
                    end_of_element(rendered, tag.name, tag.after_tag)
                };
            }
            // not a cue: copy the `<` and carry on from just after it, so a
            // cue nested inside another element is still found
            _ => {
                out.push_str(&rendered[idx..=tag_start]);
                idx = tag_start + 1;
            }
        }
    }
    out
}

/// A parsed start tag, eg `<div class="whimsy-cue">`.
struct StartTag<'a> {
    /// The element name, as written.
    name: &'a str,
    /// Byte index just past the closing `>`.
    after_tag: usize,
    /// The tag was written `<foo ... />` and so encloses nothing.
    self_closing: bool,
    /// Its `class` attribute carries [`WHIMSY_CUE_CLASS`] as a whole token.
    has_whimsy_class: bool,
}

/// Parse the start tag beginning at `at`, which must index a `<`.
///
/// Returns `None` for anything that is not a start tag: closing tags,
/// comments, doctypes, and a bare `<` in text.
fn parse_start_tag(html: &str, at: usize) -> Option<StartTag<'_>> {
    let rest = &html[at + 1..];
    let name_len = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    if name_len == 0 || !rest.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    let name = &rest[..name_len];

    // walk the attributes, respecting quoting, until the tag closes
    let attrs_start = at + 1 + name_len;
    let bytes = html.as_bytes();
    let mut idx = attrs_start;
    let mut quote: Option<u8> = None;
    let end = loop {
        if idx >= bytes.len() {
            // unterminated tag; nothing sensible to remove
            return None;
        }
        let byte = bytes[idx];
        match quote {
            Some(q) if byte == q => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => break idx,
            None => {}
        }
        idx += 1;
    };

    let attrs = &html[attrs_start..end];
    Some(StartTag {
        name,
        after_tag: end + 1,
        self_closing: attrs.trim_end().ends_with('/'),
        has_whimsy_class: class_attr(attrs)
            .is_some_and(|value| value.split_whitespace().any(|c| c == WHIMSY_CUE_CLASS)),
    })
}

/// The value of the `class` attribute within a start tag's attribute text.
fn class_attr(attrs: &str) -> Option<&str> {
    let mut rest = attrs;
    while let Some(offset) = rest.find("class") {
        let before_ok = offset == 0
            || rest[..offset]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_whitespace());
        let after = rest[offset + "class".len()..].trim_start();
        if before_ok {
            if let Some(value) = after.strip_prefix('=') {
                let value = value.trim_start();
                return Some(match value.as_bytes().first() {
                    Some(b'"') => value[1..].split('"').next().unwrap_or(""),
                    Some(b'\'') => value[1..].split('\'').next().unwrap_or(""),
                    _ => value.split_whitespace().next().unwrap_or(""),
                });
            }
        }
        rest = &rest[offset + "class".len()..];
    }
    None
}

/// Byte index just past the `</name>` matching a start tag that ended at
/// `from`, counting nested elements of the same name. An unclosed element
/// swallows the rest of the html, which is what a browser would render anyway.
fn end_of_element(html: &str, name: &str, from: usize) -> usize {
    let mut depth = 1usize;
    let mut idx = from;
    while let Some(offset) = html[idx..].find('<') {
        let tag_start = idx + offset;
        let rest = &html[tag_start + 1..];
        let (closing, rest) = match rest.strip_prefix('/') {
            Some(rest) => (true, rest),
            None => (false, rest),
        };
        if rest.len() >= name.len()
            && rest[..name.len()].eq_ignore_ascii_case(name)
            && !rest[name.len()..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-')
        {
            if closing {
                depth -= 1;
                if depth == 0 {
                    return html[tag_start..]
                        .find('>')
                        .map(|o| tag_start + o + 1)
                        .unwrap_or(html.len());
                }
            } else if !parse_start_tag(html, tag_start).is_some_and(|t| t.self_closing) {
                depth += 1;
            }
        }
        idx = tag_start + 1;
    }
    html.len()
}

/// Whether this note's whimsy cue may be rendered.
///
/// False whenever the card is a neutral test item, regardless of the flag.
pub(crate) fn whimsy_cue_visible(whimsy_enabled: bool, tags: &[String]) -> bool {
    whimsy_enabled && !tags.iter().any(|tag| tag == NEUTRAL_TEST_TAG)
}

/// Whether a note field's content may be counted towards scoring evidence.
///
/// Whimsy is decoration, not knowledge: it must never inflate an evidence
/// count.
pub(crate) fn field_bears_evidence(field_name: &str) -> bool {
    field_name != WHIMSY_FIELD
}
