# Open Questions

Standing Q&A file: Claude collects questions here so Ben can answer them in one
go; answered questions are folded into PROJECT-STRUCTURE.md (marked *(decided)*)
and removed.

**Q16 — Luck vs. fumble and explosion (dice engine, M1).** The wiki says luck
points modify the die directly ("eine gewürfelte 9 und ein Glückspunkt zählen
als natürliche 10"), which I implemented symmetrically:
- a rolled 9 + 1 luck counts as a natural 10 and **explodes** (re-roll and add), and
- a rolled 1 + luck counts as 2+, so it is **not a fumble**.

Is the symmetric reading correct — can players buy their way out of a botch
with a pre-committed luck point? (Implemented this way in `dice.rs`; easy to
change if not.)
