/*
 * Copyright (c) 2026 AnkiDroid contributors
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
 * details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program.  If not, see <http://www.gnu.org/licenses/>.
 */

package com.ichi2.anki.readiness

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.util.TypedValue
import android.view.View
import android.widget.LinearLayout
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.text.HtmlCompat
import androidx.fragment.app.Fragment
import anki.readiness.Score
import anki.readiness.ThreeScoresResponse
import com.ichi2.anki.CollectionManager.withCol
import com.ichi2.anki.R
import com.ichi2.anki.SingleFragmentActivity
import com.ichi2.anki.databinding.FragmentReadinessBinding
import com.ichi2.anki.launchCatchingTask
import dev.androidbroadcast.vbpd.viewBinding
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * The exam readiness screen: the same three scores the desktop dashboard shows, read from the
 * same backend, under the same give-up rule.
 *
 * The one rule this screen exists to enforce: **an abstaining score never renders an estimate**.
 * Not greyed out, not with an asterisk, not as a plausible-looking figure with a caveat beside it.
 * When the backend abstains, this screen shows the refusal and names what is missing, because
 * dressing a guess as a measurement is the failure the whole readiness module is built to prevent.
 *
 * There is deliberately no code path below that formats a score's estimate, range or confidence
 * while its `abstaining` flag is set.
 */
class ReadinessFragment : Fragment(R.layout.fragment_readiness) {
    private val binding by viewBinding(FragmentReadinessBinding::bind)

    override fun onViewCreated(
        view: View,
        savedInstanceState: Bundle?,
    ) {
        super.onViewCreated(view, savedInstanceState)

        binding.toolbar.apply {
            title = TITLE
            setNavigationOnClickListener {
                requireActivity().onBackPressedDispatcher.onBackPressed()
            }
        }
        (requireActivity() as AppCompatActivity).setSupportActionBar(binding.toolbar)

        addHtml("Reading the collection…")

        launchCatchingTask {
            val scores = withCol { backend.threeScores() }
            render(scores)
        }
    }

    private fun render(scores: ThreeScoresResponse) {
        binding.readinessContainer.removeAllViews()

        addHtml(
            "<big><big><b>$TITLE</b></big></big><br><br>" +
                "Three separate scores, each with its own evidence and its own range. " +
                "They are never averaged, combined or reduced to a single headline number: " +
                "a blend would hide which of the three is actually evidenced.",
        )
        addHtml(
            "<b>The give-up rule.</b><br>" +
                "Readiness (DOK 4) reports <b>no number at all</b> unless the bar is cleared. " +
                "Performance and Memory abstain on their own terms too. Inventing a readiness " +
                "number, or dressing a guess as a measurement, is not something this screen " +
                "will do.",
        )

        addCard(
            title = "Memory  ·  DOK 1",
            subtitle =
                "Recall of the cards themselves. Reads graded card reviews and FSRS " +
                    "retrievability, and nothing else.",
            unit = "percent recall $PERCENT_UNIT",
            evidenceLabel = "graded review(s)",
            score = if (scores.hasMemory()) scores.memory else null,
        )
        addCard(
            title = "Performance  ·  DOK 2–3",
            subtitle =
                "Objective exam-item accuracy. Scored from the typed answer alone; the " +
                    "answer button pressed is ignored.",
            unit = "percent of exam items correct $PERCENT_UNIT",
            evidenceLabel = "answered exam item(s)",
            score = if (scores.hasPerformance()) scores.performance else null,
        )
        addCard(
            title = "Readiness  ·  DOK 4",
            subtitle =
                "A claim about exam-day performance: measured exam-item accuracy weighted " +
                    "against how much of the outline the deck reaches.",
            unit = MCAT_UNIT,
            evidenceLabel = "graded review(s) + answered exam item(s)",
            score = if (scores.hasReadiness()) scores.readiness else null,
        )
    }

    /** One of the three scores, rendered on its own terms. */
    private fun addCard(
        title: String,
        subtitle: String,
        unit: String,
        evidenceLabel: String,
        score: Score?,
    ) {
        val body =
            when {
                score == null -> missingScoreBody()
                score.abstaining -> abstentionBody(score)
                else -> estimateBody(score, unit, evidenceLabel)
            }
        addHtml(
            "<big><b>${escape(title)}</b></big><br>" +
                "<small>${escape(subtitle)}</small><br><br>$body",
            isCard = true,
        )
    }

    // abstention
    // ####################################################################

