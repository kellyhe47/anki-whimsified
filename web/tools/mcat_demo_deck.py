#!/usr/bin/env python3
"""Build the MCAT demo deck: two cards with genuine concept-mapped whimsy cues.

The product thesis is that whimsy is a memory technology *only when every
memorable element maps to the scientific structure*. Harp & Mayer (1998) is the
counter-evidence: interesting but irrelevant detail reduces recall and transfer.
So each card here carries an explicit ConceptMap field naming what maps to what.
A cue whose elements do not map is decoration, and decoration is the control
condition, not the treatment.

Fields match the contract in rslib/src/readiness/mnemonic.rs:
    Whimsy      -> rendered inside .whimsy-cue, stripped when whimsy is disabled
    ConceptMap  -> rendered inside .concept-map, never stripped with the cue

Usage:
    out/pyenv/bin/python tools/mcat_demo_deck.py [collection.anki2]
"""

import sys
from pathlib import Path

from anki.collection import Collection

NOTETYPE = "MCAT Whimsified"
DECK = "MCAT Demo"

FRONT_TEMPLATE = """{{Front}}
{{#Whimsy}}<div class="whimsy-cue"><b>Cue:</b> {{Whimsy}}</div>{{/Whimsy}}"""

BACK_TEMPLATE = """{{FrontSide}}<hr id=answer>
{{Back}}
{{#ConceptMap}}<div class="concept-map"><b>Why the cue works:</b> {{ConceptMap}}</div>{{/ConceptMap}}
{{#Source}}<div class="source">{{Source}}</div>{{/Source}}"""

CSS = """.card { font-family: -apple-system, system-ui, sans-serif; font-size: 19px;
  text-align: left; color: #e8e8e8; background: #1e1e1e; padding: 18px; line-height: 1.5; }
.whimsy-cue { margin-top: 14px; padding: 10px 12px; border-left: 3px solid #d68b3c;
  background: #2b2418; border-radius: 4px; font-style: italic; }
.concept-map { margin-top: 14px; padding: 10px 12px; border-left: 3px solid #4c8fbd;
  background: #1a2430; border-radius: 4px; font-size: 16px; }
.source { margin-top: 12px; font-size: 13px; color: #888; }
hr#answer { margin: 18px 0; border: none; border-top: 1px solid #444; }"""

