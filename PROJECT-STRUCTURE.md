# rustedPunk — Project Structure & Roadmap

Helper tool for a Cyberpunk 2.0.2.0 campaign with extensive homebrew rules
(post-apocalyptic barter world, GURPS advantages, custom dice and damage rules).

Rule sources:

- House rules: https://desaster.fandom.com/wiki/Hausregeln
- Character creation: https://desaster.fandom.com/wiki/Charaktererschaffung
- Core rules (as played, after all house rules): https://desaster.fandom.com/wiki/Regeln
- GURPS-style dis-/advantages list: https://desaster.fandom.com/wiki/VorUndNachteile
- Age points table: https://desaster.fandom.com/wiki/Alterspunkte
- Technology, barter values & weapon stats: https://desaster.fandom.com/wiki/Technlogien
- CP2020 RAW books as PDF under `docs/pdf/` (symlink to a local directory,
  **git-ignored, must never be committed**). Weapon stats: `Cyberpunk 2020 -
  Reference Book 5.pdf`.

Wikitext snapshots of the wiki pages (fetched 2026-07-09) live in
[docs/rules/](docs/rules/).

> Fandom sits behind Cloudflare; plain fetches fail, but the MediaWiki API works:
> `https://desaster.fandom.com/api.php?action=parse&page=<Name>&prop=wikitext&format=json`

**Workflow note:** the repo does not allow direct pushes to `main` — all changes
go through feature branches and PRs.

Where the code and the wiki disagreed, Ben ruled on 2026-07-09 which side is
authoritative; those decisions are folded into section 3 below and marked
*(decided)*.

---

## 1. Where the project stands (2026-07-09, after M0 merges)

- PR #20 (weight & encumbrance, containing all of PR #17's armor work) is merged
  into `main` (squash commit `5639a75`). PR #17 closed as superseded. All stale
  branches deleted; only `main` exists. Issues #7, #12, #14 closed.
- `main` builds clean, all 24 tests pass.
- Modules: `character.rs`, `armor.rs`, `inventory.rs`, `weapons.rs`, `lib.rs`
  (re-exports only), `main.rs` (demo playground).

### Open issues → milestone map

| Issue | Title | Milestone |
|---|---|---|
| #18 | usize → i32 | M0 (last remaining piece) |
| #11 | Rolling dice | M1 — foundation for everything else |
| #19 | Fix details on Character::hit | M2 (TODOs in `character.rs:280-297` + Prellschaden rework) |
| #13 | Injuries and healing | M2 |
| #10 | Dis-/advantages | M3 (phase 1 storage, phase 2 apply) |
| #15 | Skills with special functions | M4 |
| #21 | Weapons | M5 |
| #9 | Generate characters (NSCs) | M6 — needs chargen rules |
| #22 | Aging rules for chars older than 28 | Backlog — rules never written yet |

---

## 2. Target module structure

Keep the existing pattern: `lib.rs` as thin re-export layer, one file per domain,
binary as the user-facing frontend.

```
src/
├── main.rs              # CLI entry point (currently a demo playground → M7)
├── lib.rs               # re-exports only
│
│  # -- rules engine (pure logic, no IO) --
├── dice.rs              # M1: d10 rolls, exploding 10s, fumble confirm, luck spending
├── rolls.rs             # M1: skill checks (attr + skill + d10 vs threshold), auto-success
├── character.rs         # exists: Character, Attribute(s), Skill, List
├── advantages.rs        # M3: Dis-/Advantage storage + modifier application
├── health.rs            # M2: damage track (4-pt blocks), Prellschaden scale,
│                        #     wound penalties, crippling, healing rates
├── chargen.rs           # M6: point-buy, age points, lifepath, NSC generation
│
│  # -- items --
├── inventory.rs         # exists: Inventory, Item, InventoryItem trait, UUIDs, weight
├── armor.rs             # exists: Armor, HitZone, hard/soft, per-zone degradation
├── weapons.rs           # M5: Weapon item type, categories, attachments, noise
│
│  # -- persistence & campaign --
├── io.rs                # TOML load/save (exists in main.rs scope today, extract)
└── campaign.rs          # M8: workshop equipment-token trade system
```

