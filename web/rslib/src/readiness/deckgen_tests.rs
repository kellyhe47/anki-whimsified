// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 010 -- deck generation with enforced citations and topic tags.
//!
//! The load-bearing pair in this file is
//! [`deckgen_rejects_a_chunk_it_cannot_attribute`] and
//! [`deckgen_rejects_a_topic_that_matches_no_outline_category`]. This project's
//! whole claim is that it does not fabricate. An uncited card, or a tag
//! pointing at an invented category, is a fabricated measurement feeding the
//! coverage percentage and hence the give-up rule -- so both must stop
//! generation outright rather than warn and continue, and neither may leave a
//! half-built deck behind.
//!
//! The happy path runs against a vendored CC BY source document
//! (`testdata/mcat_source_excerpt.txt`). There is no corpus in the repo and no
//! network: the generator is handed a file path and nothing else.
//!
//! Nothing here pins card text or error strings. The assertions are about
//! shape, presence and rejection, so that the generator's wording stays free to
//! change.

use std::path::Path;
use std::path::PathBuf;

use tempfile::tempdir;
use tempfile::TempDir;

use crate::prelude::*;
use crate::readiness::data::aamc_outline::outline;
use crate::readiness::deckgen::generate_deck;
use crate::readiness::deckgen::GeneratedCard;
use crate::readiness::deckgen::GeneratedDeck;
use crate::readiness::deckgen::BACK_FIELD;
use crate::readiness::deckgen::FRONT_FIELD;
use crate::readiness::deckgen::NOTETYPE_FIELDS;
use crate::readiness::deckgen::NOTETYPE_NAME;
use crate::readiness::deckgen::SOURCE_FIELD;
use crate::readiness::evidence::TOPIC_TAG_PREFIX;
use crate::readiness::mnemonic::CONCEPT_MAP_FIELD;
use crate::readiness::mnemonic::NEUTRAL_TEST_TAG;
use crate::readiness::mnemonic::WHIMSY_FIELD;
use crate::readiness::provenance::provenance_of;
use crate::readiness::provenance::Provenance;
use crate::readiness::provenance::AI_GENERATED_TAG;
use crate::readiness::provenance::SOURCE_DERIVED_TAG;
use crate::revlog::RevlogEntry;
use crate::revlog::RevlogReviewKind;

// -------------------------------------------------------------------------
// fixtures
// -------------------------------------------------------------------------

/// The vendored CC BY source document, relative to the crate root.
const FIXTURE: &str = "src/readiness/testdata/mcat_source_excerpt.txt";

/// The work the fixture cites. Mirrored from the fixture's `book:` line;
/// [`deckgen_the_vendored_fixture_is_a_cited_cc_by_document`] catches drift.
const FIXTURE_BOOK: &str = "MCAT Foundations: An Open Excerpt";

/// The chapters the fixture's chunks cite, in source order.
const FIXTURE_CHAPTERS: [&str; 5] = [
    "1.2 Amino acids and the proteins they build",
    "1.4 Bioenergetics and the pacing of metabolism",
    "3.1 Circulation and the geometry of the vascular bed",
    "5.1 Water, acids and the behaviour of buffers",
    "7.3 Conditioning and the shaping of behaviour",
];

/// The topic keys the fixture's chunks name, in source order. Every one of them
/// belongs to a real outline category.
const FIXTURE_TOPICS: [&str; 5] = [
    "bb::amino_acids",
    "bb::bioenergetics",
    "bb::organ_systems",
    "cp::water",
    "ps::learning",
];

/// The deck name the collection-facing generator is asked for.
const DECK_NAME: &str = "MCAT Generated";

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

/// The header block every synthetic document below shares.
const HEADER: &str = "\
book: MCAT Foundations: An Open Excerpt
license: CC BY 4.0
attribution: \"MCAT Foundations: An Open Excerpt\" (2026) is licensed under CC BY 4.0.";

/// A chunk with everything the generator needs and nothing optional.
const PLAIN_CHUNK: &str = "\
chapter: 1.2 Amino acids and the proteins they build
topic: bb::amino_acids
prompt: What distinguishes one standard amino acid from another?
answer: Only the side chain.";

/// Assembles a source document from a header and some chunks.
fn document(header: &str, chunks: &[&str]) -> String {
    let mut out = header.to_string();
    for chunk in chunks {
        out.push_str("\n\n");
        out.push_str(chunk);
    }
    out.push('\n');
    out
}