CARDS = [
    {
        "Front": "How do competitive and noncompetitive inhibitors differ in "
        "their effect on K<sub>m</sub> and V<sub>max</sub>?",
        "Back": "<b>Competitive</b> binds the active site: K<sub>m</sub> "
        "<b>increases</b>, V<sub>max</sub> is <b>unchanged</b> (excess substrate "
        "outcompetes it).<br><br><b>Noncompetitive</b> binds an allosteric site: "
        "V<sub>max</sub> <b>decreases</b>, K<sub>m</sub> is <b>unchanged</b> "
        "(more substrate cannot help).",
        "Whimsy": "A competitive inhibitor is a queue-jumper at the box office. "
        "Send a big enough crowd and every real fan still gets in — the theatre "
        "holds exactly as many as before, you just needed a longer queue. "
        "A noncompetitive inhibitor is someone quietly unbolting seats at the "
        "back. No size of crowd fixes that; the theatre simply holds fewer now, "
        "and how eagerly people queue was never the problem.",
        "ConceptMap": "queue-jumper &rarr; inhibitor occupying the active site &middot; "
        "bigger crowd &rarr; raised [substrate] &middot; every fan still gets in &rarr; "
        "V<sub>max</sub> unchanged &middot; longer queue needed &rarr; K<sub>m</sub> "
        "increased &middot; unbolting seats &rarr; allosteric change lowering catalytic "
        "capacity &rarr; V<sub>max</sub> falls &middot; eagerness to queue unaffected "
        "&rarr; K<sub>m</sub> unchanged. Every element of the image is one variable "
        "in the kinetics; nothing in it is decorative.",
        "Source": "Content category 1A — structure and function of proteins and "
        "their constituent amino acids",
        "tags": ["mcat::bb::1a", "mcat::bb::proteins"],
    },
    {
        "Front": "Which factors shift the oxygen&ndash;hemoglobin dissociation "
        "curve to the right, and what does a right shift mean physiologically?",
        "Back": "Right shift = <b>decreased</b> O<sub>2</sub> affinity = more "
        "O<sub>2</sub> <b>released</b> to tissue.<br><br>Caused by &uarr; "
        "CO<sub>2</sub>, &uarr; H<sup>+</sup> (&darr; pH), &uarr; temperature, "
        "&uarr; 2,3-BPG.",
        "Whimsy": "Hemoglobin is a courier who only lets go of the parcel "
        "somewhere hot, sour and out of breath &mdash; because that is where the "
        "work is happening. Every condition that makes it drop the parcel is a "
        "sign of a muscle already labouring. <b>Right means release.</b>",
        "ConceptMap": "parcel &rarr; bound O<sub>2</sub> &middot; letting go &rarr; "
        "decreased affinity &middot; hot &rarr; raised temperature from exertion &middot; "
        "sour &rarr; H<sup>+</sup> and lowered pH &middot; out of breath &rarr; raised "
        "CO<sub>2</sub> &middot; labouring muscle &rarr; 2,3-BPG from sustained "
        "oxygen demand. The image is not a mood: each of its four sensations is one "
        "of the four right-shift factors, and 'right means release' names the "
        "direction of the curve and its consequence together.",
        "Source": "Content category 3B — structure and integrative functions of "
        "the main organ systems",
        "tags": ["mcat::bb::3b", "mcat::bb::organ_systems"],
    },
    {
        "Front": "In glycolysis, which enzyme is the main rate-limiting step, "
        "and what activates or inhibits it?",
        "Back": "<b>Phosphofructokinase-1 (PFK-1)</b>.<br><br>"
        "<b>Inhibited</b> by ATP and citrate (the cell is already rich).<br>"
        "<b>Activated</b> by AMP and fructose-2,6-bisphosphate (the cell is "
        "running low).",
        "Whimsy": "PFK-1 is the turnstile at the entrance, not the door. The "
        "door lets anyone in; the turnstile decides the rate. It reads the room "
        "before turning: a room already full of energy (ATP) and leftovers "
        "(citrate) makes it stiffen, while an empty, hungry room (AMP) makes it "
        "spin freely.",
        "ConceptMap": "turnstile rather than door &rarr; the committed, "
        "flux-controlling step rather than mere entry &middot; rate of turning "
        "&rarr; reaction velocity &middot; room already full of energy &rarr; high "
        "[ATP] &middot; leftovers &rarr; citrate signalling a stocked citric acid "
        "cycle &middot; empty hungry room &rarr; high [AMP] signalling depletion. "
        "The turnstile is chosen over a door precisely because a door is binary "
        "and a rate-limiting enzyme is not.",
        "Source": "Content category 1D — principles of bioenergetics and fuel "
        "molecule metabolism",
        "tags": ["mcat::bb::1d", "mcat::bb::glycolysis"],
    },
    {
        "Front": "What does a buffer do, and at what pH is it strongest?",
        "Back": "A buffer resists pH change by converting strong acid or base "
        "into weak.<br><br>It is <b>strongest when pH = pK<sub>a</sub></b>, where "
        "[A<sup>&minus;</sup>] = [HA] &mdash; equal reserves to absorb a push in "
        "either direction.<br><br>pH = pK<sub>a</sub> + log([A<sup>&minus;</sup>]/[HA])",
        "Whimsy": "A buffer is a seesaw with sandbags piled at both ends. Push "
        "down on either side and a sandbag shifts to absorb it. It resists best "
        "when the piles are equal &mdash; that balance point is pK<sub>a</sub>. "
        "Load one end and it still works, but a shove from the light side now "
        "tips it easily.",
        "ConceptMap": "sandbags at each end &rarr; the reserves of conjugate base "
        "and weak acid &middot; a push &rarr; added strong acid or base &middot; a "
        "sandbag shifting &rarr; conversion between HA and A<sup>&minus;</sup> "
        "&middot; equal piles &rarr; [A<sup>&minus;</sup>] = [HA], the point where "
        "the log term is zero and pH = pK<sub>a</sub> &middot; tipping easily when "
        "lopsided &rarr; falling buffer capacity away from pK<sub>a</sub>. The "
        "seesaw encodes why capacity is maximal at the midpoint, not merely that "
        "it is.",
        "Source": "Content category 5A — unique nature of water and its solutions",
        "tags": ["mcat::cp::5a", "mcat::cp::acids_and_bases"],
    },
    {
        "Front": "Why does blood pressure drop as blood moves from the aorta "
        "into the capillary bed, even though the capillaries are far narrower?",
        "Back": "Because <b>total cross-sectional area</b> rises enormously: "
        "billions of capillaries in parallel far exceed the aorta's area.<br><br>"
        "By continuity (A&middot;v = constant), velocity <b>falls</b>. Slow flow "
        "plus huge area means low pressure &mdash; and slow transit is exactly "
        "what gas exchange needs.",
        "Whimsy": "One motorway feeding ten thousand village lanes. Each lane is "
        "narrower than the motorway, yet all ten thousand together are far wider, "
        "so the traffic crawls. That crawl is the point: nobody unloads a delivery "
        "at speed.",
        "ConceptMap": "motorway &rarr; aorta &middot; one lane narrower &rarr; a "
        "single capillary's small radius &middot; ten thousand lanes together far "
        "wider &rarr; total cross-sectional area of the capillary bed &middot; "
        "traffic crawling &rarr; reduced flow velocity by continuity &middot; "
        "unloading deliveries &rarr; gas and nutrient exchange. The image exists "
        "to separate <i>individual</i> radius from <i>aggregate</i> area, which is "
        "the exact confusion the question tests.",
        "Source": "Content category 4B — importance of fluids for the circulation "
        "of blood, gas movement, and gas exchange",
        "tags": ["mcat::cp::4b", "mcat::cp::fluids"],
    },
    {
        "Front": "Distinguish classical from operant conditioning.",
        "Back": "<b>Classical</b>: an involuntary response is drawn to a new "
        "stimulus by pairing (Pavlov). The learner is passive.<br><br>"
        "<b>Operant</b>: a voluntary behaviour is shaped by its consequences "
        "(Skinner). The learner acts first.",
        "Whimsy": "Classical conditioning is a doorbell you learn to salivate at. "
        "Operant conditioning is a vending machine you learn to kick. In the first "
        "the world acts on you and your body answers; in the second you act on the "
        "world and it answers back.",
        "ConceptMap": "doorbell &rarr; a neutral stimulus paired until it is "
        "conditioned &middot; salivating &rarr; an involuntary reflexive response "
        "&middot; the world acting first &rarr; the passive learner of classical "
        "conditioning &middot; kicking the machine &rarr; an emitted voluntary "
        "behaviour &middot; the machine answering &rarr; reinforcement or "
        "punishment as consequence &middot; you acting first &rarr; the active "
        "learner of operant conditioning. Which party moves first is the "
        "discriminating feature, and it is what the image encodes.",
        "Source": "Content category 7A — individual influences on behavior",
        "tags": ["mcat::ps::7a", "mcat::ps::learning"],
    },
]


