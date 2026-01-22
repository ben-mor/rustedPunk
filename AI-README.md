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
- `protection_max: usize` - Maximum stopping power (SP)
- `protection_current: HashMap<HitZone, usize>` - Tracks per-zone degradation (WIP)

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

## Character Damage Mechanics (`character.rs`)

### `Character.hit(damage, zone, damage_type)` - IMPLEMENTED
- Processes damage through all worn armor layers (outside-in, reverse iteration)
- Each layer's `Armor.hit()` returns remaining damage to pass to next layer
- Applies 20% blunt trauma house rule: `absorbed_damage / 5` added back as damage
- Calls `take_damage()` with final remaining damage after all armor

### `Character.take_damage(damage, zone)` - IMPLEMENTED
- Bypasses armor, applies damage directly to character
- Applies Body Type Modifier (BTM) with minimum 1 damage rule (CP2020 RAW)
- Uses `saturating_sub()` to prevent underflow when BTM > damage
- **Head damage doubles** after BTM calculation
- **Crippling damage** (8+ points after BTM):
  - **Critical zones (Head/Chest/Vitals)**: Instant death, sets `current_damage = 100`
  - **Other zones**: Mortal 0 state, minimum `current_damage = 13` (doesn't reduce if already higher)
- Updates `damage_notes` with death/crippling messages

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

## Quick Reference

| What | Where |
|------|-------|
| Character stats | `character.rs` → `Character`, `Attribute` |
| Skills | `character.rs` → `Skill`, `List` |
| Inventory system | `inventory.rs` → `Inventory`, `Item`, `InventoryItem` |
| Armor & hit zones | `armor.rs` → `Armor`, `HitZone` |
