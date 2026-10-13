# LearnZed UI design audit (v1 screenshots → v2 polish)

Principles applied: Impeccable (Operate mode: scanability, consistency, native expectations; craft floor: tight groups / generous separation, one primary action, states, copy that names the recovery, no eyebrows/decoration), UI Design Brain (restraint, one accent, no glow), ui-design-saas (≥3 hierarchy levels, one primary CTA per viewport, 8px rhythm, proximity, tokens not magic numbers). Implemented with Zed `ui` components and theme tokens only (Headline/Label sizes, `ButtonStyle::Tinted(Accent)` as the single primary, `status().success/error_*` for feedback, `colors().border_variant`, spacing scale `gap_1/2/3/4`, `p_2/3/4`).

## Track picker (01-track-picker.png)
| # | Finding | Fix |
|---|---|---|
| P1 | Content block floats near the top with no vertical anchor; large empty canvas below reads unfinished. | Column max-width 560px, top offset ~15% of height, consistent 24px section rhythm. |
| P2 | Primary CTA "Start" is a plain-text button at card corner; weaker than the card title. | `Tinted(Accent)` primary button, right-aligned on the title row; one primary per card, CTA verb reflects state (Start / Continue / Review). |
| P3 | Meta row (chapters · exercises · XP · last opened) is four equal-weight muted labels; noisy. | Single muted meta line; XP and "last opened" only when meaningful. |
| P4 | Progress bar at 0% shows an accent sliver; percentage detached at far right. | Progress row below a hairline: "N of M exercises complete" left, percent right, bar beneath. |
| P5 | "Add track from folder…" sits under a divider like a footer link; competes with nothing but looks orphaned. | Secondary (Subtle) button in the header row, right of the heading. |
| P6 | No empty state (only a muted "No tracks installed."). | Empty state: icon, one-line explanation, Add-track action. |
| P7 | Card uses elevated background + border + rounded = generic card. | Keep one bordered surface per track (it is a list item, not nested), editor-matched background, hover border feedback. |

## Learn panel (02/03/04)
| # | Finding | Fix |
|---|---|---|
| L1 | Header (track title + progress) has no separation from the tree; same weight as chapter labels. | Header block with bottom border, title + "N% · XP" on one line, bar below. |
| L2 | Chapter labels "1. Python Basics" are small muted text with tight top spacing; numbering redundant with order. | Chapter headers: Label Small, Muted, more space above than below; no numerals. |
| L3 | Tree items: XP badge right-aligned at same visual weight; locked items lose title contrast entirely. | XP as XSmall muted; locked = muted title + lock icon (still legible), passed = success check, current = accent dot. |
| L4 | Exercise section starts directly after a divider with inconsistent padding (panel p_2 vs markdown edges). | Uniform horizontal padding (12px) for every section; exercise header (title + XP + status) separated by 16px. |
| L5 | Five equal-weight buttons in one wrapping row (Run, Submit, Hint, Reset, Next) – no primary; "Next" wraps to a second line. | Primary row: full-width `Tinted(Accent)` Submit (or Next after passing). Secondary row: Run Code / Hint / Reset as Subtle buttons. |
| L6 | Feedback card is a neutral bordered box; failed checks truncated to one line (03). Pass/fail only distinguishable by a small icon color. | Status-tinted card (success/error background + border tokens), header names outcome + count ("2 of 5 checks failed"), each message wrapped, recovery copy ("Fix the items above and submit again, or take a hint."), output in a code block. |
| L7 | No loading state beyond a muted line; buttons stay enabled. | "Checking your code…" with spinner icon; Submit disabled while checking. |
| L8 | Hint cost label "Hint (-30 XP)" crowds the row. | "Hint" button with tooltip-free secondary label "−30 XP" muted; disabled when exhausted/passed. |

## Out of scope (kept Zed-native)
Fonts, theme, iconography come from Zed. No custom colors, gradients, or shadows added.