def build(col: Collection) -> None:
    mm = col.models
    nt = mm.by_name(NOTETYPE)
    if nt is None:
        nt = mm.new(NOTETYPE)
        for field in ("Front", "Back", "Whimsy", "ConceptMap", "Source"):
            mm.add_field(nt, mm.new_field(field))
        tmpl = mm.new_template("Card 1")
        tmpl["qfmt"] = FRONT_TEMPLATE
        tmpl["afmt"] = BACK_TEMPLATE
        mm.add_template(nt, tmpl)
        nt["css"] = CSS
        mm.add(nt)
        nt = mm.by_name(NOTETYPE)
        print(f"created notetype {NOTETYPE!r}")

    deck_id = col.decks.id(DECK)

    for spec in CARDS:
        note = col.new_note(nt)
        for field in ("Front", "Back", "Whimsy", "ConceptMap", "Source"):
            note[field] = spec[field]
        note.tags = list(spec["tags"])
        col.add_note(note, deck_id)
        print(f"added card tagged {' '.join(spec['tags'])}")

    col.save()


def main() -> None:
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    if target is None:
        print(__doc__)
        raise SystemExit("error: pass the path to a collection.anki2")
    col = Collection(str(target))
    try:
        build(col)
    finally:
        col.close()
    print(f"done: {target}")


if __name__ == "__main__":
    main()
