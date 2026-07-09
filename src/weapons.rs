use crate::dice::DiceExpr;
use crate::inventory::{InventoryItem, Item};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum DamageType {
    Blunt,
    Slashing,
    ArmorPiercing,
    HollowPoint,
    /// Fire (Molotov etc.): ignores the ">8 damage" crippling rule.
    Fire,
}

impl fmt::Display for DamageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Weapon categories per issue #21.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum WeaponCategory {
    Knife,
    Club,
    Axe,
    Spear,
    BrassKnuckles,
    Pistol,
    Smg,
    Rifle,
    Shotgun,
    Bow,
    RocketLauncher,
    Sling,
    Shuriken,
    Molotov,
}

impl WeaponCategory {
    /// Melee weapons add the wielder's DAM to damage and map to a
    /// melee weapon class for the skill roll.
    pub fn melee_class(self) -> Option<crate::melee::MeleeClass> {
        use crate::melee::MeleeClass;
        match self {
            WeaponCategory::Knife | WeaponCategory::BrassKnuckles => Some(MeleeClass::Short),
            WeaponCategory::Club | WeaponCategory::Axe => Some(MeleeClass::Medium),
            WeaponCategory::Spear => Some(MeleeClass::Long),
            _ => None,
        }
    }

    /// Firearms are subject to the penetration cap (4 + 1d10 before the
    /// shot exits through the back).
    pub fn is_gunshot(self) -> bool {
        matches!(
            self,
            WeaponCategory::Pistol
                | WeaponCategory::Smg
                | WeaponCategory::Rifle
                | WeaponCategory::Shotgun
        )
    }
}

/// CP2020 concealability rating.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Concealability {
    /// P — pocket, pants leg or sleeve.
    Pocket,
    /// J — jacket, coat or shoulder rig.
    Jacket,
    /// L — long coat.
    LongCoat,
    /// N — can't be hidden.
    NoHide,
}

/// CP2020 availability rating; the workshop trading system maps this
/// to token difficulties (common 10, rare 20, legendary 40).
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Availability {
    /// E — excellent, can be found almost anywhere.
    Excellent,
    /// C — common, most stores and black marketeers.
    Common,
    /// P — poor, specialty stores, black market.
    Poor,
    /// R — rare, custom order, stolen or very hard to find.
    Rare,
}

/// CP2020 reliability rating (matters on fumbles with autofire etc.).
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Reliability {
    VeryReliable,
    Standard,
    Unreliable,
}

/// A weapon, stats per CP2020 Reference Book 5 (see PROJECT-STRUCTURE §3).
// Field order matters for TOML: the table-like `item` must serialize last.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub category: WeaponCategory,
    /// Weapon accuracy (WA), added to attack rolls.
    pub weapon_accuracy: i32,
    pub concealability: Concealability,
    pub availability: Availability,
    pub damage: DiceExpr,
    pub damage_type: DamageType,
    /// Magazine size; 0 for melee weapons.
    pub magazine: i32,
    /// Maximum shots per round; 0 for melee weapons.
    pub rate_of_fire: i32,
    pub reliability: Reliability,
    /// Effective range in meters; melee weapons use their reach here.
    pub range_m: i32,
    pub silencer: bool,
    pub scope: bool,
    pub item: Item,
}

impl InventoryItem for Weapon {
    fn get_item(&self) -> &Item {
        &self.item
    }

    fn get_item_mut(&mut self) -> &mut Item {
        &mut self.item
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn equals(&self, other: &dyn InventoryItem) -> bool {
        if let Some(other_weapon) = other.as_any().downcast_ref::<Weapon>() {
            self == other_weapon
        } else {
            false
        }
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({:?}, WA {:+}, {} {})",
            self.item, self.category, self.weapon_accuracy, self.damage, self.damage_type
        )
    }
}

/// The noise bonus for hearing a shot (roll vs 15), per the Hausregeln
/// formula: average damage × 3, −2 per wall, −6 per soundproofed wall,
/// −2 per 50 m distance, +2 per bullet fired that round. A silencer halves
/// the initial loudness. Direction is known if the check succeeds by 10+.
pub fn shot_noise_bonus(
    weapon: &Weapon,
    bullets_this_round: i32,
    distance_m: i32,
    walls: i32,
    soundproofed_walls: i32,
) -> i32 {
    let mut initial = weapon.damage.average() * 3;
    if weapon.silencer {
        initial /= 2;
    }
    initial - 2 * walls - 6 * soundproofed_walls - 2 * (distance_m / 50) + 2 * bullets_this_round
}

