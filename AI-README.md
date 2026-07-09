# AI-README: RustedPunk Project Structure

## Overview
Cyberpunk 2020 RPG character management system. Currency: `eb` = Eurobucks.

## Instructions for the AI
I'm learning Rust and you are my teacher.
Don't generate code unless I explicitly ask for it.
Mention situations where I mistype or misspell words / grammar.

**Note:** This file is maintained by the AI to track project structure and design decisions.

---

## Key Abbreviations

### Character Attributes (`character.rs`)
- `att` - Attractiveness
- `mov` - Movement  
- `coo` - Cool
- `emp` - Empathy
- `luck` - Luck
- `int` - Intelligence
- `body` - Body
- `refl` - Reflex (field name avoids `ref` keyword)
- `tec` - Technical Ability

---

## Non-Obvious Design Patterns

### Armor Damage Tracking (`armor.rs`)
- `protection_max: i32` - Maximum stopping power (SP)
- `protection_current: BTreeMap<HitZone, i32>` - Tracks per-zone degradation (WIP)

### Number Types (#18)
- All game values are `i32` (attributes, damage, protection, encumbrance,
  skill levels, amounts, weights, prices) — allows direct subtraction.
- Non-negativity of `Item` quantities (`amount`, `weight_grams`, `price_eb`) is
  enforced by explicit validation: `Item::new` panics with a meaningful message,
  deserialization fails via `#[serde(try_from = "UncheckedItem")]`.
- `effective_attribute()` clamps at 0; `take_damage` keeps the min-1-after-BTM rule.

### List Struct (`character.rs`)
- `List(pub Vec<Skill>)` - Newtype wrapper for skill collections
- Defined but not yet integrated into `Character` (intended for character skill lists)

### InventoryItem Trait (`inventory.rs`)
- Polymorphic inventory: `Vec<Box<dyn InventoryItem>>`
- Composition pattern: `Armor` contains `Item`, accessed via `get_item()`
- Uses `InventoryItem.as_any()` and `as_any_mut()` for downcasting trait objects to concrete types
- `Inventory.get_item()` and `get_item_mut()` return `Option<&dyn InventoryItem>` and `Option<&mut dyn InventoryItem>`

---

## Units & Conventions
- Weight: grams (`weight_grams`)
- Display implementations delegate to Debug for enums (e.g., `HitZone`)

---

## Damage Type Mechanics (`armor.rs`)
- **ArmorPiercing**: Halves effective protection, halves remaining penetrating damage
- **Blunt**: Uses full protection, no damage modifications
- **HollowPoint**: Halves incoming damage before armor calculation
- **Slashing**: Full protection vs hard armor, halved vs soft armor (does NOT halve penetrating damage)

Each damage type has a private helper method. Full mechanics documented on public `hit()` method.

---

## Item Identity System

### UUID Usage (`inventory.rs`)
- All `Item` instances have unique `Uuid` (uuid crate with v4 feature)
- `Item::new(uuid: Option<Uuid>, ...)` - Pass `None` to auto-generate, `Some(uuid)` for loading from storage

### Armor Layering (Implemented)
- Characters wear armor in ordered layers: `Character.worn_armor: Vec<Uuid>`
- Index 0 = innermost layer, higher indices = outer layers
- `wear_armor(uuid, underneath: Option<Uuid>)` - wear armor at outermost or under specific armor
- `Inventory.get_item(uuid)` and `get_all_armor()` - lookup by UUID or filter by type
- Uses `InventoryItem.as_any_mut()` for downcasting trait objects to concrete `Armor` type
- Damage processes layers from outside-in (reverse iteration) - IMPLEMENTED in `Character.hit()`

---

## Character Damage Mechanics (`character.rs`, `health.rs`)

### `Character.hit(damage, zone, damage_type, is_gunshot, roller)` → `HitOutcome`
- Processes damage through all worn armor layers (outside-in, reverse iteration)
- Damage absorbed by SOFT armor becomes Prellschaden point-for-point; hard
  armor absorbs without consequence (Q20)
