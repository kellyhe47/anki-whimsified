// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 008 -- the AI off-switch and the single gate to the network.
//!
//! STUB. Every body here is `todo!()`; the tests in `ai_firewall_tests.rs`
//! encode what they must do.
//!
//! Spec §3 requires both apps to run with AI switched off, and §7 requires the
//! app to still score with AI off. Ticket 008 does not build the AI feature
//! (that is 012) -- it builds the wall the feature will have to stand behind.
//!
//! Every outbound AI request must first obtain an [`AiPermit`]. `AiPermit` has
//! no public constructor and no public fields, so [`ai_gate`] is the *only*
//! place one can come from: a code path that reaches the network without
//! passing the gate cannot be written. [`permits_issued`] then lets a test prove
//! a negative that is otherwise unobservable -- that scoring did not attempt a
//! network call -- by showing the counter did not move across a scoring run.

use crate::prelude::*;

/// Permission to make one outbound AI request.
///
/// Unforgeable outside this module: the single field is private, so no other
/// module can construct one.
#[derive(Debug)]
#[allow(dead_code)] // stub: the field exists to make the type unforgeable
pub(crate) struct AiPermit {
    private: (),
}

/// The one gate every outbound AI request passes through.
///
/// Errors whenever [`BoolKey::AiEnabled`] is off -- which is its default -- so
/// the shipped default is an app that cannot talk to a model at all.
pub(crate) fn ai_gate(col: &Collection) -> Result<AiPermit> {
    todo!("008: refuse a permit unless BoolKey::AiEnabled is on")
}

/// How many permits [`ai_gate`] has issued in this process.
///
/// Exists so that "scoring never attempts a network call" is testable rather
/// than merely asserted in a comment.
pub(crate) fn permits_issued() -> u64 {
    todo!("008: tally permits issued by ai_gate")
}