impl Weapon {
    #[allow(clippy::too_many_arguments)] // catalog constructor, named per column
    fn catalog_entry(
        name: &str,
        category: WeaponCategory,
        weapon_accuracy: i32,
        concealability: Concealability,
        availability: Availability,
        damage: &str,
        damage_type: DamageType,
        magazine: i32,
        rate_of_fire: i32,
        reliability: Reliability,
        range_m: i32,
        price_eb: i32,
        weight_grams: i32,
        comment: &str,
    ) -> Weapon {
        Weapon {
            item: Item::new(
                None,
                name.to_string(),
                1,
                weight_grams,
                price_eb,
                comment.to_string(),
            ),
            category,
            weapon_accuracy,
            concealability,
            availability,
            damage: damage.parse().expect("catalog dice expression"),
            damage_type,
            magazine,
            rate_of_fire,
            reliability,
            range_m,
            silencer: false,
            scope: false,
        }
    }

    /// One middle-of-the-road CP2020 representative per category from
    /// issue #21 (Q27). Stats from Reference Book 5; weights are estimates.
    /// Sling and Molotov have no RB5 entries — best-guess stats, marked
    /// in the comment for review.
    pub fn catalog() -> Vec<Weapon> {
        use Availability::*;
        use Concealability::*;
        use Reliability::*;
        use WeaponCategory::*;
        vec![
            Weapon::catalog_entry(
                "Combat Knife", Knife, 1, Jacket, Poor, "1d6+3",
                DamageType::Slashing, 0, 0, VeryReliable, 1, 70, 400,
                "RB5: MEL +1 J P 1d6+3 (PAC)",
            ),
            Weapon::catalog_entry(
                "Club", Club, 0, LongCoat, Common, "1d6",
                DamageType::Blunt, 0, 0, VeryReliable, 1, 2, 800,
                "RB5: MEL +0 L C 1d6 (CP20)",
            ),
            Weapon::catalog_entry(
                "Axe", Axe, -1, NoHide, Common, "2d6+3",
                DamageType::Slashing, 0, 0, VeryReliable, 1, 20, 1500,
                "RB5: MEL -1 N C 2d6+3 (CP20)",
            ),
            Weapon::catalog_entry(
                "Speer", Spear, 0, NoHide, Poor, "3d6",
                DamageType::Slashing, 0, 0, VeryReliable, 2, 95, 2000,
                "RB5: Fang Tian Ji, MEL +0 N P 3d6 2m (PAC)",
            ),
            Weapon::catalog_entry(
                "Brass Knuckles", BrassKnuckles, 0, Pocket, Common, "1d6+2",
                DamageType::Blunt, 0, 0, VeryReliable, 1, 10, 250,
                "RB5: Punch +0 P C 1d6+2 (CP20)",
            ),
            Weapon::catalog_entry(
                "Militech Arms Avenger", Pistol, 0, Jacket, Excellent, "2d6+1",
                DamageType::Blunt, 10, 2, VeryReliable, 50, 250, 1000,
                "RB5: P +0 J E 2d6+1 (9mm) 10/2 VR 50m (CP20)",
            ),
            Weapon::catalog_entry(
                "Uzi Miniauto 9", Smg, 1, Jacket, Excellent, "2d6+1",
                DamageType::Blunt, 30, 35, VeryReliable, 150, 475, 3000,
                "RB5: SMG +1 J E 2d6+1 (9mm) 30/35 VR 150m (CP20)",
            ),
            Weapon::catalog_entry(
                "AK-47", Rifle, 0, NoHide, Excellent, "5d6",
                DamageType::ArmorPiercing, 30, 20, VeryReliable, 400, 300, 4300,
                "RB5: RIF +0 N E 5d6 (7.62sov) 30/20 VR 400m (CP20); Technlogien-Seite",
            ),
            Weapon::catalog_entry(
                "Ithaca Stakeout", Shotgun, -1, LongCoat, Common, "4d6",
                DamageType::Blunt, 8, 2, Standard, 50, 200, 3200,
                "RB5: SHT -1 L C 4d6 (12ga) 8/2 ST 50m (CP13)",
            ),
            Weapon::catalog_entry(
                "TomKatt Hunting Bow", Bow, 0, NoHide, Common, "4d6",
                DamageType::ArmorPiercing, 12, 1, VeryReliable, 150, 150, 1400,
                "RB5: BOW +0 N C 4d6 12/1 VR 150m (CGen)",
            ),
            Weapon::catalog_entry(
                "LAW", RocketLauncher, -2, LongCoat, Poor, "4d10",
                DamageType::ArmorPiercing, 1, 1, VeryReliable, 200, 300, 2500,
                "RB5: HVY -2 L P 4d10 HEAT 2m radius, single use (MM)",
            ),
            Weapon::catalog_entry(
                "Schleuder", Sling, -1, Pocket, Excellent, "1d6",
                DamageType::Blunt, 1, 1, VeryReliable, 50, 1, 100,
                "BEST GUESS (kein RB5-Eintrag): improvisierte Steinschleuder — bitte reviewen",
            ),
            Weapon::catalog_entry(
                "Bo-Shuriken", Shuriken, 0, Pocket, Common, "1d6",
                DamageType::ArmorPiercing, 1, 1, VeryReliable, 10, 5, 50,
                "RB5: MEL +0 P C 1d6 Throw (PAC)",
            ),
            Weapon::catalog_entry(
                "Molotov-Cocktail", Molotov, -1, Jacket, Excellent, "2d10",
                DamageType::Fire, 1, 1, Unreliable, 20, 5, 900,
                "BEST GUESS (kein RB5-Eintrag): 2d10 Feuer im 2m-Radius, brennt weiter — bitte reviewen",
            ),
        ]
    }
}