/// Writes `text` to a file inside `dir` and returns its path.
fn write_source(dir: &TempDir, name: &str, text: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// Generates from the vendored fixture, failing the test loudly if it cannot.
#[track_caller]
fn generate_fixture_deck() -> GeneratedDeck {
    generate_deck(&fixture_path())
        .unwrap_or_else(|e| panic!("the vendored CC BY fixture must generate: {e:?}"))
}

/// The `mcat::` tags on a card.
fn topic_tags(card: &GeneratedCard) -> Vec<&str> {
    card.tags
        .iter()
        .map(String::as_str)
        .filter(|tag| tag.starts_with(TOPIC_TAG_PREFIX))
        .collect()
}

/// Whether some outline category claims this deck topic key.
fn outline_knows(topic: &str) -> bool {
    outline()
        .iter()
        .any(|category| category.topics.iter().any(|known| *known == topic))
}

/// Appends `count` graded revlog entries to a card.
fn add_reviews(col: &mut Collection, cid: CardId, count: usize) {
    for _ in 0..count {
        let entry = RevlogEntry {
            id: RevlogId::new(),
            cid,
            usn: Usn(-1),
            button_chosen: 3,
            interval: 1,
            last_interval: 1,
            ease_factor: 2500,
            taken_millis: 1000,
            review_kind: RevlogReviewKind::Review,
        };
        col.storage.add_revlog_entry(&entry, true).unwrap();
    }
}

/// Every note in the collection, as (field values, sorted tags).
fn note_shapes(col: &mut Collection) -> Vec<(Vec<String>, Vec<String>)> {
    let mut shapes: Vec<(Vec<String>, Vec<String>)> = col
        .storage
        .get_all_notes()
        .into_iter()
        .map(|note| {
            let mut tags = note.tags.clone();
            tags.sort();
            (note.fields().clone(), tags)
        })
        .collect();
    shapes.sort();
    shapes
}

// -------------------------------------------------------------------------
// the vendored source document
// -------------------------------------------------------------------------

/// The fixture really is what the rest of this file assumes: a document that
/// names its book, its chapters and a CC BY licence. If it drifts, the tests
/// below would still pass while testing something else.
#[test]
fn deckgen_the_vendored_fixture_is_a_cited_cc_by_document() {
    let path = fixture_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the vendored fixture must exist at {path:?}: {e}"));

    assert!(
        text.contains(&format!("book: {FIXTURE_BOOK}")),
        "the fixture must name the book the tests expect"
    );
    assert!(
        text.to_ascii_uppercase().contains("CC BY"),
        "the fixture must state its CC BY licence"
    );
    for chapter in FIXTURE_CHAPTERS {
        assert!(
            text.contains(chapter),
            "the fixture must still cite chapter {chapter:?}"
        );
    }
    for topic in FIXTURE_TOPICS {
        assert!(
            text.contains(topic),
            "the fixture must still name topic {topic:?}"
        );
        assert!(
            outline_knows(topic),
            "fixture topic {topic:?} is not an AAMC outline topic; \
             the happy-path tests would be testing the failure path"
        );
    }
}

// -------------------------------------------------------------------------
// the notetype contract
// -------------------------------------------------------------------------

/// The generated notetype has exactly the five contract fields, in order.
///
/// An earlier draft of this ticket named `WhimsyCue`, `Topic` and `NeutralTest`.
/// Ticket 006's contract is authoritative and this test pins it: whimsy lives in
/// `Whimsy` (the render-time strip keys off that exact name), the topic lives in
/// `mcat::` tags rather than a second, disagreeable field, and a neutral test
/// item is marked by a tag.
#[test]
fn deckgen_notetype_has_exactly_the_five_contract_fields() {
    assert_eq!(
        NOTETYPE_FIELDS.to_vec(),
        vec![
            FRONT_FIELD,
            BACK_FIELD,
            WHIMSY_FIELD,
            CONCEPT_MAP_FIELD,
            SOURCE_FIELD
        ],
        "the five-field contract changed"
    );

    let deck = generate_fixture_deck();
    assert_eq!(
        deck.field_names,
        NOTETYPE_FIELDS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        "the generated deck declared different fields"
    );

    // and the notetype actually written to a collection agrees
    let mut col = Collection::new();
    col.generate_mcat_deck(&fixture_path(), DECK_NAME).unwrap();
    let ntid = col
        .storage
        .get_notetype_id(NOTETYPE_NAME)
        .unwrap()
        .unwrap_or_else(|| panic!("generation must create the {NOTETYPE_NAME:?} notetype"));
    let notetype = col.storage.get_notetype(ntid).unwrap().unwrap();
    let names: Vec<&str> = notetype
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    assert_eq!(
        names,
        NOTETYPE_FIELDS.to_vec(),
        "the notetype in the collection has the wrong fields"
    );

    // the fields of the wrong draft must not exist under any name
    for rejected in ["WhimsyCue", "Topic", "NeutralTest"] {
        assert!(
            !names.contains(&rejected),
            "{rejected:?} is not part of the contract"
        );
    }
}

// -------------------------------------------------------------------------
// every card is cited
// -------------------------------------------------------------------------

/// Every generated card carries a non-empty `Source` naming the book and the
/// chapter it came from.
#[test]
fn deckgen_every_card_cites_its_book_and_chapter() {
    let deck = generate_fixture_deck();
    assert!(
        !deck.cards.is_empty(),
        "the fixture must produce cards, or nothing below is tested"
    );

    for (n, card) in deck.cards.iter().enumerate() {
        assert!(
            !card.source.trim().is_empty(),
            "card {n} has an empty Source"
        );
        assert!(
            card.source.contains(FIXTURE_BOOK),
            "card {n}'s Source does not name the book: {:?}",
            card.source
        );
        assert!(
            FIXTURE_CHAPTERS
                .iter()
                .any(|chapter| card.source.contains(chapter)),
            "card {n}'s Source does not name a chapter of the book: {:?}",
            card.source
        );
    }

    // every chapter of the fixture is cited by something, so the citation is
    // per-chunk rather than one book-wide string stamped on every card
    for chapter in FIXTURE_CHAPTERS {
        assert!(
            deck.cards.iter().any(|card| card.source.contains(chapter)),
            "no generated card cites chapter {chapter:?}"
        );
    }
}

// -------------------------------------------------------------------------
// every card is topic-tagged, and every topic is real
// -------------------------------------------------------------------------

/// Every generated card carries at least one well-formed
/// `mcat::<section>::<topic>` tag.
#[test]
fn deckgen_every_card_carries_a_well_formed_topic_tag() {
    let deck = generate_fixture_deck();

    for (n, card) in deck.cards.iter().enumerate() {
        let tags = topic_tags(card);
        assert!(
            !tags.is_empty(),
            "card {n} carries no {TOPIC_TAG_PREFIX} tag; tags {:?}",
            card.tags
        );
        for tag in tags {
            let rest = tag.strip_prefix(TOPIC_TAG_PREFIX).unwrap();
            let parts: Vec<&str> = rest.split("::").collect();
            assert!(
                parts.len() >= 2 && parts.iter().all(|part| !part.is_empty()),
                "card {n}: {tag:?} is not of the form {TOPIC_TAG_PREFIX}<section>::<topic>"
            );
        }
    }
}

/// Every topic a generated tag names resolves to a real AAMC outline category.
///
/// A tag the outline has never heard of is reported by `coverage_map` as an
/// unmapped topic -- it inflates nothing, but it also measures nothing. The
/// generator must never emit one.
#[test]
fn deckgen_every_generated_topic_resolves_to_an_outline_category() {
    let deck = generate_fixture_deck();

    for (n, card) in deck.cards.iter().enumerate() {
        for tag in topic_tags(card) {
            let topic = tag.strip_prefix(TOPIC_TAG_PREFIX).unwrap();
            assert!(
                outline_knows(topic),
                "card {n}: topic {topic:?} matches no AAMC outline category"
            );
        }
    }

    // and the generated deck is genuinely visible to the coverage map
    let mut col = Collection::new();
    col.generate_mcat_deck(&fixture_path(), DECK_NAME).unwrap();
    let coverage = col.coverage().unwrap();
    assert!(
        coverage.unmapped_topics.is_empty(),
        "generation produced topics the outline does not know: {:?}",
        coverage.unmapped_topics
    );
}

/// **Load-bearing.** A topic that matches no outline category fails generation
/// loudly. It does not warn, it does not drop the tag, and it does not emit the
/// card untagged.
#[test]
fn deckgen_rejects_a_topic_that_matches_no_outline_category() {
    let dir = tempdir().unwrap();

    // (case, topic line)
    let cases: Vec<(&str, &str)> = vec![
        ("an invented category number", "topic: bb::1z"),
        ("a plausible but unknown topic", "topic: bb::krebs_cycle"),
        ("a typo in a real topic", "topic: bb::amino_acid"),
        (
            "the wrong section for a real topic",
            "topic: cp::amino_acids",
        ),
        ("an empty topic", "topic:"),
        ("a topic from no section at all", "topic: enzymes"),
    ];

    for (case, topic_line) in cases {
        let chunk = format!(
            "chapter: 1.2 Amino acids and the proteins they build\n\
             {topic_line}\n\
             prompt: What distinguishes one standard amino acid from another?\n\
             answer: Only the side chain."
        );
        let text = document(HEADER, &[&chunk]);
        let path = write_source(&dir, &format!("{}.txt", case.replace(' ', "_")), &text);

        assert!(
            generate_deck(&path).is_err(),
            "{case}: {topic_line:?} must fail generation, not be tolerated"
        );
    }

    // the same chunk with a real topic generates, so the rejections above are
    // about the topic and not about the chunk being malformed
    let path = write_source(&dir, "control.txt", &document(HEADER, &[PLAIN_CHUNK]));
    assert!(
        generate_deck(&path).is_ok(),
        "the control chunk, whose topic is real, must generate"
    );
}

// -------------------------------------------------------------------------
// whimsy, concept maps and neutral test items
// -------------------------------------------------------------------------

/// A card tagged `neutral-test` has an empty `Whimsy` field, even though the
/// source chunk it came from supplied a cue. The neutral condition is the
/// control arm of the whimsy experiment; a cue that survived into the field
/// would contaminate it.
#[test]
fn deckgen_a_neutral_test_card_has_no_whimsy() {
    let deck = generate_fixture_deck();

    let neutral: Vec<&GeneratedCard> = deck
        .cards
        .iter()
        .filter(|card| card.tags.iter().any(|tag| tag == NEUTRAL_TEST_TAG))
        .collect();
    assert!(
        !neutral.is_empty(),
        "the fixture must produce a neutral test item, or this proves nothing"
    );

    for card in &neutral {
        assert!(
            card.whimsy.is_empty(),
            "a neutral-test card kept a whimsy cue: {:?}",
            card.whimsy
        );
    }

    // the fixture's neutral chunk does supply a cue, so emptying it is work the
    // generator did rather than an absence it inherited
    let source = std::fs::read_to_string(fixture_path()).unwrap();
    assert!(
        source.contains("neutral: yes") && source.contains("whimsy:"),
        "the fixture's neutral chunk must supply a cue for the generator to drop"
    );

    // the tag is not mcat::-prefixed, so it can never be read as a topic
    assert!(!NEUTRAL_TEST_TAG.starts_with(TOPIC_TAG_PREFIX));
    for card in &neutral {
        assert!(
            !topic_tags(card).is_empty(),
            "a neutral-test card still needs its topic tag"
        );
    }
}

/// A card with a non-empty `Whimsy` also has a non-empty `ConceptMap`, and a
/// source chunk offering a cue with no mapping is rejected.
///
/// Mapping is the treatment and decoration is the control: whimsy without an
/// explicit concept mapping is decoration wearing the treatment's clothes.
#[test]
fn deckgen_whimsy_without_a_concept_map_is_rejected() {
    let deck = generate_fixture_deck();
    assert!(
        deck.cards.iter().any(|card| !card.whimsy.is_empty()),
        "the fixture must produce at least one card with a cue"
    );
    for (n, card) in deck.cards.iter().enumerate() {
        if !card.whimsy.is_empty() {
            assert!(
                !card.concept_map.trim().is_empty(),
                "card {n} has a whimsy cue and no concept map"
            );
        }
    }

    let dir = tempdir().unwrap();
    // (case, extra lines appended to a plain chunk, generation succeeds)
    let cases: Vec<(&str, &str, bool)> = vec![
        ("no cue at all", "", true),
        (
            "cue with a mapping",
            "whimsy: A turnstile, not a door.\nconcept-map: turnstile -> the committed step.",
            true,
        ),
        (
            "cue with no mapping",
            "whimsy: A turnstile, not a door.",
            false,
        ),
        (
            "cue with an empty mapping",
            "whimsy: A turnstile, not a door.\nconcept-map:",
            false,
        ),
    ];

    for (case, extra, ok) in cases {
        let chunk = if extra.is_empty() {
            PLAIN_CHUNK.to_string()
        } else {
            format!("{PLAIN_CHUNK}\n{extra}")
        };
        let text = document(HEADER, &[&chunk]);
        let path = write_source(&dir, &format!("{}.txt", case.replace(' ', "_")), &text);
        assert_eq!(
            generate_deck(&path).is_ok(),
            ok,
            "{case}: wrong verdict for a cue/mapping combination"
        );
    }
}

// -------------------------------------------------------------------------
// the deck carries its licence
// -------------------------------------------------------------------------

/// The deck carries the source document's CC-BY attribution.
#[test]
fn deckgen_deck_carries_cc_by_attribution() {
    let deck = generate_fixture_deck();
    assert!(
        !deck.attribution.trim().is_empty(),
        "the generated deck carries no attribution"
    );
    assert!(
        deck.attribution.to_ascii_uppercase().contains("CC BY"),
        "the attribution does not name the CC BY licence: {:?}",
        deck.attribution
    );
    assert!(
        deck.attribution.contains(FIXTURE_BOOK),
        "the attribution does not name the work it attributes: {:?}",
        deck.attribution
    );
}

// -------------------------------------------------------------------------
// determinism
// -------------------------------------------------------------------------

/// Same input, identical cards. A generator whose output wandered would make
/// every measurement downstream of it unreproducible.
#[test]
fn deckgen_is_deterministic() {
    let first = generate_fixture_deck();
    let second = generate_fixture_deck();
    assert_eq!(first, second, "two runs over one document differed");

    // and the same document at a different path is still the same deck
    let dir = tempdir().unwrap();
    let text = std::fs::read_to_string(fixture_path()).unwrap();
    let copied = write_source(&dir, "copy_of_excerpt.txt", &text);
    let third = generate_deck(&copied).unwrap();
    assert_eq!(
        first, third,
        "generation depended on the file's path rather than its contents"
    );

    // down to what lands in a collection
    let mut one = Collection::new();
    let mut two = Collection::new();
    one.generate_mcat_deck(&fixture_path(), DECK_NAME).unwrap();
    two.generate_mcat_deck(&fixture_path(), DECK_NAME).unwrap();
    assert_eq!(
        note_shapes(&mut one),
        note_shapes(&mut two),
        "two collections generated from one document hold different notes"
    );
}

// -------------------------------------------------------------------------
// failing loudly
// -------------------------------------------------------------------------

/// **Load-bearing.** A source chunk the generator cannot attribute fails
/// generation. It never emits an uncited card.
#[test]
fn deckgen_rejects_a_chunk_it_cannot_attribute() {
    let dir = tempdir().unwrap();

    let no_chapter = "\
topic: bb::amino_acids
prompt: What distinguishes one standard amino acid from another?
answer: Only the side chain.";
    let blank_chapter = "\
chapter:
topic: bb::amino_acids
prompt: What distinguishes one standard amino acid from another?
answer: Only the side chain.";
    let no_book = "\
license: CC BY 4.0
attribution: licensed under CC BY 4.0.";
    let empty_book = "\
book:
license: CC BY 4.0
attribution: licensed under CC BY 4.0.";

    // (case, header, chunks)
    let cases: Vec<(&str, &str, Vec<&str>)> = vec![
        ("chunk with no chapter", HEADER, vec![no_chapter]),
        ("chunk with an empty chapter", HEADER, vec![blank_chapter]),
        ("document that names no book", no_book, vec![PLAIN_CHUNK]),
        (
            "document whose book is empty",
            empty_book,
            vec![PLAIN_CHUNK],
        ),
        ("document with no header at all", "", vec![PLAIN_CHUNK]),
        (
            "one bad chunk among good ones",
            HEADER,
            vec![PLAIN_CHUNK, no_chapter, PLAIN_CHUNK],
        ),
    ];

    for (case, header, chunks) in cases {
        let text = document(header, &chunks);
        let path = write_source(&dir, &format!("{}.txt", case.replace(' ', "_")), &text);
        assert!(
            generate_deck(&path).is_err(),
            "{case}: an unattributable chunk must fail generation, not be skipped"
        );
    }
}

/// A failed generation emits nothing at all: no notetype, no deck, no notes.
/// "One bad chunk among good ones" is the case that matters -- a generator that
/// wrote the good cards first and then errored would have shipped a deck whose
/// missing chapter nobody can see.
#[test]
fn deckgen_a_failed_generation_leaves_the_collection_untouched() {
    let dir = tempdir().unwrap();
    let bad_chunk = "\
chapter: 4.9 An uncitable aside
topic: bb::not_a_real_topic
prompt: Why is this card fabricated?
answer: Because its topic names no outline category.";
    let text = document(HEADER, &[PLAIN_CHUNK, bad_chunk, PLAIN_CHUNK]);
    let path = write_source(&dir, "partly_bad.txt", &text);

    let mut col = Collection::new();
    let notes_before = col.storage.get_all_notes().len();

    assert!(
        col.generate_mcat_deck(&path, DECK_NAME).is_err(),
        "a document with a bad chunk must fail generation"
    );
    assert_eq!(
        col.storage.get_all_notes().len(),
        notes_before,
        "a failed generation left notes behind"
    );
    assert!(
        col.get_deck_id(DECK_NAME).unwrap().is_none(),
        "a failed generation left a deck behind"
    );
    assert!(
        col.coverage().unwrap().unmapped_topics.is_empty(),
        "a failed generation leaked an unmapped topic into the coverage map"
    );
}

/// A source path that does not exist is an error, not a panic and not an empty
/// deck.
#[test]
fn deckgen_a_missing_source_document_is_an_error() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no_such_document.txt");
    assert!(
        generate_deck(&missing).is_err(),
        "a missing source document must be reported as an error"
    );

    let mut col = Collection::new();
    assert!(
        col.generate_mcat_deck(&missing, DECK_NAME).is_err(),
        "a missing source document must be reported as an error"
    );
}

