// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 010 -- deck generation with enforced citations and topic tags.
//!
//! The generator turns a vendored source document into MCAT cards. Its
//! invariants are enforced here rather than trusted: a chunk that cannot be
//! attributed, and a topic tag that matches no AAMC outline category, both fail
//! generation outright. Neither warns and continues. An uncited card, or a tag
//! pointing at an invented category, would become a fabricated measurement
//! feeding the coverage percentage and hence the give-up rule.
//!
//! There is no network access: the generator reads a file path and nothing
//! else. Cards built from a cited source are
//! [`crate::readiness::provenance::SOURCE_DERIVED_TAG`], never `ai-generated` --
//! they are transcribed from a work a human wrote, so they remain evidence.
//!
//! Field names come from ticket 006's contract and are re-exported from
//! [`crate::readiness::mnemonic`] rather than restated, so the two cannot drift.
//!
//! # Determinism
//!
//! Same document in, identical [`GeneratedDeck`] out. Nothing here reads the
//! clock, the system random source, or the file's path; the parser preserves
//! source order throughout and never puts anything in a `HashMap`, so no
//! hash seed can reorder a field, a tag or a card.
//!
//! # Why nothing is written until everything is validated
//!
//! [`Collection::generate_mcat_deck`] builds and validates the whole deck
//! before it opens a transaction, so a document with one bad chunk among good
//! ones leaves the collection exactly as it was found. A half-written deck
//! would be a half-fabricated one, and the missing chapter would be invisible
//! to whoever studied it.

use std::path::Path;

use crate::prelude::*;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::evidence::TOPIC_TAG_PREFIX;
use crate::readiness::mnemonic::CONCEPT_MAP_CLASS;
use crate::readiness::mnemonic::CONCEPT_MAP_FIELD;
use crate::readiness::mnemonic::NEUTRAL_TEST_TAG;
use crate::readiness::mnemonic::WHIMSY_CUE_CLASS;
use crate::readiness::mnemonic::WHIMSY_FIELD;
use crate::readiness::provenance::SOURCE_DERIVED_TAG;

/// The notetype generated decks use.
pub(crate) const NOTETYPE_NAME: &str = "MCAT Whimsified";

/// Note field holding the question.
pub(crate) const FRONT_FIELD: &str = "Front";

/// Note field holding the answer.
pub(crate) const BACK_FIELD: &str = "Back";

/// Note field holding the citation: the book and chapter the card came from.
pub(crate) const SOURCE_FIELD: &str = "Source";

/// The five fields of the generated notetype, in order.
///
/// Exactly these five. There is no `Topic` field -- topics live in `mcat::`
/// tags, which is what the evidence query reads -- and no `NeutralTest` field,
/// because a neutral test item is marked by a note tag.
pub(crate) const NOTETYPE_FIELDS: [&str; 5] = [
    FRONT_FIELD,
    BACK_FIELD,
    WHIMSY_FIELD,
    CONCEPT_MAP_FIELD,
    SOURCE_FIELD,
];

/// One card the generator produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedCard {
    /// The `Front` field.
    pub front: String,
    /// The `Back` field.
    pub back: String,
    /// The `Whimsy` field. Empty when the card carries no cue, and always empty
    /// on a neutral test item.
    pub whimsy: String,
    /// The `ConceptMap` field. Never empty when `whimsy` is non-empty.
    pub concept_map: String,
    /// The `Source` field: the book and chapter this card was built from. Never
    /// empty -- a card that cannot be cited is never emitted.
    pub source: String,
    /// The note's tags: at least one `mcat::<section>::<topic>` topic tag, the
    /// source-derived provenance marker, and `neutral-test` on a neutral item.
    pub tags: Vec<String>,
}

/// A deck the generator produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedDeck {
    /// The generated notetype's field names, in order.
    pub field_names: Vec<String>,
    /// The CC-BY attribution the source document carries, which the deck must
    /// carry onwards.
    pub attribution: String,
    /// The cards, in source order.
    pub cards: Vec<GeneratedCard>,
}

// -------------------------------------------------------------------------
// the source document format
// -------------------------------------------------------------------------

