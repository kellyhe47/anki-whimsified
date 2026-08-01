// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Storage for objective exam-item correctness.
//!
//! The answers live in their own `exam_item_answers` table, created on demand
//! rather than through a schema upgrade: bumping the collection's schema version
//! would make the file unreadable to other Anki builds, and this record is not
//! part of the synced collection. It is emphatically *not* the revlog -- the
//! revlog holds the scheduler's self-rated reviews, and an objective-correctness
//! record is a different kind of claim.

use rusqlite::params;

use super::SqliteStorage;
use crate::prelude::*;
use crate::readiness::evidence::TOPIC_TAG_PREFIX;
use crate::readiness::exam_items::ExamItemAnswer;
use crate::readiness::exam_items::TopicExamItems;
use crate::readiness::exam_items::EXAM_ITEM_TAG;

/// Per-topic exam-item counts, in one pass.
///
/// `notes.tags` is a single space-separated string, so the tags are peeled off
/// one at a time by the recursive `split` CTE -- the same shape
/// [`crate::readiness::evidence`] uses. A note is an exam item when one of those
/// tags is exactly [`EXAM_ITEM_TAG`]; its topics are the `mcat::`-prefixed tags
/// on the same note.
///
/// Topics whose exam items nobody has answered still appear, with zero counts:
/// "not attempted" is a different claim from "not present".
fn topic_exam_items_sql() -> String {
    let prefix = TOPIC_TAG_PREFIX;
    // `substr` is 1-based, so the topic name starts one past the prefix.
    let topic_starts_at = prefix.len() + 1;
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
     where tag = '{EXAM_ITEM_TAG}'
),
answers as (
    select cid,
           count(*) as answered,
           sum(matched) as correct
      from exam_item_answers
     group by cid
)
select t.topic,
       coalesce(sum(coalesce(a.answered, 0)), 0),
       coalesce(sum(coalesce(a.correct, 0)), 0)
  from topics t
  join exam_notes e on e.nid = t.nid
  join cards c on c.nid = t.nid
  left join answers a on a.cid = c.id
 group by t.topic
"#,
        prefix_len = prefix.len(),
    )
}

impl SqliteStorage {
    /// Create the exam-item table if this collection has never had one.
    ///
    /// Idempotent, and cheap enough to call before every access, which keeps the
    /// table available on collections created before this feature existed
    /// without touching the schema version.
    fn ensure_exam_item_table(&self) -> Result<()> {
        self.db.execute_batch(include_str!("create.sql"))?;
        Ok(())
    }

    pub(crate) fn add_exam_item_answer(&self, answer: &ExamItemAnswer) -> Result<()> {
        self.ensure_exam_item_table()?;
        self.db
            .prepare_cached(include_str!("add.sql"))?
            .execute(params![answer.card_id, answer.answered_at, answer.matched])?;
        Ok(())
    }

    /// Every recorded answer, oldest first.
    pub(crate) fn all_exam_item_answers(&self) -> Result<Vec<ExamItemAnswer>> {
        self.ensure_exam_item_table()?;
        self.db
            .prepare_cached(include_str!("get.sql"))?
            .query_and_then([], |row| -> Result<ExamItemAnswer> {
                Ok(ExamItemAnswer {
                    card_id: row.get(0)?,
                    answered_at: row.get(1)?,
                    matched: row.get(2)?,
                })
            })?
            .collect()
    }

    pub(crate) fn topic_exam_item_counts(&self) -> Result<Vec<TopicExamItems>> {
        self.ensure_exam_item_table()?;
        self.db
            .prepare_cached(&topic_exam_items_sql())?
            .query_and_then([], |row| -> Result<TopicExamItems> {
                Ok(TopicExamItems {
                    topic: row.get(0)?,
                    exam_items_answered: row.get(1)?,
                    exam_items_correct: row.get(2)?,
                })
            })?
            .collect()
    }
}