/// Autofire (house rule): +1 on the attack per 10 bullets when close,
/// −1 per 10 when far; on a success one bullet hits per point above the
/// target, capped at the bullets fired. No precision bonuses apply.
pub fn autofire_attack_modifier(bullets: i32, close: bool) -> i32 {
    let per_ten = bullets / 10;
    if close {
        per_ten
    } else {
        -per_ten
    }
}

/// Bullets that hit after a successful autofire attack.
pub fn autofire_hits(total: i32, target: i32, bullets: i32) -> i32 {
    (total - target).max(0).min(bullets)
}

/// Three-round burst (house rule): +3 on the attack, single target only;
/// on a success 1d2 bullets hit.
pub const BURST_ATTACK_BONUS: i32 = 3;

pub fn burst_hits(roller: &mut dyn crate::dice::DieRoller) -> i32 {
    roller.die(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::SequenceRoller;

    #[test]
    fn test_damage_type_serialization() {
        let damage_type = DamageType::ArmorPiercing;
        let serialized = toml::to_string(&damage_type).unwrap();
        let deserialized: DamageType = toml::from_str(&serialized).unwrap();
        assert_eq!(damage_type, deserialized);
    }

    #[test]
    fn test_catalog_covers_all_categories() {
        let catalog = Weapon::catalog();
        assert_eq!(catalog.len(), 14);
        for category in [
            WeaponCategory::Knife,
            WeaponCategory::Club,
            WeaponCategory::Axe,
            WeaponCategory::Spear,
            WeaponCategory::BrassKnuckles,
            WeaponCategory::Pistol,
            WeaponCategory::Smg,
            WeaponCategory::Rifle,
            WeaponCategory::Shotgun,
            WeaponCategory::Bow,
            WeaponCategory::RocketLauncher,
            WeaponCategory::Sling,
            WeaponCategory::Shuriken,
            WeaponCategory::Molotov,
        ] {
            assert!(
                catalog.iter().any(|weapon| weapon.category == category),
                "missing category {:?}",
                category
            );
        }
    }

    #[test]
    fn test_shot_noise_matches_wiki_examples() {
        let catalog = Weapon::catalog();
        let ak = catalog
            .iter()
            .find(|weapon| weapon.item.name == "AK-47")
            .unwrap();
        // wiki: AK (5d6) at 200 m -> 45 - 8 = 37... wiki says 38 with -2*4;
        // 200/50 = 4 walls-equivalents; 45 - 8 = 37. The wiki example counts
        // "(-2*4) = 38" which is arithmetically off by one (45-8=37);
        // we implement the formula, not the typo.
        assert_eq!(shot_noise_bonus(ak, 1, 200, 0, 0), 45 - 8 + 2);
        // silencer halves the initial 45 -> 22 (integer)
        let mut silenced = ak.clone();
        silenced.silencer = true;
        assert_eq!(shot_noise_bonus(&silenced, 1, 0, 0, 0), 22 + 2);
    }

    #[test]
    fn test_autofire_math() {
        assert_eq!(autofire_attack_modifier(30, true), 3);
        assert_eq!(autofire_attack_modifier(30, false), -3);
        assert_eq!(autofire_attack_modifier(9, true), 0);
        // total 22 vs target 15 with 5 bullets: capped at 5
        assert_eq!(autofire_hits(22, 15, 5), 5);
        assert_eq!(autofire_hits(18, 15, 30), 3);
        assert_eq!(autofire_hits(14, 15, 30), 0);
    }

    #[test]
    fn test_burst_hits() {
        let mut roller = SequenceRoller::new(vec![2]);
        assert_eq!(burst_hits(&mut roller), 2);
    }

    #[test]
    fn test_weapon_serialization() {
        let weapon = Weapon::catalog().remove(0);
        let serialized = toml::to_string(&weapon).unwrap();
        let deserialized: Weapon = toml::from_str(&serialized).unwrap();
        assert_eq!(weapon, deserialized);
    }
}
