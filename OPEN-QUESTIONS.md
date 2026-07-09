# Open Questions

Standing Q&A file: Claude collects questions here so Ben can answer them in one
go; answered questions are folded into PROJECT-STRUCTURE.md (marked *(decided)*)
and removed. Q1–Q31 are answered (Q31 = M7 on hold until Ben decides hosting).

## Non-blocking (answer whenever)

**Q32 — In-play skill-leveling cost formula.** Two data points conflict:
the Charaktererschaffung page says 7→8 costs 40 CP in play, your Q23 example
says Nahkampf Kurz 3→4 costs 40 XP. (new level × 10 fits the second, × 5 the
first.) Not blocking anything — XP tracking is a player thing and out of scope —
but the tool can't validate in-play level-ups until this is settled.

## FYI — working assumptions for the unattended run (veto in PR review)

- **Lifepath source**: the core manual PDF is a scan without text layer, so I
  implement the lifepath from Reference Book 5's collated "Expanded Character
  Creation" tables (which include the RAW core tables). Anything that looks
  expanded-only gets flagged in the PR.
- **Missing RB5 weapons**: Sling, Lance/Spear and Molotov cocktail have no
  Reference Book 5 entries; they get plausible RAW-consistent best-guess stats,
  clearly marked for review (same pattern as the trait catalog, Q30).
- **DAM table** (melee damage modifier), from RB5: BODY 2 → −2, 3-4 → −1,
  5-7 → 0, 8-9 → +1, 10 → +2, 11-12 → +4, 13-14 → +6, 15+ → +8.
