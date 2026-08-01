// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Raw per-topic evidence, gathered in a single pass over cards, revlog and
//! FSRS memory state.
//!
//! This module gathers evidence only: it computes no mastery and no scores.
//! Turning this evidence into a `TopicMastery` or a `Score` belongs to later
//! tickets.

use crate::prelude::*;
use crate::readiness::exam_items::EXAM_ITEM_TAG;
use crate::revlog::RevlogReviewKind;

/// Note tags of the form `mcat::<section>::<topic>` name a topic. Tags without
/// this prefix are not topics and are silently ignored.
pub(crate) const TOPIC_TAG_PREFIX: &str = "mcat::";

/// Everything we observed about one topic, before any interpretation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TopicEvidence {
    /// The tag with [`TOPIC_TAG_PREFIX`] removed, eg `bb::amino_acids`.
    pub topic: String,
    /// Every card of every note tagged with this topic, including suspended
    /// ones.
    pub cards_total: u32,
    /// How many of those cards have at least one *graded* review behind them.
    /// A card whose only revlog entries are manual or rescheduled has never
    /// been answered and so has no history.
    pub cards_with_history: u32,
    /// Real graded reviews. Manual and rescheduled revlog entries are not
    /// gradings and are excluded. Suspending a card stops its future
    /// scheduling but does not erase the reviews it already earned.
    pub graded_reviews: u32,
    /// Mean current retrievability across the cards that have FSRS memory
    /// state. `None` when no card in the topic has memory state -- absent is
    /// not the same claim as zero.
    pub avg_retrievability: Option<f32>,
}

/// The revlog kinds that record an actual grading. `Manual` and `Rescheduled`
/// are bookkeeping entries: they change a card's schedule without the learner
/// ever answering it, so they are not evidence of study.
const GRADED_REVIEW_KINDS: &[RevlogReviewKind] = &[
    RevlogReviewKind::Learning,
    RevlogReviewKind::Review,
    RevlogReviewKind::Relearning,
    RevlogReviewKind::Filtered,
];

/// The single query behind [`Collection::topic_evidence`].
///
/// `notes.tags` is one space-separated string, so the topics have to be split
/// out in SQL if the whole collection is to be summarised in one pass. The
/// recursive `split` CTE peels one tag off the front of each note's tag string
/// at a time; everything downstream is plain aggregation.
///
/// `?1` is the current unix timestamp, the only value the query needs from
/// outside. `extract_fsrs_retrievability` also wants the scheduler's day count
/// and next-day rollover, but `Collection::timing_today()` reads (and
/// sometimes writes) config, which would cost us extra statements and break
/// the one-query guarantee. So the day count is derived from `col.crt` inside
/// the statement instead, and the rollover -- which that function never reads
/// -- is passed as zero. The derived day count is only consulted for cards
/// that have no recorded last review time; for every other card the answer is
/// exact.
///
/// Cards of exam-item notes are excluded from the retrievability average, and
/// from that alone. Retrievability is the memory score's evidence, and memory is
/// DOK 1 flashcard recall: an exam item is not a flashcard, so its recall must
/// not move that average. Its *reviews* still count -- the give-up rule spans
/// every graded review in the collection, exam item or not.
fn evidence_sql() -> String {
    let prefix = TOPIC_TAG_PREFIX;
    let exam_item_tag = EXAM_ITEM_TAG;
    // `substr` is 1-based, so the topic name starts one past the prefix.
    let topic_starts_at = prefix.len() + 1;
    let graded_kinds = GRADED_REVIEW_KINDS
        .iter()
        .map(|kind| (*kind as u8).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
with recursive split(nid, tag, rest) as (
    select id, '', trim(tags) || ' '
      from notes
     where tags like '%{prefix}%'
    union all
    select nid,
           substr(rest, 1, instr(rest, ' ') - 1),
           substr(rest, instr(rest, ' ') + 1)
      from split
     where rest <> ''
),
topics as (
    select distinct nid, substr(tag, {topic_starts_at}) as topic
      from split
     where tag like '{prefix}%'
       and length(tag) > {prefix_len}
),
exam_notes as (
    select distinct nid
      from split
     where tag = '{exam_item_tag}'
),
graded as (
    select cid, count(*) as reviews
      from revlog
     where type in ({graded_kinds})
     group by cid
)
select t.topic,
       count(distinct c.id),
       count(distinct case when coalesce(g.reviews, 0) > 0 then c.id end),
       coalesce(sum(coalesce(g.reviews, 0)), 0),
       avg(case when x.nid is null then extract_fsrs_retrievability(
               c.data,
               case when c.odue != 0 then c.odue else c.due end,
               c.ivl,
               (select max((?1 - crt) / 86400, 0) from col),
               0,
               ?1) end)
  from topics t
  join cards c on c.nid = t.nid
  left join graded g on g.cid = c.id
  left join exam_notes x on x.nid = t.nid
 group by t.topic
"#,
        prefix_len = prefix.len(),
    )
}

impl Collection {
    /// Gather raw evidence for every topic in the collection.
    ///
    /// Must issue exactly one database query regardless of how many topics or
    /// cards exist: the dashboard calls this on collections of ~50,000 cards.
    pub(crate) fn topic_evidence(&mut self) -> Result<Vec<TopicEvidence>> {
        let now = TimestampSecs::now().0;
        let mut stmt = self.storage.db.prepare_cached(&evidence_sql())?;
        let rows = stmt
            .query_and_then([now], |row| -> Result<TopicEvidence> {
                Ok(TopicEvidence {
                    topic: row.get(0)?,
                    cards_total: row.get(1)?,
                    cards_with_history: row.get(2)?,
                    graded_reviews: row.get(3)?,
                    avg_retrievability: row.get::<_, Option<f64>>(4)?.map(|v| v as f32),
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }
}
