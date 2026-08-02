# MCAT study tools and Math Academy: DOK teardown

Research date: 2026-07-30

Status: in progress. This report separates **observed behavior** from **official product claims**. Account-gated Blueprint/UWorld testing and paid Math Academy testing remain pending.

## Method

The lead question is: **What DOK level does the tool actually measure, and what level does it imply it measured?**

- DOK 1: recall and reproduction
- DOK 2: skills and concepts using a known path
- DOK 3: strategic reasoning, justification, and non-routine problems
- DOK 4: extended synthesis and transfer to a genuinely new context

DOK is cognitive complexity, not difficulty. Completion, question volume, accuracy, and time spent are treated as different constructs unless the product explicitly and defensibly connects them.

## Early findings

### Jack Westin MCAT

#### Observed behavior

- Account creation asked for name and email, with phone number and test date optional. Advancing the form immediately produced “Signed in Successfully.” No password was requested, so the supplied password was not used.
- The signed-in home page foregrounded content volume: featured flashcard decks showed 6,301, 157, and 2,887 cards. It also displayed a “Daily Streak: 0.”
- The daily MCAT feed visibly separated “Fundamental Discrete Questions,” “Fundamental Passages,” and “AAMC Style Passages.” This is a useful admission that the inventory spans different cognitive demands.
- I opened the five-question CARS passage “Ojibwe Archeology.” The interface provided an optional timer, highlighting, strikethrough, flagging, navigation, and a “Solution” control.
- Question 1 asked the student to combine two claims supported at different points in a 578-word passage. That item measured DOK 2: evidence retrieval and combination along a known multiple-choice path. It did not require the student to construct or justify an argument, so it did not reach DOK 3 despite the passage format.
- Selecting the correct response and opening “Solution” showed the correct mark and the response distribution (1%, 22%, 6%, 71%). In the state observed, no written rationale appeared. The feedback therefore calibrated correctness and peer agreement, but not the reasoning process.
- The timer remained at `00:00:00` because it was not started. Timing was available but opt-in; the answer was accepted without a timing signal.
- Attempting to leave after answering one of five questions triggered an exact warning: work was saved, but “will not count towards your progress or be displayed in your analytics” until the student clicks “End.” The product therefore counts a formally ended set, not every saved response, in progress/analytics.
- The end-of-set summary classified each question as marked, incomplete, or completed. One answered question was “Completed”; four unanswered questions were “Incomplete.”
- No confidence-before-answer prompt, “I don’t know” answer, system uncertainty state, or automatic rewording was observed in this set.

#### DOK verdict

The tested CARS item actually measured DOK 2. The surrounding “AAMC Style” and “same reasoning found on the MCAT” language implies MCAT-like performance, which can include DOK 3, but a passage wrapper does not make each item DOK 3. The observed progress gate measured set completion state; it was not evidence of readiness.

#### Break log

- “Study QBank Now” returned to the same home-page state instead of visibly opening a distinct builder.
- A promotional modal repeatedly covered the study UI.
- The first signup step acted as account creation without a password or a clearly separate final-create step.
- Direct navigation to analytics was aborted while a passage was active and produced an exit warning.

### Khan Academy MCAT

#### Observed behavior

- The public MCAT landing page showed **15 units · 518 skills**. Units 2–5 were passage-practice units; units 6–15 were content-foundation units. The same top-level skill count therefore combines study inventory with no visible DOK distinction.
- The Biological/Biochemical practice unit said it contained “over 80” passages. Individual passage cards were labeled with five to seven questions, again foregrounding inventory volume.
- I opened “Amino acids: Dietary supplements and the mTOR pathway,” a six-question passage set.
- The first question asked for the approximate pI of branched-chain amino acids. Although embedded in a long experimental passage, the answer came from applying the standard average-of-two-pKa procedure to table values. It measured DOK 2, not DOK 3; most of the passage was irrelevant to the item.
- The visible progress label was “Do 6 problems,” represented by six dots. After one correct response, one dot became complete. This number counted items in the current set, not MCAT readiness.
- Correct feedback said, “Great work! You got it. Onward!” This is reassurance first.
- Calibration was available only after opening “See a step-by-step solution.” The explanation then disclosed four hints, moving from the pI definition to the rule and finally the calculation `(9.7 + 2.3) / 2 = 6.0`.
- There was a Skip control but no confidence rating or explicit “I don’t know” state. No system uncertainty or sample-size warning was visible.
- No timer or pacing benchmark was visible in the exercise. The item accepted an answer without incorporating response time.
- No rewording was observed in the single completed item.

#### DOK verdict

The tested item actually measured DOK 2 while appearing inside an MCAT passage-practice surface. Khan Academy did not present a score or readiness estimate in the anonymous session, which is more restrained than selling the six-item completion indicator as readiness. The risk is subtler: **518 skills**, **80+ passages**, and per-set completion make content coverage visually legible while transfer remains unmeasured.

#### Break log