Design rules already established (keep them):

- Weights in grams, currency `eb` (only meaningful at chargen — the world barters).
- `AttributeValue { base, actual }` + `effective_attribute()` computed on demand,
  never cached.
- Trait-object inventory (`Vec<Box<dyn InventoryItem>>`), UUID identity,
  `Option<Uuid>` constructor for load-vs-create.
- Tests colocated per module; serialization round-trip tests for every persisted type.

---

## 3. House rules the engine must encode (translated summary + decisions)

### Dice (`dice.rs`, `rolls.rs`) — for #11

- Check = Attribute (2–10) + Skill (0–10) + 1d10. Difficulty: easy 10+, normal 15+, hard 20+.
- **Auto-success**: if attribute + skill + committed luck already ≥ target, no roll — always, even in combat.
- **Exploding 10**: a natural 10 adds and re-rolls, cascading.
  *(decided: the wiki's "a natural 10 also earns 1 CP for the skill" rule is
  dropped — the table forgot it and doesn't play it.)*
- **Fumble**: natural 1 = failure; roll again: 1 = critical failure (harm own side),
  2–5 = embarrassing failure, 6–0 = normal failure.
- **Luck**: points committed *before* the roll modify the die directly.
  *(decided)*: symmetric reading — 9 + 1 luck counts as a natural 10 and
  explodes; 1 + 1 luck counts as 2 and is no fumble.
- **Luck budget** *(decided, replaces the old per-evening reset)*: luck has
  three levels:
  1. *Starting base* — the chargen value, never changes (`AttributeValue.base`).
  2. *Current base* — permanently reducible for extreme "the world now turns in
     your favor" events; regeneration rate and cap derive from it
     (`AttributeValue.actual`, `Character::sacrifice_luck`).
  3. *Current pool* — fluctuates with every roll (`Character.current_luck`).

  Each new session regenerates ceil(current base / 2) points, capped at the
  current base. Example: current base 9, 8 spent (1 left) → next session +5 →
  starts with 6.
- **Multiple actions**: each extra action per round → −3 on all actions.

### Damage & health (`health.rs`, `character.rs`) — for #13/#19

- Health in blocks of 4 hit points: block 1 scratches (no malus), block 2 wounds
  (−2 REF), block 3 critical (REF/INT/COOL halved, round up), blocks 4–8 mortal
  (all stats except LUCK/EMP divided by 3, round up). Beyond block 8: dead.
- **Prellschaden (bruise damage)** *(decided: this is the authoritative rule; the
  20% blunt-trauma conversion currently in `Character::hit()` was an approximation
  and must be replaced in M2)*:
  - Own scale on the character sheet, 5 points.
  - Strenuous work and damage absorbed by **soft** armor tick it down.
    Damage absorbed by **hard** armor does NOT generate Prellschaden.
  - At 0 → 1 real damage, scale resets to 5.
  - Pure bruise damage = malus of that amount on the next roll (no KO check);
    the converted real damage = KO check but no malus.
  - Some advantages modify the scale (Erhöhte Ausdauer, Schmerzunempfindlich).
- **BTM** *(decided)*: Body Type Modifier is subtracted from incoming damage and
  the subtracted amount is converted into Prellschaden (minimum 1 real damage
  still applies). Melee weapons and martial arts use the related but different
  **DAM** damage modifier when *dealing* damage.
- **Head hits** double damage. (Ordering relative to the death check: see TODO
  `character.rs:297`, part of #19.)
- **Crippling / instant death** *(decided: the code is right, the wiki's
  BODY-roll version is obsolete)*: 8+ damage after BTM in one location →
  critical zones (head/chest/vitals) = instant death; other zones = crippled,
  Mortal 0, minimum 13 damage.
- **Gun overpenetration**: if a firearm deals more than 4 damage, roll 1d10 — that
  is the max extra damage before the shot becomes a through-and-through
  (TODO `character.rs:282`: cap at 4 + 1d10).
- **Crippling injuries**: critical-or-worse wounds have 5% crippling chance without
  proper medical care (0.5% with care).
- Ammo types: AP (armor halved, damage vs soft targets halved); dum-dum/HP
  *(decided, Q17)*: mushrooms — vs hard armor the damage counts half; soft
  armor faces the full damage (generating Prellschaden as usual); damage that
  reaches flesh is DOUBLED and the projectile never exits (no penetration cap).
- **Healing** *(decided, Q18)*: the morning after a wound, BODY roll vs
  10 + taken damage or complications arise — skipped entirely when a healer is
  present (practically always). Healing rate: 1 damage per two days, with a
  healer 1 per day.
- **KO check** *(decided 2026-07-09, Q21 confirmed)*: after taking real damage,
  BODY vs 10, modified by the wound category's Stun malus (character sheet
  wound track: Light 0, Serious −1, Critical −2, Mortal n −(3+n)); sheet BODY,
  no wound thirding. Failure = out of the fight (KO, screaming, …), may repeat
  every round, first success = recovered. Critical failure = GM decides.
- **Crippling roll** *(decided, Q22)*: rolled directly AFTER the fight, once
  per critical-or-worse injury — 5% without proper medical care, 0.5% with
  (Schnelle Heilung: 1% / 0.1%). What "crippled" means has no direct rule; the
  tool reports the result, the GM decides based on the situation.
- Fire damage ignores the ">8 damage" rule.

### Encumbrance (`character.rs`, `inventory.rs`) — implemented, #12 ✓

*(decided: the implemented table IS the house rule — documented here as the
authoritative source since it is not on the wiki)*

- Carry capacity = BODY × 10 kg; deadlift = 4 × carry capacity.
- Penalty by weight/capacity ratio: <0.5 → 0, <0.7 → −1, <1.0 → −2,
  <1.3 → −4, <1.6 → −6, ≥1.6 → −8.
- Reflexes are reduced by inventory encumbrance + armor encumbrance;
  Move by inventory encumbrance only.

### Skills (`character.rs`) — for #15

- All melee skills are "Nahkampf allgemein" (general melee, includes Dodge) up to
  level 3; above that they split into short / medium / long weapon classes.
  *(decided, Q23)*: each specialization CONTINUES the scale independently from
  the shared base. Example: Nahkampf allgemein 3, then Kurz bought to 4 → rolls
  1d10+4 with short weapons, 1d10+3 with medium/long; raising Mittel to 4 is a
  separate purchase.
- *(decided, Q24)*: Dodge stays on the general level (caps at 3). Whether the
  explicit Ausweichen skill should base on general melee is undecided — for now
  it doesn't.
- *(decided, Q25)*: unfamiliar weapons within a known class → difficulty +3
  (the wiki's "+3 auf den Wurf" was an incomplete sentence; general weapon
  knowledge means part of the skill still applies).
- Martial arts *(decided, Q26)*: styles at the table with key-attack bonuses —
  **Prügeln** (none), **Boxen** (+3 Punch, +3 Sweep, +1 Block), **Ringen**
  (+2 Sweep, +4 Grapple, +3 Throw, +4 Hold, +2 Choke, +4 Escape). Key-attack
  bonuses add to the attack roll; the skill level is added 1:1 to damage.
  Base actions per wiki: Strike/Punch 1d3+DAM, Kick 1d6+DAM, Sweep, Block,
  Dodge, Disarm, Grapple, Throw 1d6+DAM (stun check −2), Hold (+1/round),
  Escape, Choke (1d6/round). DAM per RB5 hand-to-hand table (BODY 2 → −2 …
  10 → +2, 11-12 → +4, 13-14 → +6, 15+ → +8).
- Role/class special abilities are abolished — they are ordinary skills
  (Authority→COOL, Interface→INT, Jury Rig→TECH, Meditech→TECH, Leadership) or
  advantages (Charisma, Combat Sense) or replaced by reputation (Credibility,
  Family, Resources) or by Streetwise (Streetdeal).

### Dis-/Advantages (`advantages.rs`) — for #10

- GURPS-style list (see `docs/rules/VorUndNachteile.wiki`, ~25 advantages,
  ~40 disadvantages with exact CP costs and effects).
- Budget: up to 30 CP total, or ONE bigger than 30 plus at most 5 CP more.
- 1 CP = 1 skill point at chargen.
- Effects range from flat roll bonuses (model as modifiers) to purely narrative
  (model as text + CP cost only). Phase 1 = storage, phase 2 = apply to rolls.

### Character creation (`chargen.rs`) — for #9

- 60 points over 9 attributes (INT, REF, COOL, TECH, BODY, MOVE, EMP, LUCK, ATTR);
  INT and REF ≥ 5; value 10 not allowed at creation.
- Lifepath: for each year of age above 15, optionally roll "did something
  interesting happen" — *(decided: this is the CP2020 RAW lifepath system,
  GM-interpreted into the post-apocalyptic setting)* — or write a history by
  hand (no boni, no mali). *(decided, Q29)*: implement the RAW lifepath tables
  with a variant toggle (e.g. `chargen --classic` vs `chargen --desaster`) so
  differently flavored table sets can be added later. Source: RB5's collated
  Expanded Character Creation section (core manual is a text-less scan).