- **Penetration cap** (gunshots only): more than 4 remaining damage rolls 1d10;
  the shot does at most `4 + 1d10` — the rest exits through the back
  (`through_and_through` in the outcome)
- Then delegates to `resolve_damage` (same core as `take_damage`)
- **HollowPoint** (Q17): mushrooms — halved vs hard armor (in `Armor.hit`),
  full damage vs soft armor; damage reaching flesh DOUBLES and the projectile
  never exits (no penetration cap)

### KO check ("Stun Save")
- Any hit causing real damage sets `ko_check_required` (pure Prellschaden
  gives the next-roll malus instead)
- `Character::ko_check(roller)`: BODY (sheet value + advantage modifiers,
  NOT wound-thirded — the category malus covers that) − Stun malus vs 10.
  Stun malus per wound track: Light 0, Serious −1, Critical −2, Mortal n −(3+n)
- Failure = out of the fight, may repeat every round, first success recovers;
  critical failure = GM decides (usually out longer)

### Healing (Q18)
- `Character::rest_day(healer_present)`: heals 1/day with healer, 1 per two
  days without (`healing_progress` carries the half-day)
- `Character::complication_check(healer_present, roller)`: morning-after BODY
  roll vs 10 + current damage; `None` with a healer (no check needed)

### `Character.take_damage(damage, zone)` → `HitOutcome`
- Bypasses armor. BTM is SUBTRACTED and CONVERTED to Prellschaden (min 1 real
  damage remains): `converted = btm.min(damage - 1)`
- **Prellschaden scale** (`current_bruise`, 0–4): every 5 points → 1 real
  damage + KO check required (`ko_check_required`); a hit causing ONLY
  Prellschaden instead sets `pending_roll_malus` (consumed by the next
  check_skill/check_attribute)
- **Crippling check** (8+ zone damage after BTM) uses the UNDOUBLED value;
  **head doubling** applies afterwards:
  - Critical zones (Head/Chest/Vitals): instant death, `current_damage = 100`
  - Other zones: Mortal 0 state, minimum `current_damage = 13`

### Wound states (`health.rs::WoundState`)
- Blocks of 4 damage: Light (1-4, no malus), Serious (5-8, −2 REF), Critical
  (9-12, REF/INT/COOL halved round up), Mortal 0-4 (13-32, all stats except
  LUCK/EMP divided by 3 round up), Dead (>32)
- `Character::wound_state()`; penalties are applied inside
  `effective_attribute()` BEFORE encumbrance is subtracted (Q19)

---

## Encumbrance & Weight System (`character.rs`, `inventory.rs`)

### Weight Tracking
- `Inventory.calculate_total_weight()` - Sums `weight_grams` of all items
- `Character.carry_capacity()` - Returns `Body * 10,000` grams (Body 5 = 50kg)
- `Character.deadlift()` - Returns `carry_capacity() * 4`

### Encumbrance Penalties
Calculated as ratio of `inventory_weight / carry_capacity`:
- **0.0-0.49**: No penalty (0)
- **0.5-0.69**: -1 penalty
- **0.7-0.99**: -2 penalty
- **1.0-1.29**: -4 penalty (overloaded!)
- **1.3-1.59**: -6 penalty
- **1.6+**: -8 penalty

Implementation uses integer math: `(inventory_weight * 10) / capacity` to avoid floating point.

### Attribute System: Base vs Actual vs Effective
- **`base`**: Natural attribute at character creation (never changes)
- **`actual`**: "Current" value shown on character sheet, includes semi-permanent mods (cyberware, training, long-term injuries). Persists between sessions.
- **`effective_attribute(attr)`**: Calculated on-demand for dice rolls. Includes temporary modifiers (drugs, encumbrance, combat effects).

Design pattern: Calculate effective values on-demand rather than caching. Avoids synchronization issues with multiple modifier sources.

