-- Objective exam-item correctness lives in its own table, created on demand.
--
-- It is deliberately *not* part of the synced schema and deliberately not the
-- revlog: the revlog is the scheduler's record of self-rated reviews, and an
-- objective-correctness record is not a review. Creating the table lazily keeps
-- the collection's schema version -- and therefore sync -- untouched.
CREATE TABLE IF NOT EXISTS exam_item_answers (
  -- insertion order, so answers read back oldest first even when two share a
  -- timestamp
  id integer PRIMARY KEY,
  cid integer NOT NULL,
  answered_at integer NOT NULL,
  -- 1 when the typed answer matched the expected text; never derived from the
  -- button the learner pressed
  matched integer NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_exam_item_answers_cid ON exam_item_answers (cid);