- Skill points = INT + REF + 40 + age points (table in `docs/rules/Alterspunkte.wiki`:
  age 16→1 … 24→9, 25→11, 26→13, 27→15, 28→17, 29→20, 30→23, 31→26, 32→29).
- Skill costs at creation: level N costs N points cumulative-per-level
  (1,2,3 …). Caps: one skill at 8, one at 7, two at 6 — tradeable 1:2 downward.
- In-play skill raises *(decided, Q32)*: **target level × 10 CP** (7→8 = 80 CP;
  the wiki's old "40" was wrong and is fixed).
- Reading/writing is a 5/10 CP advantage; higher education 15 CP and gates
  academic skills (2 levels allowed without it, given fitting background).
- Starting equipment budget = (sum of levels of three profession-defining skills,
  player-chosen, GM veto) × 350 eb. "Reich" advantage doubles the factor per level.
  Last 100 eb convertible to cigarettes/ammo (the actual currency in play).
- Age > 28: aging rules were intended but never written and are not played —
  tracked as its own backlog issue, NOT part of M6.

### Weapons (`weapons.rs`) — for #21

- Stats come from **CP2020 RAW** (`docs/pdf/Cyberpunk 2020 - Reference Book 5.pdf`,
  git-ignored local copy). *(decided, Q27)*: one representative per category
  from the issue list; with multiple candidates pick a MIDDLE one (knife →
  Combat Knife, not Knife and not Kendachi Monoknife). Sling, lance and Molotov
  have no RB5 entries → best-guess stats marked for review.
- Categories from issue #21: knives, clubs, axes, lances, brass knuckles, pistols,
  SMGs, rifles, shotguns, bows, rocket launcher, sling, shuriken, Molotov cocktail.
- *(decided, Q28)*: availability/rarity/cost fields STAY — the tool should also
  serve classical CP2020 settings, not just the post-desaster campaign; expect
  this dual-setting support to grow.
- Attachments: silencer, scope.
- AK damage *(decided/verified in Reference Book 5)*: the classic AK-47/AKM is
  **5d6** (7.62sov, 30 mag, ROF 20, Very Reliable, 400 m) — the table plays 5d6.
  The wiki Technlogien page's "6d6+2" corresponds to the *Czar AK-47* /
  Kalashnikov A-80 variants; the wiki page conflates the two.
- Firing modes: autofire (+1 attack per 10 bullets when close, −1 per 10 when far;
  1 bullet hits per point over target; no precision boni), 3-round burst (+3,
  single target, 1d2 hits on success).
- Aiming/precision modifiers: head shot −4 (double damage), aim +1/round max +3,
  ambush +5 (one round only), prone/rest +1, tripod/turret +2, laser +1,
  scope +1 (+2 extreme range), motionless target +4.
- Shot noise (until someone researches real values): hearing check vs 15 with
  bonus = average damage × 3, −2 per wall, −6 per soundproofed wall, −2 per 50 m,
  +2 per bullet fired that round; silencer halves the initial loudness; direction
  known if check succeeds by 10+.
- Initiative: 1d10 + REF (+boosters); quick-draw +3 initiative for −3 on the action.

### Campaign bookkeeping (`campaign.rs`) — M8

*(decided: XP/Punktevergabe tracking is OUT of scope — that's a player thing.
The workshop trade system IS in scope.)*

- Workshop trading ("Handel in der Werkstattrunde"): equipment-token economy.
  Phase 1 "what does the day offer": roll vs average LUCK of participants against
  10; points over 10 × number of workers = potential tokens; a natural 10 triggers
  a special trade situation (re-roll LUCK, max once/day). Phase 2 "realization":
  average of Menschenkenntnis/Verhandeln/Tech across the round (each worker
  contributes at least one value, each skill present at least once) rolled vs 15,
  no botches/crits; result × workers. Day's tokens = min(phase 1, phase 2).
  The workshop loses 1 token/day per person that needs support (incl. food).
  Acquiring items: roll Luck + existing tokens (cap 15) vs item rarity
  (common 10, rare 20, legendary 40), then reduce tokens by 1.

---

## 4. Roadmap

- **M0 — Consolidate**: ~~merge PRs, delete stale branches, close #7/#12/#14~~
  ✓ done 2026-07-09. Remaining: usize→i32 refactor (#18).
- **M1 — Dice engine (#11)**: pure, seedable RNG (inject `rand::Rng` or a trait for
  testability), skill checks, exploding/fumble/luck/auto-success. Everything later
  (combat, chargen, NSCs) consumes this.
- **M2 — Damage details (#19) + injuries & healing (#13)**: *done 2026-07-09*
  (Prellschaden scale, BTM conversion, penetration cap, head-hit ordering,
  health blocks, wound penalties, hollow-point flesh doubling, healing rates,
  complication check). Deferred: KO-check resolution mechanics and the 5%
  crippling roll (need the advantage system for the modifiers anyway, → M3).
- **M3 — Dis-/advantages (#10)**: *done 2026-07-09* — storage with CP budget
  validation plus generic `Modifier` mechanism (attribute / skill-by-name /
  free tag) wired into checks, initiative, bruise scale and healing rate.
  KO check landed separately (PR #30). Crippling roll (Q22) → M4 or M6.
- **M4 — Skills special functions (#15)**: *done 2026-07-10* — melee split at
  level 3 with independently-continued specializations (Q23), Dodge capped on
  general (Q24), unfamiliar-weapon +3 difficulty (Q25), martial-arts styles
  Prügeln/Boxen/Ringen with key attacks and 1:1 skill-level damage (Q26),
  DAM table, crippling roll (Q22).
- **M5 — Weapons (#21)**: *done 2026-07-10* — Weapon as InventoryItem, one
  middle RAW representative per category (Q27; Sling/Molotov = best guesses),
  availability/cost kept for classic CP2020 support (Q28), DamageType::Fire,
  autofire/burst helpers, noise formula (wiki example has an off-by-one typo:
  AK at 200 m gives 37, not 38).
- **M6 — Character generation (#9)**: *done 2026-07-10* — point-buy/caps/budget
  validation, age points, data-driven lifepath with --classic/--desaster
  toggle (Q29; classic tables are a best-effort transcription, REVIEW), money,
  trait catalog as reviewable TOML (Q30), NSC skeleton generation.
- **M7 — Frontend**: scoped as **EPIC #33** (2026-07-10), implementation
  **ON HOLD (Q31: hosting/auth undecided)**. Sub-issues in priority order:
  #34 character maintenance UI live + PDF (*first*), #35 news tracker with
  propagation (merchants carry news; AI-agent collaboration; replaces Ben's
  Gnumeric sheet), #37 random encounters (post-apoc adaptation of the Night
  City list), #38 loot generation (refresh the ~150-item list), #39 session
  log with tags (replaces the Fandom diary; unified with the character sheet),
  #36 automated full-auto rolls (*lowest*). Ben has detailed requirements per
  step, to be collected when each sub-issue starts. Ground rule: rules engine
  stays a pure library, thin agent-friendly API on top.
- **M8 — Campaign tools**: workshop equipment-token trade simulator (in scope);
  no XP tracking.

**Working agreement (2026-07-09):** Ben approved autonomous milestone-by-milestone
execution; Claude checks in roughly every 20,000 tokens of work.
