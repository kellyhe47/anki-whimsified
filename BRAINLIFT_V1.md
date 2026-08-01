# Brainlift v1: Whimsy as an MCAT memory technology

Status: pre-code research draft  
Exam: MCAT (472-528)  
Spike: Can concept-relevant whimsy make abstract MCAT knowledge more memorable and improve later performance when every whimsical cue has been removed?

## 1. Purpose and scope

### Purpose

Design an Anki-based MCAT product that keeps memory, performance, and readiness separate:

1. **Memory (DOK 1):** Can the learner recall the component knowledge after a delay?
2. **Performance (DOK 2-3):** Can the learner select and apply that knowledge to a new exam-style item?
3. **Readiness (DOK 4 inference):** What score range is supported by novel, mixed, timed evidence, and how uncertain is the estimate?

The product teaches difficult abstract information through accurate, memorable associations, then removes those associations during testing. A 15-minute adaptive snapshot chooses where teaching begins; a continuously updated learner model changes the level of support within each content domain and core skill. Friends-first support targets persistence and emotional resilience. None of these interventions may impersonate readiness: memory, neutral MCAT performance, and score forecasts remain separate.

### Out of scope for v1

- Teaching every MCAT topic or replacing a complete commercial content library.
- Claiming that an intake questionnaire, card-retention estimate, Qbank percentage, streak, or course-completion number predicts an MCAT score.
- A public leaderboard, stranger-first community, follower network, or points economy.
- Calling a 15-minute snapshot a comprehensive diagnostic.
- Treating decorative jokes, mascots, confetti, or enjoyment as evidence that concept-relevant whimsy improved learning.
- AI-generated explanations that lack named sources, held-out evaluation, and a simpler baseline.
- A readiness point estimate without a range, coverage, evidence count, calibration history, and give-up rule.
- Proving that an intervention works for every learner. The first build tests bounded, falsifiable claims.

## 2. DOK 1: sources

### Systems lineage