/// Header key naming the work being cited.
const BOOK_KEY: &str = "book";
/// Header key naming the licence the work is offered under.
const LICENSE_KEY: &str = "license";
/// Header key holding the attribution line the deck must carry onwards.
const ATTRIBUTION_KEY: &str = "attribution";

/// Chunk key naming the chapter being cited.
const CHAPTER_KEY: &str = "chapter";
/// Chunk key naming an AAMC outline topic. May repeat.
const TOPIC_KEY: &str = "topic";
/// Chunk key holding the question.
const PROMPT_KEY: &str = "prompt";
/// Chunk key holding the answer.
const ANSWER_KEY: &str = "answer";
/// Chunk key holding the whimsical retrieval cue.
const WHIMSY_KEY: &str = "whimsy";
/// Chunk key holding what each element of the cue maps to.
const CONCEPT_MAP_KEY: &str = "concept-map";
/// Chunk key marking a neutral test item.
const NEUTRAL_KEY: &str = "neutral";

/// The only value [`NEUTRAL_KEY`] may take. Anything else is a typo, and a typo
/// that silently read as "not neutral" would contaminate the control arm.
const NEUTRAL_YES: &str = "yes";

/// A blank-line-separated block of `key: value` lines, in source order.
///
/// An ordered `Vec` rather than a map: keys repeat (`topic`), and the order
/// they were written in is the order the tags come out in. A map would put a
/// hash seed between the document and the deck.
#[derive(Debug, Default)]
struct Block {
    /// 1-based line number the block started on, for error messages.
    line: usize,
    entries: Vec<(String, String)>,
}

impl Block {
    /// The first value written for `key`, trimmed. `None` when the key is
    /// absent; `Some("")` when it was written with nothing after the colon,
    /// because "present but empty" is a different mistake from "missing".
    fn value(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Every value written for `key`, in source order.
    fn values(&self, key: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// The first value for `key`, required to be present and non-empty.
    fn required(&self, key: &str) -> Result<&str> {
        match self.value(key) {
            Some(value) if !value.is_empty() => Ok(value),
            Some(_) => invalid_input!("source document line {}: {key:?} is empty", self.line),
            None => invalid_input!("source document line {}: {key:?} is missing", self.line),
        }
    }
}

/// Split a source document into blocks.
///
/// Lines beginning with `#` are comments and lines that are blank separate
/// blocks; a block that holds nothing but comments is not a block at all.
fn parse_blocks(text: &str) -> Result<Vec<Block>> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut current = Block::default();

    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            if !current.entries.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            invalid_input!("source document line {line_no}: {line:?} is not a `key: value` line");
        };
        let key = key.trim();
        if key.is_empty() {
            invalid_input!("source document line {line_no}: a key may not be empty");
        }
        if current.entries.is_empty() {
            current.line = line_no;
        }
        current
            .entries
            .push((key.to_string(), value.trim().to_string()));
    }
    if !current.entries.is_empty() {
        blocks.push(current);
    }

    Ok(blocks)
}

/// The document header: what the deck cites and under what licence.
#[derive(Debug)]
struct SourceHeader {
    book: String,
    attribution: String,
}

impl SourceHeader {
    /// Read the header block.
    ///
    /// A document that names no book cannot attribute anything it contains, so
    /// this is one of the two failures the whole ticket is about.
    fn parse(block: &Block) -> Result<SourceHeader> {
        let book = block.required(BOOK_KEY)?.to_string();
        // named but unused beyond this check: a document that states no licence
        // is one we have no right to redistribute cards from.
        block.required(LICENSE_KEY)?;
        let attribution = block.required(ATTRIBUTION_KEY)?.to_string();
        Ok(SourceHeader { book, attribution })
    }
}

/// One validated source chunk: everything needed for exactly one card.
#[derive(Debug)]
struct SourceChunk {
    chapter: String,
    topics: Vec<String>,
    prompt: String,
    answer: String,
    whimsy: String,
    concept_map: String,
    neutral: bool,
}

