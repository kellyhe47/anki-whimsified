// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! The official AAMC MCAT content outline, as static data.
//!
//! This is a compiled-in table: there is no network access and no runtime
//! download. The outline is roughly thirty content categories spread across the
//! four sections of the exam.

/// The four sections of the MCAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OutlineSection {
    /// Biological and Biochemical Foundations of Living Systems.
    BioBiochem,
    /// Chemical and Physical Foundations of Biological Systems.
    ChemPhys,
    /// Psychological, Social, and Biological Foundations of Behavior.
    PsychSoc,
    /// Critical Analysis and Reasoning Skills.
    ///
    /// The AAMC publishes no *content* categories for this section: CARS is
    /// specified as three skills -- Foundations of Comprehension, Reasoning
    /// Within the Text, and Reasoning Beyond the Text. Those three skills are
    /// modelled here as categories so that the section is represented at all.
    /// They are skills, not content categories, and no others exist: content
    /// categories must never be invented for CARS, because a coverage figure
    /// measured against a fabricated outline is a fabricated measurement.
    Cars,
}

impl OutlineSection {
    /// Every section, for exhaustive iteration.
    pub(crate) const ALL: [OutlineSection; 4] = [
        OutlineSection::BioBiochem,
        OutlineSection::ChemPhys,
        OutlineSection::PsychSoc,
        OutlineSection::Cars,
    ];

    /// The section's name, for showing to a learner. These are the AAMC's own
    /// section names, abbreviated only where the full title would not fit a
    /// table row; nothing here is invented.
    pub(crate) fn label(self) -> &'static str {
        match self {
            OutlineSection::BioBiochem => "Biological and Biochemical Foundations",
            OutlineSection::ChemPhys => "Chemical and Physical Foundations",
            OutlineSection::PsychSoc => "Psychological, Social, and Biological Foundations",
            OutlineSection::Cars => "Critical Analysis and Reasoning Skills",
        }
    }
}

/// One content category of the outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OutlineCategory {
    /// Stable identifier, eg `bb::1a`. Unique across the outline.
    pub id: &'static str,
    /// The category's name as the AAMC writes it, for showing to the learner.
    pub name: &'static str,
    /// Which section of the exam the category belongs to.
    pub section: OutlineSection,
    /// The deck topic keys -- the part of an `mcat::` tag after the prefix --
    /// that belong to this category. A deck topic listed here maps to this
    /// category; a deck topic listed nowhere is unmapped.
    pub topics: &'static [&'static str],
}

/// Shorthand for one row of the table below.
const fn category(
    id: &'static str,
    name: &'static str,
    section: OutlineSection,
    topics: &'static [&'static str],
) -> OutlineCategory {
    OutlineCategory {
        id,
        name,
        section,
        topics,
    }
}