- The exercise remained in a loading state until the cookie choice was resolved.
- The correct-answer DOM state updated before the visible feedback popover was represented consistently in the accessibility snapshot.
- A large signup banner consumed substantial space above the study interface during anonymous practice.

### Blueprint MCAT

#### Observed public behavior

- A Blueprint account was successfully created on 2026-07-30. The originally supplied password was rejected because Blueprint requires at least 12 characters; the user approved and the account was created with the revised password.
- Creating the base Blueprint account did **not** activate MCAT study access. The account dashboard showed a separate “Blueprint MCAT — Get Started” card.
- Activating the advertised no-credit-card MCAT trial required additional mandatory fields: phone number, target test date, and at least one product-interest checkbox. First name, last name, and email were prefilled from the base account. With the user's approval, the form was completed using an obviously fictional reserved phone number, an offered future test date, and “Practice Tests & Qbank.”
- This is a concrete funnel break: “Try today. No credit card required” is true about payment, but the study surface remains gated behind telephone collection and an intended test date.
- After trial activation, the account dashboard immediately foregrounded paid upgrades: $99/month for exam/Qbank access, $899 for a three-month self-paced course, and $2,999 for the 515+ course. The active free product appeared as “My MCAT — Learning Portal.”
- Entering the learning portal produced another gate: a required checkbox accepting a separate MCAT Enrollment Agreement, plus the linked Terms of Use, Privacy Policy, and Score Increase Guarantee. The user explicitly authorized acceptance.
- The free Qbank offered exactly two practice sets. Before any questions were answered, the builder explicitly admitted insufficient evidence: “To provide personalized recommendations our AI engine needs a steady diet of data… If you’re a new user or haven’t been active for a while, simply continue answering questions so it can learn more about your strengths and weaknesses.” This was the clearest observed MCAT-tool example of telling the student that the system does not yet know.
- After selecting CARS, the builder showed `169/3722 Unseen`. In context, 169 was the eligible unseen CARS pool out of 3,722 total unseen questions, not a readiness or completion percentage.
- The builder let the student choose passage count and a timing multiplier: unlimited, 1x, 1.25x, 1.5x, or 2x. One CARS passage at 1x produced a ten-minute timer.
- A copy defect appeared immediately: the CARS set’s instruction page told the student that Next would advance to “the first question of the Chemical and Physical Foundations of Biological Systems section.”
- The tested CARS passage contained five questions. The items asked for passage-supported implications, author attitude, a likely disagreement by a hypothetical fundamentalist, classification of possible dogma, and identification of a core belief. These were primarily DOK 2: infer, classify, and apply the passage’s stated framework. None required the learner to construct a justification, reconcile competing evidence, or transfer the framework to a genuinely novel source.
- The test explicitly said score was “determined by the number of questions you answer correctly” and encouraged guessing because there was no penalty. It offered no confidence rating or “I don’t know” response.
- The section-review screen classified each item as Complete or Incomplete and allowed Review All, Review Incomplete, and Review Flagged before final submission.
- The completed score report separated `Correct: 3`, `Incomplete: 2`, and `Incorrect: 0`, while also displaying `3/5 Correct`. This is construct slippage: the headline fraction treats omitted items as part of the denominator while the category summary correctly distinguishes omission from error.
- Timing was prominent. The report showed regular-time mode and `Your Time / Max Time: 1:03` for the passage. It did not infer that finishing rapidly meant mastery or readiness.
- The report broke results down by relationship to passage (all five “Requires passage information only”), item difficulty (three low, one medium, one high), and AAMC reasoning skill. Skill 1 showed 100% on one item; Skill 2 showed 50% correct and 50% incomplete across four items. No sample-size warning or confidence interval accompanied those percentages.
- Answer-change reporting distinguished changes from non-changes and explained that answers were automatically selected when the student clicked the text or bubble.
- The top-level Analytics page did **not** incorporate the Qbank set. It displayed: “Your analytics will start loading after you take an exam.” Qbank analytics therefore live in the set-level score report, while the headline Analytics surface is exam-gated.
- The current public site advertised a 14-day trial with no credit card, a half-length diagnostic, a full-length practice exam, analytics, and practice sets.
- Marketing promised that the study plan “automatically adapts to your schedule” and “rebalances” after schedule changes.
- Inventory was prominent: 5,000+ Qbank questions, 1,200 discrete and passage-based sets, 160 learning modules, and multiple practice exams.
- The site described the free diagnostic and full-length exam as tools to “diagnose your strengths and weaknesses.”
- Account-gated study behavior is pending account-creation confirmation.

#### Preliminary DOK hypothesis

The tested Qbank passage measured DOK 2 despite its MCAT-style passage wrapper. Full-length items may reach DOK 3, but that remains untested. Blueprint’s public “representative” and diagnostic claims imply MCAT performance, while its Qbank report measured correctness, omission, speed, item metadata, and coarse AAMC reasoning categories. None of the observed numbers measured DOK 4 readiness.

