# AI-README: RustedPunk Project Structure

## Overview
Cyberpunk 2020 RPG character management system. Currency: `eb` = Eurobucks.

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
- Uses `InventoryItem.as_any()` for downcasting trait objects to concrete `Armor` type
- Damage will process layers from outside-in (reverse iteration) - TODO in `Character.hit()`

---

## Quick Reference

| What | Where |
|------|-------|
| Character stats | `character.rs` → `Character`, `Attribute` |
| Skills | `character.rs` → `Skill`, `List` |
| Inventory system | `inventory.rs` → `Inventory`, `Item`, `InventoryItem` |
| Armor & hit zones | `armor.rs` → `Armor`, `HitZone` |