impl SourceChunk {
    /// Read and validate one chunk block.
    ///
    /// Every rejection here is a refusal to fabricate: an unattributable chunk,
    /// a topic no outline category claims, or a cue with no mapping behind it.
    fn parse(block: &Block) -> Result<SourceChunk> {
        let chapter = block.required(CHAPTER_KEY)?.to_string();
        let prompt = block.required(PROMPT_KEY)?.to_string();
        let answer = block.required(ANSWER_KEY)?.to_string();

        let written = block.values(TOPIC_KEY);
        if written.is_empty() {
            invalid_input!(
                "source document line {}: chunk names no {TOPIC_KEY}",
                block.line
            );
        }
        let mut topics: Vec<String> = Vec::with_capacity(written.len());
        for topic in written {
            if topic.is_empty() {
                invalid_input!(
                    "source document line {}: an empty {TOPIC_KEY} names no outline category",
                    block.line
                );
            }
            if !outline_knows(topic) {
                invalid_input!(
                    "source document line {}: topic {topic:?} matches no AAMC outline category",
                    block.line
                );
            }
            if !topics.iter().any(|seen| seen == topic) {
                topics.push(topic.to_string());
            }
        }

        let whimsy = block.value(WHIMSY_KEY).unwrap_or_default().to_string();
        let concept_map = block.value(CONCEPT_MAP_KEY).unwrap_or_default().to_string();
        if !whimsy.is_empty() && concept_map.is_empty() {
            invalid_input!(
                "source document line {}: a {WHIMSY_KEY} cue with no {CONCEPT_MAP_KEY} is \
                 decoration, not a mnemonic",
                block.line
            );
        }

        let neutral = match block.value(NEUTRAL_KEY) {
            None => false,
            Some(value) if value.eq_ignore_ascii_case(NEUTRAL_YES) => true,
            Some(value) => invalid_input!(
                "source document line {}: {NEUTRAL_KEY} must be {NEUTRAL_YES:?}, not {value:?}",
                block.line
            ),
        };

        Ok(SourceChunk {
            chapter,
            topics,
            prompt,
            answer,
            whimsy,
            concept_map,
            neutral,
        })
    }

    /// Turn a validated chunk into a card.
    fn into_card(self, header: &SourceHeader) -> GeneratedCard {
        let mut tags: Vec<String> = self
            .topics
            .iter()
            .map(|topic| format!("{TOPIC_TAG_PREFIX}{topic}"))
            .collect();
        tags.push(SOURCE_DERIVED_TAG.to_string());
        if self.neutral {
            tags.push(NEUTRAL_TEST_TAG.to_string());
        }

        GeneratedCard {
            front: self.prompt,
            back: self.answer,
            // the control arm never carries a cue, whatever the source offered.
            // The concept map stays: the render-time strip removes the cue and
            // never the mapping, and this must agree with it.
            whimsy: if self.neutral {
                String::new()
            } else {
                self.whimsy
            },
            concept_map: self.concept_map,
            source: format!("{}, chapter {}", header.book, self.chapter),
            tags,
        }
    }
}

/// Whether some AAMC outline category claims this deck topic key.
///
/// The outline is a static, ordered table, so this answer is the same on every
/// run and on every machine.
fn outline_knows(topic: &str) -> bool {
    outline()
        .iter()
        .any(|category| category.topics.iter().any(|known| *known == topic))
}

/// Build a deck from a source document.
///
/// Fails loudly, emitting nothing at all, when the document contains a chunk
/// that cannot be attributed to a book and chapter, a topic that matches no
/// AAMC outline category, or a whimsy cue with no concept mapping behind it.
pub(crate) fn generate_deck(source_path: &Path) -> Result<GeneratedDeck> {
    let text = anki_io::read_to_string(source_path)?;
    let blocks = parse_blocks(&text)?;
    let (header_block, chunk_blocks) = blocks
        .split_first()
        .or_invalid("source document is empty")?;

    let header = SourceHeader::parse(header_block)?;
    let mut cards: Vec<GeneratedCard> = Vec::with_capacity(chunk_blocks.len());
    for block in chunk_blocks {
        cards.push(SourceChunk::parse(block)?.into_card(&header));
    }
    if cards.is_empty() {
        invalid_input!("source document contains no chunks");
    }

    Ok(GeneratedDeck {
        field_names: NOTETYPE_FIELDS.iter().map(ToString::to_string).collect(),
        attribution: header.attribution,
        cards,
    })
}

