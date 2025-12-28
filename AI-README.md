# AI-README: RustedPunk Project Structure

## Overview
Cyberpunk 2020/RED RPG character management system. Currency: `eb` = Eurobucks.

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

## Quick Reference

| What | Where |
|------|-------|
| Character stats | `character.rs` → `Character`, `Attribute` |
| Skills | `character.rs` → `Skill`, `List` |
| Inventory system | `inventory.rs` → `Inventory`, `Item`, `InventoryItem` |
| Armor & hit zones | `armor.rs` → `Armor`, `HitZone` |