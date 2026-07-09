# Open Questions

Standing Q&A file: Claude collects questions here so Ben can answer them in one
go; answered questions are folded into PROJECT-STRUCTURE.md (marked *(decided)*)
and removed. Q1–Q20 are answered; batch below collected 2026-07-09 so work can
continue through M4–M7 without check-ins.

## Confirmations (implemented, veto if wrong)

**Q21 — KO-check BODY basis.** `ko_check()` uses the sheet BODY (+ advantage
modifiers) minus the wound-track Stun malus — WITHOUT the Mortal thirding,
since the Stun malus already encodes the wound effect and applying both would
double-count. (Merged in PR #30; flagging for the record.)

## Damage / healing leftovers

**Q22 — Crippling roll timing (5% / 0.5%).** "Critical or worse injuries have
a 5% crippling chance without proper medical care (0.5% with care; Schnelle
Heilung lowers to 1% / 0.1%)." When is that rolled — once per qualifying
injury at the moment it's taken, or during healing (e.g. with the
morning-after complication check)? And what does "crippled" mean mechanically
for the tool — permanent note on the zone, attribute reduction, both?

## M4 — Skills with special functions (#15)

**Q23 — Melee specialization levels.** A character has Nahkampf allgemein 3
and then specializes. Does "Kurz 2" mean effective level 5 (3 general + 2
specialization stack), or does the specialization CONTINUE the scale (buying
level 4, 5, …) so the effective level is just the specialization value? What
do rolls with medium/long weapons use for this character — general 3?

**Q24 — Dodge placement.** "Nahkampf allgemein enthält Ausweichen" — after
specializing, does Dodge stay on the general level (3) forever, or does it
grow with a specialization?

**Q25 — Unfamiliar weapon "+3 auf den Wurf".** The wiki sentence reads like a
BONUS for unfamiliar weapons ("bekommt man +3 auf den Wurf, durch das
allgemeine Wissen über Waffen"). I assume it means the roll is made at a
DIFFICULTY +3 (i.e. −3 effectively), or that one rolls with only a base +3
instead of the full skill. Which is it?

**Q26 — Martial arts styles & damage scaling.** Which styles exist at your
table (wiki names Prügeln, Boxen, Ringen) and which key attacks does each
have? And "skill level scales damage" — how exactly (e.g. Strike 1d3 + DAM +
skill level? level added to damage rolls? something else)? The DAM values I
can take from the CP2020 sheet/RAW unless yours differ.

## M5 — Weapons (#21)

**Q27 — Weapon catalog scope.** Import the full CP2020 RAW weapon list from
Reference Book 5 as a data file, or a curated TL6-appropriate list for your
post-desaster world (AK-47, Grach, hunting rifles, shotguns, bows, melee …)?
If curated: rough list of what exists in-world would help.

**Q28 — Weapon stats that matter.** Planned fields: damage dice, damage type,
ROF/burst modes, magazine size, range, reliability, concealability, weight,
attachments (silencer/scope). Anything else your table actually uses —
e.g. availability/rarity for the workshop trading system?

## M6 — Chargen (#9)

**Q29 — Lifepath depth.** Should the tool implement the CP2020 RAW lifepath
tables (roll per year over 15) with your post-apocalyptic reinterpretation as
editable text results — or keep lifepath freeform (text only) and only
implement the mechanical parts (attributes, skill points, age points, budget,
advantages)?

**Q30 — Trait catalog.** OK if I encode all ~65 wiki Vor-/Nachteile as a TOML
data file with my best-guess modifiers (mechanical ones tagged, narrative ones
plain), for you to review in the PR diff?

## M7 — Web service (planning ahead)

**Q31 — Hosting target.** What runs on your server (Docker? bare systemd?
reverse proxy?), and how should players authenticate — simple shared password,
per-user accounts, something existing (e.g. forum/Nextcloud SSO)? This shapes
whether I build a simple axum binary with a token or something bigger.