    /** The backend returned no score object at all. */
    private fun missingScoreBody(): String =
        "<font color='$DANGER_COLOUR'><big><b>NO SCORE</b></big></font><br><br>" +
            "The backend returned no score for this measure. Nothing is estimated in its place."

    private fun abstentionBody(score: Score): String {
        val missing =
            score.missingEvidenceList.ifEmpty {
                listOf("(the backend named no specific shortfall)")
            }
        val html =
            StringBuilder()
                .append(
                    "<font color='$DANGER_COLOUR'><big><b>NO SCORE — INSUFFICIENT EVIDENCE" +
                        "</b></big></font><br><br>",
                ).append("<b>Missing evidence:</b><br>")
                .append(bullets(missing))
        if (score.reasonsList.isNotEmpty()) {
            html.append("<br><b>Why:</b><br>").append(bullets(score.reasonsList))
        }
        html
            .append(
                "<br>Coverage: ${percent(score.coveragePct)} of the AAMC outline<br>" +
                    "Last updated: ${escape(timestamp(score.lastUpdated))}<br><br>",
            ).append(
                "<small>No estimate, no range and no confidence figure are shown, " +
                    "because there is nothing measured to report.</small>",
            )
        return html.toString()
    }

    // a reported estimate
    // ####################################################################

    private fun estimateBody(
        score: Score,
        unit: String,
        evidenceLabel: String,
    ): String {
        val html =
            StringBuilder()
                .append("<big><big><big><b>${decimal(score.estimate)}</b></big></big></big><br>")
                .append("<small>${escape(unit)}</small><br><br>")
                .append("<b>Range: ${decimal(score.low)} – ${decimal(score.high)}</b><br><br>")
                .append(
                    "Confidence: ${twoDecimals(score.confidence)} (0 = none, 1 = certain)<br>" +
                        "Coverage: ${percent(score.coveragePct)} of the AAMC outline<br>" +
                        "Evidence: ${score.evidenceCount} ${escape(evidenceLabel)}<br>" +
                        "Last updated: ${escape(timestamp(score.lastUpdated))}",
                )
        if (score.reasonsList.isNotEmpty()) {
            html.append("<br><br><b>Reasons:</b><br>").append(bullets(score.reasonsList))
        }
        return html.toString()
    }

    // rendering helpers
    // ####################################################################

    private fun addHtml(
        html: String,
        isCard: Boolean = false,
    ) {
        val textView =
            TextView(requireContext()).apply {
                text = HtmlCompat.fromHtml(html, HtmlCompat.FROM_HTML_MODE_COMPACT)
                textSize = 14f
                layoutParams =
                    LinearLayout
                        .LayoutParams(
                            LinearLayout.LayoutParams.MATCH_PARENT,
                            LinearLayout.LayoutParams.WRAP_CONTENT,
                        ).apply { bottomMargin = dp(16) }
                if (isCard) {
                    setPadding(dp(12), dp(12), dp(12), dp(12))
                    setBackgroundResource(R.drawable.readiness_card_background)
                }
            }
        binding.readinessContainer.addView(textView)
    }

    private fun dp(value: Int): Int =
        TypedValue
            .applyDimension(
                TypedValue.COMPLEX_UNIT_DIP,
                value.toFloat(),
                resources.displayMetrics,
            ).toInt()

    companion object {
        private const val TITLE = "Exam Readiness"

        /** The colour an abstention is stated in; legible on both the light and dark themes. */
        private const val DANGER_COLOUR = "#E53935"
        private const val PERCENT_UNIT = "%"
        private const val MCAT_SCALE_MIN = 472
        private const val MCAT_SCALE_MAX = 528
        private const val MCAT_UNIT = "on the $MCAT_SCALE_MIN–$MCAT_SCALE_MAX MCAT scale"

        fun getIntent(context: Context): Intent = SingleFragmentActivity.getIntent(context, ReadinessFragment::class)

        private fun escape(text: String): String =
            text
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("\"", "&quot;")

        /**
         * Every entry, one per line. `<ul>`/`<li>` are not rendered by Android's HTML parser, so
         * the bullet is written out rather than relying on list markup.
         */
        private fun bullets(items: List<String>): String = items.joinToString("") { "•&nbsp;${escape(it)}<br>" }

        private fun decimal(value: Float): String = String.format(Locale.US, "%.1f", value)

        private fun twoDecimals(value: Float): String = String.format(Locale.US, "%.2f", value)

        private fun percent(value: Float): String = String.format(Locale.US, "%.1f%%", value)

        private fun timestamp(secs: Long): String {
            if (secs == 0L) return "never"
            return SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).format(Date(secs * 1000))
        }
    }
}
