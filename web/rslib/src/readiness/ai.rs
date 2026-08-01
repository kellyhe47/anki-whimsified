// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Ticket 008 -- the AI off-switch and the single gate to the network.
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

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::config::BoolKey;
use crate::prelude::*;

/// Permits handed out by [`ai_gate`] since the process started.
///
/// Process-global rather than per-collection on purpose: the claim it supports
/// is "this code path did not attempt to reach a model", and that is a property
/// of the process, not of one open collection.
static PERMITS_ISSUED: AtomicU64 = AtomicU64::new(0);

/// Permission to make one outbound AI request.
///
/// Unforgeable outside this module: the single field is private, so no other
/// module can construct one.
#[derive(Debug)]
#[allow(dead_code)] // the field exists only to make the type unforgeable
pub(crate) struct AiPermit {
    private: (),
}

/// The one gate every outbound AI request passes through.
///
/// Errors whenever [`BoolKey::AiEnabled`] is off -- which is its default -- so
/// the shipped default is an app that cannot talk to a model at all.
#[allow(dead_code)] // consumed by ticket 012, when the AI feature is built
pub(crate) fn ai_gate(col: &Collection) -> Result<AiPermit> {
    if !col.get_config_bool(BoolKey::AiEnabled) {
        invalid_input!("ai is disabled");
    }
    // counted only on success: a refused request never reached the network, and
    // the counter is read as "requests that were allowed out".
    PERMITS_ISSUED.fetch_add(1, Ordering::SeqCst);
    Ok(AiPermit { private: () })
}

/// How many permits [`ai_gate`] has issued in this process.
///
/// Exists so that "scoring never attempts a network call" is testable rather
/// than merely asserted in a comment.
#[allow(dead_code)] // read by the firewall tests
pub(crate) fn permits_issued() -> u64 {
    PERMITS_ISSUED.load(Ordering::SeqCst)
}