/// The published outline: 31 content categories across the three science
/// sections, plus the three CARS skills. Nothing here is invented -- the ids
/// and names are the AAMC's own.
///
/// The `topics` of each category are deck-side keys, not AAMC data: the
/// section-qualified category key (`bb::1a`, matching the
/// `mcat::<section>::<topic>` tag convention), its bare suffix (`1a`, for decks
/// that tag by category number alone), and a couple of conventional topic names
/// a deck is likely to use. Adding a key here only ever widens what maps; it
/// never changes the denominator.
const OUTLINE: &[OutlineCategory] = &[
    // -- Biological and Biochemical Foundations of Living Systems -----------
    category(
        "bb::1a",
        "Structure and function of proteins and their constituent amino acids",
        OutlineSection::BioBiochem,
        &["bb::1a", "1a", "bb::amino_acids", "bb::proteins"],
    ),
    category(
        "bb::1b",
        "Transmission of genetic information from the gene to the protein",
        OutlineSection::BioBiochem,
        &["bb::1b", "1b", "bb::transcription", "bb::translation"],
    ),
    category(
        "bb::1c",
        "Transmission of heritable information from generation to generation and the processes \
         that increase genetic diversity",
        OutlineSection::BioBiochem,
        &["bb::1c", "1c", "bb::meiosis", "bb::genetic_diversity"],
    ),
    category(
        "bb::1d",
        "Principles of bioenergetics and fuel molecule metabolism",
        OutlineSection::BioBiochem,
        &["bb::1d", "1d", "bb::bioenergetics", "bb::metabolism"],
    ),
    category(
        "bb::2a",
        "Assemblies of molecules, cells, and groups of cells within single cellular and \
         multicellular organisms",
        OutlineSection::BioBiochem,
        &["bb::2a", "2a", "bb::cell_membranes", "bb::eukaryotic_cells"],
    ),
    category(
        "bb::2b",
        "The structure, growth, physiology, and genetics of prokaryotes and viruses",
        OutlineSection::BioBiochem,
        &["bb::2b", "2b", "bb::prokaryotes", "bb::viruses"],
    ),
    category(
        "bb::2c",
        "Processes of cell division, differentiation, and specialization",
        OutlineSection::BioBiochem,
        &["bb::2c", "2c", "bb::mitosis", "bb::differentiation"],
    ),
    category(
        "bb::3a",
        "Structure and functions of the nervous and endocrine systems and ways they coordinate \
         the organ systems",
        OutlineSection::BioBiochem,
        &["bb::3a", "3a", "bb::nervous_system", "bb::endocrine_system"],
    ),
    category(
        "bb::3b",
        "Structure and integrative functions of the main organ systems",
        OutlineSection::BioBiochem,
        &[
            "bb::3b",
            "3b",
            "bb::organ_systems",
            "bb::circulatory_system",
        ],
    ),
    // -- Chemical and Physical Foundations of Biological Systems ------------
    category(
        "cp::4a",
        "Translational motion, forces, work, energy, and equilibrium in living systems",
        OutlineSection::ChemPhys,
        &["cp::4a", "4a", "cp::kinematics", "cp::work_and_energy"],
    ),
    category(
        "cp::4b",
        "Importance of fluids for the circulation of blood, gas movement, and gas exchange",
        OutlineSection::ChemPhys,
        &["cp::4b", "4b", "cp::fluids", "cp::gas_exchange"],
    ),
    category(
        "cp::4c",
        "Electrochemistry and electrical circuits and their elements",
        OutlineSection::ChemPhys,
        &["cp::4c", "4c", "cp::electrochemistry", "cp::circuits"],
    ),
    category(
        "cp::4d",
        "How light and sound interact with matter",
        OutlineSection::ChemPhys,
        &["cp::4d", "4d", "cp::light", "cp::sound"],
    ),
    category(
        "cp::4e",
        "Atoms, nuclear decay, electronic structure, and atomic chemical behavior",
        OutlineSection::ChemPhys,
        &["cp::4e", "4e", "cp::atomic_structure", "cp::nuclear_decay"],
    ),
    category(
        "cp::5a",
        "Unique nature of water and its solutions",
        OutlineSection::ChemPhys,
        &["cp::5a", "5a", "cp::water", "cp::acids_and_bases"],
    ),
    category(
        "cp::5b",
        "Nature of molecules and intermolecular interactions",
        OutlineSection::ChemPhys,
        &[
            "cp::5b",
            "5b",
            "cp::bonding",
            "cp::intermolecular_interactions",
        ],
    ),
    category(
        "cp::5c",
        "Separation and purification methods",
        OutlineSection::ChemPhys,
        &["cp::5c", "5c", "cp::separations", "cp::chromatography"],
    ),
    category(
        "cp::5d",
        "Structure, function, and reactivity of biologically relevant molecules",
        OutlineSection::ChemPhys,
        &["cp::5d", "5d", "cp::organic_chemistry", "cp::reactivity"],
    ),
    category(
        "cp::5e",
        "Principles of chemical thermodynamics and kinetics",
        OutlineSection::ChemPhys,
        &["cp::5e", "5e", "cp::thermodynamics", "cp::kinetics"],
    ),
    // -- Psychological, Social, and Biological Foundations of Behavior ------
    category(
        "ps::6a",
        "Sensing the environment",
        OutlineSection::PsychSoc,
        &["ps::6a", "6a", "ps::sensation", "ps::perception"],
    ),
    category(
        "ps::6b",
        "Making sense of the environment",
        OutlineSection::PsychSoc,
        &["ps::6b", "6b", "ps::memory", "ps::cognition"],
    ),
    category(
        "ps::6c",
        "Responding to the world",
        OutlineSection::PsychSoc,
        &["ps::6c", "6c", "ps::emotion", "ps::stress"],
    ),
    category(
        "ps::7a",
        "Individual influences on behavior",
        OutlineSection::PsychSoc,
        &["ps::7a", "7a", "ps::personality", "ps::motivation"],
    ),
    category(
        "ps::7b",
        "Social processes that influence human behavior",
        OutlineSection::PsychSoc,
        &["ps::7b", "7b", "ps::group_behavior", "ps::socialization"],
    ),
    category(
        "ps::7c",
        "Attitude and behavior change",
        OutlineSection::PsychSoc,
        &["ps::7c", "7c", "ps::learning", "ps::attitude_change"],
    ),
    category(
        "ps::8a",
        "Self-identity",
        OutlineSection::PsychSoc,
        &[
            "ps::8a",
            "8a",
            "ps::self_identity",
            "ps::identity_formation",
        ],
    ),
    category(
        "ps::8b",
        "Social thinking",
        OutlineSection::PsychSoc,
        &["ps::8b", "8b", "ps::attribution", "ps::prejudice"],
    ),
    category(
        "ps::8c",
        "Social interactions",
        OutlineSection::PsychSoc,
        &[
            "ps::8c",
            "8c",
            "ps::social_interaction",
            "ps::self_presentation",
        ],
    ),
    category(
        "ps::9a",
        "Understanding social structure",
        OutlineSection::PsychSoc,
        &[
            "ps::9a",
            "9a",
            "ps::social_institutions",
            "ps::social_structure",
        ],
    ),
    category(
        "ps::9b",
        "Demographic characteristics and processes",
        OutlineSection::PsychSoc,
        &[
            "ps::9b",
            "9b",
            "ps::demographics",
            "ps::population_dynamics",
        ],
    ),
    category(
        "ps::10a",
        "Social inequality",
        OutlineSection::PsychSoc,
        &[
            "ps::10a",
            "10a",
            "ps::social_inequality",
            "ps::health_disparities",
        ],
    ),
    // -- Critical Analysis and Reasoning Skills -----------------------------
    // Skills, not content categories. See `OutlineSection::Cars`.
    category(
        "cars::foundations",
        "Foundations of Comprehension",
        OutlineSection::Cars,
        &["cars::foundations", "foundations", "cars::comprehension"],
    ),
    category(
        "cars::within",
        "Reasoning Within the Text",
        OutlineSection::Cars,
        &["cars::within", "within", "cars::reasoning_within_text"],
    ),
    category(
        "cars::beyond",
        "Reasoning Beyond the Text",
        OutlineSection::Cars,
        &["cars::beyond", "beyond", "cars::reasoning_beyond_text"],
    ),
];

/// The whole outline.
pub(crate) fn outline() -> &'static [OutlineCategory] {
    OUTLINE
}