// -------------------------------------------------------------------------
// writing the deck into a collection
// -------------------------------------------------------------------------

/// The notetype the generated deck uses, built from the five-field contract.
///
/// The cue and the mapping are wrapped in the classes ticket 006's render-time
/// strip keys off, taken from `mnemonic.rs` rather than spelled again here:
/// a cue the strip could not find would survive into a neutral test.
fn mcat_notetype() -> Notetype {
    let mut notetype = Notetype {
        name: NOTETYPE_NAME.to_string(),
        ..Default::default()
    };
    for field in NOTETYPE_FIELDS {
        notetype.add_field(field);
    }
    notetype.add_template(
        "Card 1",
        format!(
            "{{{{{FRONT_FIELD}}}}}\n\
             {{{{#{WHIMSY_FIELD}}}}}<div class=\"{WHIMSY_CUE_CLASS}\"><b>Cue:</b> \
             {{{{{WHIMSY_FIELD}}}}}</div>{{{{/{WHIMSY_FIELD}}}}}"
        ),
        format!(
            "{{{{FrontSide}}}}<hr id=answer>\n\
             {{{{{BACK_FIELD}}}}}\n\
             {{{{#{CONCEPT_MAP_FIELD}}}}}<div class=\"{CONCEPT_MAP_CLASS}\"><b>Why the cue \
             works:</b> {{{{{CONCEPT_MAP_FIELD}}}}}</div>{{{{/{CONCEPT_MAP_FIELD}}}}}\n\
             {{{{#{SOURCE_FIELD}}}}}<div class=\"source\">{{{{{SOURCE_FIELD}}}}}</div>\
             {{{{/{SOURCE_FIELD}}}}}"
        ),
    );
    notetype
}

impl Collection {
    /// Generate a deck from `source_path` and add it to the collection.
    ///
    /// Adds the notetype, the deck and one note per generated card. On failure
    /// the collection is left as it was found: a partially generated deck would
    /// be a partially fabricated one.
    pub(crate) fn generate_mcat_deck(
        &mut self,
        source_path: &Path,
        deck_name: &str,
    ) -> Result<GeneratedDeck> {
        // Everything is parsed and validated before the collection is touched
        // at all, so a document that fails cannot leave a partial deck behind.
        // The transaction below is the second line of defence, not the first.
        let generated = generate_deck(source_path)?;

        self.transact_no_undo(|col| {
            let notetype = col.mcat_notetype()?;
            let deck_id = col.mcat_deck_id(deck_name)?;
            for card in &generated.cards {
                let mut note = notetype.new_note();
                note.set_field(0, &card.front)?;
                note.set_field(1, &card.back)?;
                note.set_field(2, &card.whimsy)?;
                note.set_field(3, &card.concept_map)?;
                note.set_field(4, &card.source)?;
                note.tags = card.tags.clone();
                col.add_note_inner(&mut note, deck_id)?;
            }
            Ok(())
        })?;

        Ok(generated)
    }

    /// The generated notetype, created on first use.
    fn mcat_notetype(&mut self) -> Result<std::sync::Arc<Notetype>> {
        if let Some(existing) = self.get_notetype_by_name(NOTETYPE_NAME)? {
            return Ok(existing);
        }
        let usn = self.usn()?;
        let mut notetype = mcat_notetype();
        notetype.set_modified(usn);
        self.add_notetype_inner(&mut notetype, usn, false)?;
        self.get_notetype_by_name(NOTETYPE_NAME)?
            .or_not_found(NOTETYPE_NAME)
    }

    /// The deck the generated notes go into, created on first use.
    fn mcat_deck_id(&mut self, deck_name: &str) -> Result<DeckId> {
        if let Some(existing) = self.get_deck_id(deck_name)? {
            return Ok(existing);
        }
        let usn = self.usn()?;
        let mut deck = Deck::new_normal();
        deck.name = NativeDeckName::from_human_name(deck_name);
        self.add_deck_inner(&mut deck, usn)?;
        Ok(deck.id)
    }
}