// -------------------------------------------------------------------------
// provenance: generated is source-derived, and source-derived is still evidence
// -------------------------------------------------------------------------

/// Cards built from a real source document are source-derived, never
/// AI-generated. The distinction is the whole reason ticket 008 has three
/// provenance arms rather than two.
#[test]
fn deckgen_generated_cards_are_source_derived_not_ai_generated() {
    let deck = generate_fixture_deck();
    for (n, card) in deck.cards.iter().enumerate() {
        assert!(
            card.tags.iter().any(|tag| tag == SOURCE_DERIVED_TAG),
            "card {n} is missing the source-derived marker; tags {:?}",
            card.tags
        );
        assert!(
            !card.tags.iter().any(|tag| tag == AI_GENERATED_TAG),
            "card {n} claims AI provenance for transcribed source material"
        );
        assert_eq!(
            provenance_of(&card.tags),
            Provenance::SourceDerived,
            "card {n} has the wrong provenance"
        );
    }
}

/// GUARD (ticket 008). Source-derived cards still count as evidence. The
/// firewall excludes `ai-generated` and nothing else; this pins the two apart so
/// a later change cannot quietly lump them together and silently zero the
/// coverage of every generated deck.
#[test]
fn deckgen_generated_cards_still_count_as_evidence() {
    let mut col = Collection::new();
    let deck = col.generate_mcat_deck(&fixture_path(), DECK_NAME).unwrap();
    assert!(!deck.cards.is_empty());

    let cids: Vec<CardId> = col
        .storage
        .get_all_cards()
        .into_iter()
        .map(|card| card.id)
        .collect();
    assert!(!cids.is_empty(), "generation produced no cards to review");
    for cid in cids {
        add_reviews(&mut col, cid, 3);
    }

    let evidence = col.topic_evidence().unwrap();
    let total: u32 = evidence.iter().map(|row| row.graded_reviews).sum();
    assert!(
        total > 0,
        "reviews of source-derived cards were excluded from the evidence"
    );

    for topic in FIXTURE_TOPICS {
        let row = evidence
            .iter()
            .find(|row| row.topic == topic)
            .unwrap_or_else(|| panic!("no evidence row for generated topic {topic:?}"));
        assert!(
            row.cards_total > 0,
            "{topic}: generated cards were not counted"
        );
        assert!(
            row.graded_reviews > 0,
            "{topic}: reviews of generated cards were not counted"
        );
    }

    let coverage = col.coverage().unwrap();
    assert!(
        coverage.coverage_pct > 0.0,
        "a studied, source-derived generated deck covered nothing"
    );
}
