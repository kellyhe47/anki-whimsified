// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub(crate) mod ai;
#[cfg(test)]
mod ai_firewall_tests;
pub(crate) mod coverage;
#[cfg(test)]
mod coverage_tests;
pub(crate) mod data;
pub(crate) mod deckgen;
#[cfg(test)]
mod deckgen_tests;
pub(crate) mod evidence;
#[cfg(test)]
mod evidence_tests;
pub(crate) mod exam_items;
#[cfg(test)]
mod exam_items_tests;
pub(crate) mod learner_model;
#[cfg(test)]
mod learner_model_tests;
pub(crate) mod mnemonic;
#[cfg(test)]
mod mnemonic_tests;
/// Test-only: the synthetic-deck generator and the mastery-query benchmark are
/// never built into a shipped binary. See the module docs for why.
#[cfg(test)]
pub(crate) mod perf;
#[cfg(test)]
mod perf_tests;
pub(crate) mod provenance;
pub(crate) mod scores;
#[cfg(test)]
mod scores_tests;
mod service;
#[cfg(test)]
mod tests;