1. Piotr Wozniak (1990), [*Optimization of Learning*](https://super-memory.org/archive/english/ol.htm) and the [SM-2 source description](https://www.super-memory.org/archive/english/ol/sm2source.htm).
2. Jarrett Ye et al., [FSRS4Anki](https://github.com/open-spaced-repetition/fsrs4anki) and [FSRS algorithm documentation](https://github.com/open-spaced-repetition/fsrs4anki/wiki).
3. Anki, [Deck Options and FSRS](https://docs.ankiweb.net/deck-options.html) and [Studying/answer-button design](https://docs.ankiweb.net/studying.html).

### Learning science

4. Richard Schmidt and Robert Bjork (1992), [“New Conceptualizations of Practice”](https://doi.org/10.1111/j.1467-9280.1992.tb00029.x).
5. Nate Kornell and Robert Bjork (2008), [“Learning Concepts and Categories”](https://doi.org/10.1111/j.1467-9280.2008.02127.x).
6. Doug Rohrer and Kelli Taylor (2007), [“The Shuffling of Mathematics Practice Problems Boosts Learning”](https://doi.org/10.1007/s11251-007-9015-8).
7. Mary Gick and Keith Holyoak (1980), [“Analogical Problem Solving”](https://doi.org/10.1016/0010-0285(80)90013-4).
8. John Sweller (1988), [“Cognitive Load During Problem Solving”](https://doi.org/10.1207/s15516709cog1202_4).
9. K. Anders Ericsson, Ralf Krampe, and Clemens Tesch-Romer (1993), [“The Role of Deliberate Practice in the Acquisition of Expert Performance”](https://doi.org/10.1037/0033-295X.100.3.363).
10. John Dunlosky and Katherine Rawson (2012), [“Overconfidence Produces Underachievement”](https://doi.org/10.1016/j.learninstruc.2011.08.003).
11. Michelle Smith et al. (2009), [“Why Peer Discussion Improves Student Performance on In-Class Concept Questions”](https://pubmed.ncbi.nlm.nih.gov/19119232/).
12. Joel Levin et al. (1996), [“Assessing Students' Application and Transfer of a Mnemonic Strategy”](https://doi.org/10.1006/ceps.1996.0007).
13. Shannon Harp and Richard Mayer (1998), [“How Seductive Details Do Their Damage”](https://doi.org/10.1037/0022-0663.90.3.414).

### Practitioner source requested for inclusion

14. UNC Learning Center, [“Memorization Strategies”](https://learningcenter.unc.edu/tips-and-tools/enhancing-your-memory/). This is useful practitioner guidance, not a primary experiment, so it is not used alone to support a causal claim.

### Product evidence

15. Hands-on teardown: [MCAT study tools and Math Academy: DOK teardown](research/mcat-tools-dok-teardown.md).
16. Primary-source analysis: [Learning science for a readiness product](research/learning-science-product-bet.md).

## 3. DOK 2: what I took and rejected

| Source | What I took | What I rejected |
|---|---|---|
| Wozniak / SM-2 | Scheduling can optimize when a memory item should be reviewed from recall history and item difficulty. | An optimized recall schedule is not a performance or readiness model. |
| FSRS | A learner-specific memory model can estimate retrievability and trade workload against desired retention. | “90% desired retention” does not mean “90% MCAT ready.” FSRS models card memory, not novel passage transfer. |
| Anki design | The prompt-first interaction requires attempted retrieval; its answer buttons distinguish failure, hesitant success, effortful success, and easy success. | Self-grading is not objective correctness, and ease of recalling a card is not strategic reasoning. |
| Schmidt & Bjork | Smooth practice performance can diverge from delayed learning and transfer. | “Difficulty is good.” Difficulty is desirable only when it produces target-relevant processing and remains learnable. |
| Kornell & Bjork | Interleaving can improve classification of new examples while feeling less effective. | “Always interleave everything.” Novices may need initial schema support, and this study does not establish far transfer. |
| Rohrer & Taylor | Mixed practice trains the decision that blocked worksheets give away: which procedure applies. | Mixed known problem families are automatically DOK 4 or far transfer. |
| Gick & Holyoak | A learner can possess relevant knowledge yet fail to notice that it applies when surface cues change. | One analogy experiment supplies a universal MCAT transfer coefficient. We must reproduce the effect with held-out MCAT items. |
| Sweller | For novices, unguided search can consume capacity needed to build schemas; worked examples and faded support can help. | Reducing all cognitive load. Test-relevant complexity must remain; only nonessential load should be removed. |
| Ericsson et al. | Practice becomes deliberate through a weakness-targeted attempt, feedback, correction, and another attempt. | A universal hours rule or a guarantee that more practice creates expertise. |
| Dunlosky & Rawson | Miscalibration changes study behavior: overconfident learners stop too soon. | Confidence equals competence. Confidence must be compared with outcomes across enough observations. |
| Smith et al. | Discussion followed by an individual isomorphic question improved conceptual performance, even when no group member initially had the right answer. | Any social feed improves learning. The useful mechanism is explanation and reconsideration, not ambient comments or popularity. |
| Levin et al. | Mnemonic instruction improved recall and application on the original material. | Learners will spontaneously transfer a mnemonic strategy to new material. Unprompted strategy transfer did not appear in the reported experiments. |
| Harp & Mayer | Interesting but irrelevant details can reduce recall of main ideas and transfer. This supplies the necessary negative control for “fun.” | All vividness or playfulness is harmful. The study targets seductive details that do not encode the lesson's causal structure. |
| UNC Learning Center | Active retrieval, delayed self-testing, expanding intervals, meaningful grouping, and connecting ideas are useful memory strategies. | Mnemonics prove higher-order understanding; switching unrelated subjects is necessarily the useful form of interleaving; spaced exposure warrants certainty. |
| Tool teardown | The observed tools can deliver DOK 2 and sometimes DOK 3 practice, but their top-line progress often counts completion, accuracy, streaks, or covered material. Blueprint at least displayed an “insufficient data” state before recommendations. | “They are all bad.” They explain content, simulate interfaces, provide useful passage practice, and expose timing. The narrower failure is that those signals are easily overread as readiness. |

## 4. DOK 3: synthesis, contradictions, and exposed assumptions

### Where the sources disagree

1. **Challenge versus support.** Desirable-difficulty work warns that easy acquisition can mislead; cognitive-load work warns that premature problem solving can prevent schema formation. The resolution is a sequence, not a compromise: support schema construction, fade support, then measure delayed cue-free transfer.
2. **Blocking versus interleaving.** Blocking can help a novice see a pattern; interleaving makes a learner select among patterns. The app should not choose one globally. It should move from worked/block-supported examples to interleaved discrimination when prerequisite evidence exists.
3. **Confidence versus correctness.** Anki asks the learner to rate recall quality; metacognition research shows judgments can be systematically wrong. Self-report remains useful as a prediction, but only after the system audits it against externally scored delayed performance.
4. **Social support versus social learning.** Peer discussion can create understanding, but the v1 friends feature is designed to improve persistence and emotional resilience. Comments and encouragement are not learning evidence. Any later collaborative-instruction claim must be measured by delayed solo performance.
5. **Relevant whimsy versus seductive detail.** A memorable association may strengthen retrieval when its elements map to the target relationship; interesting but irrelevant material can divert attention and reduce recall or transfer. “Fun” is therefore not the treatment. Explicit concept mapping is, and decorative whimsy is the necessary control.

### What the field and products often assume

- More answered questions imply more readiness.
- A topic label is harmless, even though it gives away the strategy-selection step.
- High immediate accuracy means durable learning.
- A learner’s confidence can be displayed without testing whether it is calibrated.
- “Personalized” means choosing the next content item, even when the system cannot state what evidence supports that choice.
- A diagnostic point estimate is more useful than an honest evidence boundary.
- Social activity and streak maintenance are progress because they correlate with returning.

### What the teardown exposed

- Jack Westin foregrounded card volume and a daily streak; its observed passage work was useful DOK 2 practice, but volume and streak were not readiness evidence.
- Khan Academy separated a large curriculum into units and skills and provided useful explanations, but observed completion/progress did not establish transfer or a score range.
- Blueprint CARS produced passage-style performance and timing evidence. It also said it lacked enough data before producing an AI recommendation, which is a good uncertainty behavior. Its observed item still measured bounded passage performance, not DOK 4 readiness by itself.
- Across the tested experiences, we did not observe a visible loop that captured confidence before feedback, re-probed the same structure with changed surface features after a delay, and reported calibration.
- Math Academy’s public technical model is unusually explicit about mastery and adaptation, but its construct remains mathematics knowledge mastery. A mastery percentage does not automatically answer the different question of MCAT readiness.

## 5. DOK 4: three Spiky POVs

### POV 1: The app should keep learning about the learner

**Consensus says:** A diagnostic is an onboarding event: measure the learner once, produce a score, and generate a study plan.

**I think:** One diagnostic cannot tell you who a learner is. A 15-minute adaptive **readiness snapshot** should choose where teaching begins, while every later interaction updates a multidimensional learner model. Personalization happens within each content domain across shared core measures such as recall, application, transfer, timing, and calibration, plus domain-specific subskills. The system distinguishes measured, inferred, and unknown states.

**Evidence:** Sweller supports matching instructional support to existing schemas. Gick and Holyoak show that stored knowledge can fail to transfer when cues change. Dunlosky and Rawson show that learner-declared mastery is unsafe. The teardown exposed progress displays that compress completion, accuracy, or coverage into a simpler story than the evidence warrants.

**What this forces us to build:** A 15-minute adaptive evidence sampler balancing blueprint coverage, uncertainty, item difficulty, representation, and remaining time; a profile with shared core measures and domain-specific subskills; confidence on every estimate; and continuous updates. The free result includes a wide projected MCAT range, missing evidence, explanations after submission, and conditional study-dose scenarios. The paywall unlocks instruction and continued recalibration.

**What would prove me wrong:** A one-time profile plus fixed plan matches or beats continuous modeling on 14-day neutral transfer, calibration, and forecast accuracy with equal study time. Until longitudinal verified outcomes exist, time-to-goal outputs are planning scenarios, not validated predictions.

### POV 2: Friends first, strangers optional

**Consensus says:** MCAT persistence is an individual discipline problem, so apps use reminders, public communities, streak sharing, and leaderboards.

**I think:** MCAT persistence is partly a social-resilience problem, but public comparison is the wrong default. Support should come from people the learner already trusts. Friends can see effort and moments of struggle, offer encouragement, and help the learner return after a bad session. Strangers are optional, scores are private by default, and social activity never counts as academic progress.

**Evidence:** Smith et al. establishes that structured peer interaction can improve later individual conceptual performance, but it does not prove that a general feed improves learning. This POV deliberately makes the narrower v1 claim about persistence and emotional resilience, not a universal claim that all collaboration improves every learning outcome.

**What this forces us to build:** Invitations for a small trusted circle; privacy controls; shareable effort and struggle events rather than scores; lightweight encouragement and check-ins; and a separate engagement ledger that readiness cannot consume.

**What would prove me wrong:** Compared with private study plus equivalent reminders, friends-first support fails to improve 30-day continuation by **10% relative** or return within 48 hours after a poor session by **10 percentage points**, or it materially increases pressure, embarrassment, or unwanted disclosure.

### POV 3: Concept-relevant whimsy is a memory technology

**Consensus says:** Serious MCAT instruction should present concepts plainly; whimsy belongs in branding, motivation, or decorative engagement.

**I think:** When a playful association accurately maps to an abstract scientific relationship, whimsy can become part of the memory structure. Remembering the whimsical association can retrieve the underlying concept. The platform should teach learners to build these associations, not merely hand them mascots. Whimsical cues appear during teaching and retrieval practice but disappear during formal testing.

**Evidence:** Mnemonic research supports bounded recall and application benefits, but Levin et al. also shows that learners may not spontaneously transfer a mnemonic strategy. Harp and Mayer shows the opposite danger: interesting but irrelevant details can reduce recall and transfer. The evidence therefore does **not** prove this MCAT thesis; it creates the crux. Relevance must distinguish useful mnemonic elaboration from distraction, and neutral delayed transfer must decide.

**What this forces us to build:** Scientifically vetted mnemonic associations; scaffolded co-creation in which the learner keeps, personalizes, or replaces an association; validation that every memorable element maps accurately; FSRS-scheduled retrieval rather than daily repetition of every concept; and neutral tests that remove the whimsy.

**What would prove me wrong:** With equal study time, concept-relevant whimsy fails to beat both plain instruction and decorative whimsy by at least **10 percentage points** on delayed neutral recall and **5 percentage points** on novel neutral MCAT transfer, with the benefit present in at least two of three concept families and no material damage to timing or calibration. If recall improves but neutral transfer does not, it survives as a DOK 1 mnemonic feature but fails as the readiness thesis.

### Which POV carries the product bet?

POV 3 is the thesis. The product bets that **concept-relevant whimsy can improve foundational memory and that the improvement can survive cue removal strongly enough to improve neutral MCAT performance**. POV 1 supplies the measurement and personalization infrastructure. POV 2 supports persistence without being confused with learning evidence.

### Thesis test: three builds, equal study time

Run the same three concept families through three matched instructional conditions:

1. **Plain:** accurate instruction without whimsy.
2. **Decorative whimsy:** equally engaging imagery or language that does not encode the concept.
3. **Concept-relevant whimsy:** memorable elements that map directly to the scientific structure.

Use the same facts, sequence, examples, reading level, retrieval schedule, and study time. A reviewer who does not know the preferred condition rates accuracy, completeness, clarity, and difficulty. Test immediate neutral recall, delayed neutral recall, novel neutral MCAT transfer, response time, and confidence. The test contains no whimsical cues.

## 6. AI consensus check

Model: OpenAI GPT-5.6 Codex  
Date: 2026-07-31  
POV tested: “Concept-relevant whimsy is a memory technology.”

### Pass one: POV cold

**Prompt**

> Concept-relevant whimsy directly improves memory and MCAT performance. Learners should associate difficult abstract chunks of information with playful related elements so remembering the whimsy retrieves the concept.

**Response**

> The memory claim is plausible but underspecified, and the MCAT-performance claim overreaches. Mnemonics often support recall of the encoded material; that does not establish strategic reasoning or transfer to a novel passage. “Whimsy” could mean relevant imagery, decorative humor, positive emotion, or simply an additional explanation. A comparison with plain instruction would not isolate those mechanisms. Define relevance before seeing results, remove the cue during testing, include delayed and novel outcomes, and state what result would make you abandon the readiness claim.

### Pass two: evidence and design supplied

**Prompt**

> Mnemonic research shows bounded recall/application benefits but weak spontaneous strategy transfer; Harp and Mayer found that interesting irrelevant details can reduce main-idea recall and problem-solving transfer. Therefore the intervention is restricted to concept-relevant whimsy whose memorable elements map explicitly to the scientific structure. Learners receive scaffolded, accuracy-checked co-creation. Three equal-time builds compare plain instruction, decorative whimsy, and concept-relevant whimsy across three concept families. All tests are neutral and remove the whimsical cue. Success requires at least a 10-point delayed-recall gain and a 5-point novel-transfer gain over both controls in at least two families, with no material timing or calibration cost. What specifically changes your assessment?

**Response**

> I update from “interesting but underspecified” to “defensible experimental thesis.” The decorative control isolates concept mapping from enjoyment and novelty. The seductive-details evidence makes relevance load-bearing rather than rhetorical. Neutral cue-free tests prevent memory for the mascot from being counted as concept transfer, and the two-stage outcome allows the mnemonic to succeed at recall while failing honestly at readiness. I still do not accept that the effect exists for MCAT material; the proposed experiment is capable of producing that evidence rather than assuming it.

### What moved

The model updated because the design:

1. defines concept relevance before results;
2. separates meaningful mapping from decorative fun;
3. removes whimsical cues during testing;
4. separates memory success from neutral MCAT transfer;
5. contains a numerical failure condition.

The model did **not** update to believing the thesis is true. That remains the point of the three-build test.

## 7. Traceability table

Updated after the Friday build. Paths below are **real files on branch
`feat/readiness-scoring`** in `anki-whimsified/`, not planned targets. Rows that
were not built say so plainly rather than pointing at a stub.

| POV | What it forced us to build | Code that exists now | Number that decides whether it was wrong |
|---|---|---|---|
| The app keeps learning about the learner | measured/inferred/unknown profile cells; per-topic evidence; ranges that widen as evidence falls; confidence on every estimate | ✅ `web/rslib/src/readiness/evidence.rs` (single-query topic evidence) · `learner_model.rs` (measured/inferred/unknown, `MEASURED_REVIEW_THRESHOLD=3`) · `coverage.rs` + `data/aamc_outline.rs` (34 AAMC categories) · `web/qt/aqt/readiness.py` (dashboard). ❌ 15-minute adaptive snapshot **not built**. | Continuous model must beat a frozen baseline on 14-day neutral transfer or calibration by **>=5 points** without reducing completion by more than **5 points**. **Not yet measured** — no longitudinal data exists after one build. |
| Friends first, strangers optional | Trusted-circle invitations; private-by-default sharing; effort/struggle events; no social-to-readiness mapping | ❌ **Not built.** Deliberately descoped: the scope-control rule below says a feature must strengthen a row or produce one of its measurements, and with one night the honesty infrastructure came first. No social code exists, so no social signal can leak into readiness. | **>=10% relative** improvement in 30-day continuation and **>=10-point** improvement in return within 48 hours of a poor session. **Not measured.** |
| Concept-relevant whimsy is a memory technology | Mnemonic model; cue that appears in teaching and disappears in testing; the ablation switch the thesis test needs; separate memory and transfer outcomes | ✅ `web/rslib/src/readiness/mnemonic.rs` + `web/rslib/src/notetype/render.rs` (render-time strip, byte-identical to a card that never had a cue) · `WhimsyEnabled` config key = the ablation control · neutral-test items never render a cue · `scores.rs` keeps Memory (DOK 1) and Performance (DOK 2–3) structurally separate so a mnemonic can succeed at recall and still fail at transfer. ❌ three matched lesson builds **not built**. | Versus both controls: **>=10-point** gain in delayed neutral recall and **>=5-point** gain in novel neutral transfer in at least **2 of 3** concept families. **Not measured** — the switch that makes the experiment runnable exists; the experiment does not. |

### What the build changed about the plan

Three claims in this Brainlift were falsified by contact with the code, and are
corrected rather than quietly dropped:

1. **Performance had no honest data source.** The plan defined it as DOK 2–3 while
   the only available signals were card recall, retrievability and self-rated
   ease — all of which it must not use. A correct implementation would have left it
   permanently abstaining. Resolved by adding objective exam-item correctness taken
   from Anki's typed-answer comparator, never from the answer button.
2. **"Suspended cards excluded from graded reviews" was wrong.** It would have made
   study history non-monotonic — a learner could fall back below the 200-review
   give-up threshold by suspending cards. Corrected: suspension affects future
   scheduling, not accumulated evidence.
3. **CARS has no AAMC content categories.** The plan assumed four sections of
   content categories. AAMC publishes three CARS *skills* and no content
   categories; a coverage percentage measured against an invented outline would
   have been a fabricated measurement. Modelled as skills, labelled as such.

### Scope-control rule

If a proposed feature does not strengthen one of these rows, produce one of its measurements, or satisfy a non-negotiable platform requirement, it is scope creep for Friday.
