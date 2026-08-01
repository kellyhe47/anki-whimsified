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

/// The whole outline.
pub(crate) fn outline() -> &'static [OutlineCategory] {
    todo!("ticket 005")
}