#### Blueprint break log

- Trial activation required a telephone number and intended test date despite “no credit card required” positioning.
- The study portal required a second legal agreement after the base account and trial had already been created.
- CARS test instructions incorrectly named the Chemical/Physical section.
- Promotional trial-expiration and paid-upgrade banners repeatedly overlaid study and setup screens.
- The general Analytics page remained empty after a completed Qbank set because it only begins loading after an exam.
- The set builder’s explicit insufficient-data message was good calibration, but the resulting skill percentages did not show sample size or uncertainty beside the percentages.

### UWorld MCAT

#### Official claims pending hands-on verification

- UWorld advertises a seven-day, no-credit-card trial with 100 Qbank questions, performance reporting, lessons, videos, flashcards, and an adaptive planner.
- It claims analytics provide insight into exam readiness and that peer benchmarks can “confirm” readiness.
- Public claims combine assignment completion, book completion, Qbank performance, topic trends, peer comparison, and scaled exam scores under the broad idea of progress.

#### Preliminary DOK hypothesis

Passage-based items may reach DOK 2–3; concept checks and flashcards are likely DOK 1–2. “Readiness” would be an overreach if inferred mainly from repeated, untimed, or topic-filtered item-bank performance.

## Math Academy

### Observed access and public behavior

- The public site advertised “Only $49/mo per student” and a 30-day money-back guarantee.
- Selecting the adult-learner signup path required billing address and card details before account creation. There was no free diagnostic-only path visible in the signup flow.
- The public page called the product adaptive and fully automated, said it creates a custom course from strengths and weaknesses, and offered daily XP goals plus overall progress tracking.

### Official construct definitions

- Math Academy explicitly defines one XP as roughly one minute of focused effort. XP is therefore an effort/work-unit and pacing currency, not a readiness probability.
- It separately models topic mastery and spaced-repetition stability.
- Its diagnostic documentation describes a provisional “conditionally completed” state for borderline topics and says later contrary evidence can send the learner back to prerequisites. This is a genuine model-level admission of uncertainty.
- Slow correct answers are weaker evidence than timely correct answers.
- Timed quizzes appear after roughly every 150 XP; over-time or incorrect topics trigger review and a retake.

Sources:

- https://www.mathacademy.com/
- https://www.mathacademy.com/how-it-works
- https://www.mathacademy.com/how-our-ai-works
- https://www.mathacademy.com/adult-students
- https://mathacademy.com/faq

### Preliminary DOK verdict

The core engine appears to measure DOK 1–2 procedural fluency and application, with some DOK 3 potential in multistep, proof, or unfamiliar application work. Its strongest design distinction is not higher DOK but **construct honesty**: XP is effort, while knowledge stability and timed performance are modeled separately. Whether that distinction is equally clear in the student UI remains untested because access requires payment.

## Cross-product comparison (current evidence)

| Product | Tested item DOK | What the visible progress number counted | Explicit uncertainty? | Rewording observed? | Timing behavior |
|---|---:|---|---|---|---|
| Jack Westin | 2 | Formally ended question-set status; daily streak elsewhere | No | No | Optional timer; answer accepted at 00:00:00 |
| Khan Academy MCAT | 2 | Problems completed in a six-item set; 518 skills as inventory | No | No | No timer or pacing signal observed |
| Blueprint | 2 for the tested CARS set | Correct/incomplete/incorrect counts, `3/5`, eligible unseen inventory, and exam-gated analytics | Yes before recommendations: it said it needed more data | No | User-selectable multiplier; ten-minute CARS passage; time/max-time report |
| UWorld | Pending | Claims assignment, content, accuracy, topic, peer, and exam metrics | No public low-confidence state found | Not claimed | Full-length pacing claimed |
| Math Academy | Pending paid access | XP ≈ focused minutes; mastery modeled separately | Yes, “conditionally completed” in official documentation | Not claimed | Slow correct answers discounted; timed quizzes affect review |

## Sources

- Jack Westin home/Qbank and observed passage: https://jackwestin.com/ and https://jackwestin.com/daily/mcat-practice-passages/cars-practice-passages/ojibwe-archeology
- Khan Academy MCAT course and observed exercise: https://www.khanacademy.org/test-prep/mcat and https://www.khanacademy.org/test-prep/mcat/biological-sciences-practice/x04f6bc56:mcat-bio-biochem-foundation-1-passages/e/amino-acids-and-proteins---passage-2
- Blueprint MCAT product and help pages: https://blueprintprep.com/mcat, https://blueprintprep.com/mcat/practice-exams, https://help.blueprintprep.com/en/articles/15363447-mcat-14-day-free-trial, https://help.blueprintprep.com/en/articles/9757728-mcat-lsat-account-analytics
- UWorld MCAT product pages: https://gradschool.uworld.com/mcat/ and https://gradschool.uworld.com/mcat/prep-course/
- Math Academy sources listed in its section above.