Encumbrance affects:
- **Reflexes**: Reduced by inventory encumbrance + armor encumbrance
- **Move**: Reduced by inventory encumbrance only

---

## Dice Engine (`dice.rs`) — #11

- All rolls go through the `DieRoller` trait: `RandomRoller` (rand crate) in
  play, `SequenceRoller` (scripted values) in tests and for future replay.
- `skill_check(attribute, skill, luck, difficulty, roller)` implements the
  house rules: auto-success when attribute+skill+luck ≥ target (no roll),
  exploding 10s (cascade), fumble on 1 with confirmation die (1 critical,
  2-5 embarrassing, 6-10 normal failure), luck modifies the first die
  directly (9+1 luck = natural 10 and explodes; 1+luck = no fumble).
- `open_roll(...)` = same mechanics without a target.
- `Character::check_skill(name, ...)` -> `Result` (error on unknown skill),
  uses `effective_attribute` so encumbrance maluses apply automatically.
  `Character::check_attribute(...)` for untrained rolls.
- **Luck, three levels**: starting base (`AttributeValue.base`, chargen value,
  never changes) / current base (`AttributeValue.actual`, permanently reducible
  via `Character::sacrifice_luck` for world-turning events; regen rate + cap) /
  current pool (`Character.current_luck`, persistent across sessions).
  Committed luck is deducted by check_skill/check_attribute (also on
  auto-success); overspending errors without rolling.
  `Character::start_session()` regenerates ceil(current base / 2), capped at
  the current base.
- `Difficulty::{Easy=10, Normal=15, Hard=20, Custom(n)}`.

---

## Melee & Martial Arts (`melee.rs`) — #15

- General melee skill "Nahkampf" (incl. Dodge) caps at 3 (`MELEE_GENERAL_CAP`);
  specializations "Nahkampf Kurz/Mittel/Lang" CONTINUE the scale independently.
  `Character::melee_level(class)` = specialization if present, else general
  (capped). `check_melee(class, familiar, ...)`: unfamiliar weapon = target +3.
  `check_dodge()` always uses general (Q24).
- Martial arts: styles Prügeln (no key attacks), Boxen (+3 Punch/+3 Sweep/
  +1 Block), Ringen (+2 Sweep/+4 Grapple/+3 Throw/+4 Hold/+2 Choke/+4 Escape).
  `check_martial_arts(style, action, ...)` adds the key-attack bonus to the
  roll; `martial_arts_damage()` = base dice (Punch 1d3, Kick/Throw/Choke 1d6)
  + DAM (not for Choke) + skill level 1:1.
- `Character::dam()` = hand-to-hand DAM from sheet BODY (RB5 table:
  ..2 → −2, 3-4 → −1, 5-7 → 0, 8-9 → +1, 10 → +2, 11-12 → +4, 13-14 → +6,
  15+ → +8). Distinct from `btm()` (damage reduction when hit)!
- `dice::DiceExpr` ("5d6+2"): parse/display/roll/average (integer halving —
  matches wiki noise examples); `DieRoller::die(sides)` for non-d10 dice;
  `roll_per_mille()` for the crippling check.
- `Character::crippling_check(medical_care, roller)`: rolled once per
  critical+ injury directly after the fight; 5%/0.5%, Schnelle Heilung 1%/0.1%.

## Weapons (`weapons.rs`) — #21

- `Weapon` (InventoryItem, serde: `item` field LAST for TOML) with category,
  WA, concealability, availability, damage `DiceExpr`, damage type, magazine,
  ROF, reliability, range, silencer/scope flags. Availability/cost kept for
  classic-CP2020 support (Q28).
- `Weapon::catalog()`: one middle RAW representative per issue-#21 category
  (Q27); comments carry the RB5 source line. Sling + Molotov = BEST GUESS
  (no RB5 entry), marked in the comment.
- `DamageType::Fire` (new): armor protects like Blunt, never cripples
  (>8-rule ignored), used by the Molotov.
