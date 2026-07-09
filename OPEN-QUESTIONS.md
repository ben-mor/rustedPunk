# Open Questions

Standing Q&A file: Claude collects questions here so Ben can answer them in one
go; answered questions are folded into PROJECT-STRUCTURE.md (marked *(decided)*)
and removed.

**Q17 — Dum-dum "doppelt gegen weiche Ziele" (issue #19).** The wiki says
hollow-point works at half effect against hard armor and double against soft
targets. What exactly doubles: the damage that penetrates to the character
when the zone has no/only soft armor? Or all penetrating damage? (Currently
armor-level behavior only: incoming halved before armor. TODO(Q17) in
`character.rs::hit`.)

**Q18 — Base healing rate (issue #13).** The wiki only defines the modifiers:
"Schnelle Heilung" heals at double rate, "Langsame Heilung" rolls only every
two days whether healing happens. What is the base rule at your table — a
daily roll (against what?), or fixed points per day like CP2020 RAW? And how
do the crippling-chance rolls (5% / 0.5% with care) time-wise fit in — once
per injury?

**Q19 — Order of maluses (implemented, confirm).** For effective attributes I
apply wound penalties first (halving/thirding the sheet value), then subtract
encumbrance, floor at 0. E.g. REF 8, Critical (→4), encumbrance −2 → 2. OK?

**Q20 — Prellschaden amount per hit (implemented, confirm).** Soft-armor
absorbed damage goes onto the bruise scale point-for-point (18 absorbed = 18
Prellschaden = 3 real damage + scale at 3). Evidence: your old 20%
approximation was exactly absorbed/5. Same point-for-point reading for the
BTM conversion. OK?