- `Character::check_ranged_attack(weapon, skill, ...)` adds WA (+1 scope);
  `Character::weapon_damage()` adds DAM for melee-class weapons.
- Autofire: `autofire_attack_modifier(bullets, close)` = ±1 per 10 bullets,
  `autofire_hits(total, target, bullets)` = margin capped at bullets.
  Burst: `BURST_ATTACK_BONUS` (+3), `burst_hits()` = 1d2.
- `shot_noise_bonus(weapon, bullets, distance, walls, soundproofed)`:
  avg×3 (silencer halves), −2/wall, −6/soundproofed, −2/50m, +2/bullet.
  NOTE: the wiki's 200m AK example says 38 but the formula gives 37 (wiki
  arithmetic typo).
- When adding InventoryItem types: extend SerializeableInventoryItem + both
  Serialize/Deserialize impls in inventory.rs.

## Dis-/Advantages (`advantages.rs`) — #10

- `Advantage { name, kind, cp (always positive), level, description, modifiers }`;
  narrative-only traits simply have no modifiers.
- `validate_budget()`: ≤30 CP total, or exactly ONE trait >30 plus ≤5 CP others.
- `Modifier { value, target }` with `ModifierTarget::{Attribute, Skill(name), Tag(string)}`
  (serde: adjacently tagged as `{ type = "...", of = "..." }` for TOML).
- Auto-applied: attribute modifiers in `effective_attribute()` (before wound
  penalties), skill modifiers in `check_skill()`.
- Engine-known tags: `initiative` (roll_initiative = 1d10 + eff. REF + tag),
  `prellschaden` (bruise_capacity = 5 + tag), `heilrate` (+1 double / −1 half
  healing rate; healing_progress counts quarter-days, 4 quarters = 1 damage).
- Situational tags ("hören", …): caller queries `modifier_for_tag()` when a
  fitting roll comes up.

## Character Creation (`chargen.rs`) — #9

- Validations: `validate_attributes` (60 points, 2..=9 each, INT/REF ≥ 5, no
  10 at creation), `validate_skill_caps` (one 8 / one 7 / two 6, tradeable
  1:2 downward), `validate_skill_budget` (pool = INT + REF + 40 + age points;
  skill level N costs N cumulative; advantages cost CP 1:1, disadvantages
  grant), `validate_character` aggregates. `starting_money` = 3 profession
  skills × 350 eb, Reich tag doubles the factor per level.
- `age_points`: Alterspunkte table; >32 continues +3/year (aging rules #22
  still unwritten).
- **Lifepath** (Q29): data-driven d10 tables in `data/lifepath_classic.toml`
  and `..._desaster.toml` (embedded via include_str!). Tables chain via
  `goto`. `roll_background` (family/childhood/personality once) +
  `roll_life_events` (one master roll per year over 15).
  The classic file is a BEST-EFFORT transcription (core book is a text-less
  scan) — marked for review; the desaster file starts as a copy to edit.
- `generate_nsc(name, role, age, variant, roller)`: valid random attributes +
  lifepath; skills/gear stay empty (GM flavor). CLI:
  `cargo run -- chargen [--classic|--desaster] [name] [role] [age]`.
- **Trait catalog** (Q30): `advantage_catalog()` loads all wiki traits from
  `data/advantages.toml`; GUESS comments mark my modifier interpretations,
  ENGINE-TODO marks rules needing engine hooks (half-Prellschaden, RUN factor,
  Pech's always-fail-on-1, Bluter). Leveled traits encoded at smallest level.

## Quick Reference

| What | Where |
|------|-------|
| Character stats | `character.rs` → `Character`, `Attribute` |
| Skills | `character.rs` → `Skill`, `List` |
| Dice & checks | `dice.rs` → `skill_check`, `open_roll`, `DieRoller`, `Difficulty` |
| Inventory system | `inventory.rs` → `Inventory`, `Item`, `InventoryItem` |
| Armor & hit zones | `armor.rs` → `Armor`, `HitZone` |
