use crate::inventory::InventoryItem;
use crate::inventory::Item;
use crate::weapons::DamageType;
use std::collections::HashMap;
use std::fmt;

pub struct Armor {
    pub item: Item,
    pub protection_max: usize,
    pub protection_current: HashMap<HitZone, usize>,
    pub is_hard: bool,
    pub encumbrance: usize,
}

#[derive(Debug)]
pub struct DamageResult {
    pub remaining_damage: usize,
    pub absorbed_damage: usize,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum HitZone {
    Head,
    LeftHand,
    RightHand,
    LeftArm,
    RightArm,
    Shoulders,
    Chest,
    Stomach,
    Vitals,
    Thighs,
    LeftLeg,
    RightLeg,
    LeftFoot,
    RightFoot,
}

impl fmt::Display for HitZone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl InventoryItem for Armor {
    fn get_item(&self) -> &Item {
        &self.item
    }

    fn get_item_mut(&mut self) -> &mut Item {
        &mut self.item
    }
    fn is_armor(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl fmt::Display for Armor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} SP: {}", self.item, self.protection_max)?;
        for (zone, current) in &self.protection_current {
            write!(f, " {}:{}", zone, current)?;
        }
        Ok(())
    }
}

impl Armor {
    pub fn new(
        name: String,
        amount: usize,
        weight_grams: usize,
        price_eb: usize,
        comment: String,
        protection_max: usize,
        protected_zones: Vec<HitZone>,
        is_hard: bool,
        encumbrance: usize,
    ) -> Self {
        let mut protection_current = HashMap::new();
        for zone in protected_zones {
            protection_current.insert(zone, protection_max);
        }
        Armor {
            item: Item::new(None, name, amount, weight_grams, price_eb, comment),
            protection_max,
            protection_current: protection_current,
            is_hard: is_hard,
            encumbrance: encumbrance,
        }
    }

    /// Applies damage to the armor at a specific hit zone.
    ///
    /// Different damage types interact with armor in different ways:
    ///
    /// * `ArmorPiercing` - Halves effective protection, halves remaining damage that penetrates
    /// * `Blunt` - Uses full protection value, no damage modifications
    /// * `HollowPoint` - Halves incoming damage before armor calculation
    /// * `Slashing` - Hard armor protects fully. Soft armor protects half. Damage isn't halved like armor piercing.
    ///
    /// When armor protection is insufficient to stop the attack, the armor's durability
    /// is reduced by 1 for that hit zone.
    ///
    /// If the hit zone is not protected by this armor, all damage passes through unmodified.
    ///
    /// # Arguments
    ///
    /// * `damage` - The amount of incoming damage
    /// * `zone` - The hit zone being attacked
    /// * `damage_type` - The type of damage being applied
    ///
    /// # Returns
    ///
    /// A `DamageResult` containing the remaining damage and the amount absorbed by the armor
    pub fn hit(&mut self, damage: usize, zone: HitZone, damage_type: DamageType) -> DamageResult {
        if self.protection_current.contains_key(&zone) {
            let mut remaining_damage = damage;
            let absorbed_damage;
            match damage_type {
                DamageType::ArmorPiercing => {
                    absorbed_damage = self.hit_armor_piercing(zone, &mut remaining_damage);
                }
                DamageType::Blunt => {
                    absorbed_damage = self.hit_blunt(zone, &mut remaining_damage);
                }
                DamageType::HollowPoint => {
                    absorbed_damage = self.hit_hollow_point(zone, &mut remaining_damage);
                }
                DamageType::Slashing => {
                    absorbed_damage = self.hit_slashing(zone, &mut remaining_damage);
                }
            }
            DamageResult {
                remaining_damage,
                absorbed_damage,
            }
        } else {
            DamageResult {
                remaining_damage: damage,
                absorbed_damage: 0,
            }
        }
    }

    /// Armor-piercing weapons halve the effective protection of the armor.
    /// Any remaining damage is halved at the end.
    /// This function also reduces the armor's durability by 1, IF the armor has been penetrated.
    fn hit_armor_piercing(&mut self, zone: HitZone, remaining_damage: &mut usize) -> usize {
        let protection = self.protection_current[&zone] / 2;
        let absorbed_damage;

        if protection >= *remaining_damage {
            absorbed_damage = *remaining_damage;
            *remaining_damage = 0;
        } else {
            absorbed_damage = protection;
            self.protection_current
                .insert(zone, self.protection_current[&zone] - 1);
            *remaining_damage -= absorbed_damage;
            *remaining_damage = (*remaining_damage + 1) / 2;
        }
        absorbed_damage
    }

    /// Blunt weapons use the full protection value of the armor.
    /// No additional damage modifications are applied.
    /// This function also reduces the armor's durability by 1, IF the armor has been penetrated.
    fn hit_blunt(&mut self, zone: HitZone, remaining_damage: &mut usize) -> usize {
        let absorbed_damage;

        if self.protection_current[&zone] >= *remaining_damage {
            absorbed_damage = *remaining_damage;
            *remaining_damage = 0;
        } else {
            absorbed_damage = self.protection_current[&zone];
            self.protection_current
                .insert(zone, self.protection_current[&zone] - 1);
            *remaining_damage -= absorbed_damage;
        }
        absorbed_damage
    }

    /// Hollow-point weapons halve the incoming damage before armor calculation.
    /// The armor uses its full protection value against the reduced damage.
    /// This function also reduces the armor's durability by 1, IF the armor has been penetrated.
    fn hit_hollow_point(&mut self, zone: HitZone, remaining_damage: &mut usize) -> usize {
        let protection = self.protection_current[&zone];
        *remaining_damage = (*remaining_damage + 1) / 2;
        let absorbed_damage;

        if protection >= *remaining_damage {
            absorbed_damage = *remaining_damage;
            *remaining_damage = 0;
        } else {
            absorbed_damage = self.protection_current[&zone];
            self.protection_current
                .insert(zone, self.protection_current[&zone] - 1);
            *remaining_damage -= absorbed_damage;
        }
        absorbed_damage
    }

    /// Slashing weapons use full protection against hard armor.
    /// Against soft armor, the effective protection is halved.
    /// Other then armor piercing damage, the remaining damage is not halved.
    /// This function also reduces the armor's durability by 1, IF the armor has been penetrated.
    fn hit_slashing(&mut self, zone: HitZone, remaining_damage: &mut usize) -> usize {
        let mut protection = self.protection_current[&zone];
        if !self.is_hard {
            protection = (protection + 1) / 2;
        }
        let absorbed_damage;

        if protection >= *remaining_damage {
            absorbed_damage = *remaining_damage;
            *remaining_damage = 0;
        } else {
            absorbed_damage = protection;
            self.protection_current
                .insert(zone, self.protection_current[&zone] - 1);
            *remaining_damage -= absorbed_damage;
        }
        absorbed_damage
    }

    pub fn print(&self) {
        println!(
            "Armor: {} hard: {}, max: {}",
            self.item.name, self.is_hard, self.protection_max
        );
        for (zone, protection) in &self.protection_current {
            println!("    {}: {}", zone, protection);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn flak_vest() -> Armor {
        Armor::new(
            "Flak Vest".to_string(),
            2,
            1000,
            100,
            "Hard armor with 20 SP".to_string(),
            20,
            vec![
                HitZone::Chest,
                HitZone::LeftArm,
                HitZone::RightArm,
                HitZone::Vitals,
                HitZone::Stomach,
            ],
            true,
            1,
        )
    }

    pub fn kev_shirt() -> Armor {
        Armor::new(
            "Kevlar Shirt".to_string(),
            2,
            1000,
            100,
            "Soft armor with 10 SP".to_string(),
            10,
            vec![HitZone::Chest, HitZone::Vitals, HitZone::Stomach],
            false,
            0,
        )
    }

    pub fn kevlar_tights() -> Armor {
        Armor::new(
            "Kevlar Tights".to_string(),
            1,
            400,
            150,
            "Soft leg armor".to_string(),
            10,
            vec![
                HitZone::LeftLeg,
                HitZone::RightLeg,
                HitZone::LeftFoot,
                HitZone::RightFoot,
                HitZone::Thighs,
            ],
            false,
            1,
        )
    }

    pub fn braces() -> Armor {
        Armor::new(
            "Braces".to_string(),
            1,
            200,
            100,
            "Arm braces".to_string(),
            10,
            vec![HitZone::LeftArm, HitZone::RightArm],
            false,
            0,
        )
    }

    pub fn long_leather_cloak() -> Armor {
        Armor::new(
            "Long Leather Cloak".to_string(),
            1,
            1500,
            300,
            "Long protective cloak".to_string(),
            4,
            vec![
                HitZone::LeftArm,
                HitZone::RightArm,
                HitZone::Chest,
                HitZone::Shoulders,
                HitZone::Vitals,
                HitZone::Thighs,
                HitZone::Stomach,
                HitZone::LeftLeg,
                HitZone::RightLeg,
            ],
            false,
            2,
        )
    }

    pub fn leather_boots() -> Armor {
        Armor::new(
            "Leather Boots".to_string(),
            1,
            800,
            100,
            "Hard leather boots".to_string(),
            4,
            vec![HitZone::LeftFoot, HitZone::RightFoot],
            true,
            0,
        )
    }

    pub fn helmet() -> Armor {
        Armor::new(
            "Helmet".to_string(),
            1,
            1000,
            250,
            "Hard head protection".to_string(),
            15,
            vec![HitZone::Head],
            true,
            1,
        )
    }

    #[test]
    fn test_armor_hits() {
        test_armor_hit(&mut flak_vest(), 10, DamageType::ArmorPiercing, 0, 10, 20);
        test_armor_hit(&mut kev_shirt(), 10, DamageType::ArmorPiercing, 3, 5, 9);
        test_armor_hit(&mut flak_vest(), 10, DamageType::Blunt, 0, 10, 20);
        test_armor_hit(&mut kev_shirt(), 10, DamageType::Blunt, 0, 10, 10);
        test_armor_hit(&mut flak_vest(), 10, DamageType::HollowPoint, 0, 5, 20);
        test_armor_hit(&mut kev_shirt(), 10, DamageType::HollowPoint, 0, 5, 10);
        test_armor_hit(&mut flak_vest(), 10, DamageType::Slashing, 0, 10, 20);
        test_armor_hit(&mut kev_shirt(), 10, DamageType::Slashing, 5, 5, 9);

        test_armor_hit(&mut flak_vest(), 7, DamageType::ArmorPiercing, 0, 7, 20);
        test_armor_hit(&mut kev_shirt(), 7, DamageType::ArmorPiercing, 1, 5, 9);
        test_armor_hit(&mut flak_vest(), 7, DamageType::Blunt, 0, 7, 20);
        test_armor_hit(&mut kev_shirt(), 7, DamageType::Blunt, 0, 7, 10);
        test_armor_hit(&mut flak_vest(), 7, DamageType::HollowPoint, 0, 4, 20);
        test_armor_hit(&mut kev_shirt(), 7, DamageType::HollowPoint, 0, 4, 10);
        test_armor_hit(&mut flak_vest(), 7, DamageType::Slashing, 0, 7, 20);
        test_armor_hit(&mut kev_shirt(), 7, DamageType::Slashing, 2, 5, 9);

        test_armor_hit(&mut flak_vest(), 29, DamageType::ArmorPiercing, 10, 10, 19);
        test_armor_hit(&mut kev_shirt(), 29, DamageType::ArmorPiercing, 12, 5, 9);
        test_armor_hit(&mut flak_vest(), 29, DamageType::Blunt, 9, 20, 19);
        test_armor_hit(&mut kev_shirt(), 29, DamageType::Blunt, 19, 10, 9);
        test_armor_hit(&mut flak_vest(), 29, DamageType::HollowPoint, 0, 15, 20);
        test_armor_hit(&mut flak_vest(), 49, DamageType::HollowPoint, 5, 20, 19);
        test_armor_hit(&mut kev_shirt(), 29, DamageType::HollowPoint, 5, 10, 9);
        test_armor_hit(&mut flak_vest(), 29, DamageType::Slashing, 9, 20, 19);
        test_armor_hit(&mut kev_shirt(), 29, DamageType::Slashing, 24, 5, 9);

        test_armor_hit(&mut flak_vest(), 12, DamageType::ArmorPiercing, 1, 10, 19);
        test_armor_hit(&mut kev_shirt(), 12, DamageType::ArmorPiercing, 4, 5, 9);
        test_armor_hit(&mut flak_vest(), 12, DamageType::Blunt, 0, 12, 20);
        test_armor_hit(&mut kev_shirt(), 12, DamageType::Blunt, 2, 10, 9);
        test_armor_hit(&mut flak_vest(), 12, DamageType::HollowPoint, 0, 6, 20);
        test_armor_hit(&mut kev_shirt(), 12, DamageType::HollowPoint, 0, 6, 10);
        test_armor_hit(&mut flak_vest(), 12, DamageType::Slashing, 0, 12, 20);
        test_armor_hit(&mut kev_shirt(), 12, DamageType::Slashing, 7, 5, 9);

        test_armor_hit(&mut flak_vest(), 13, DamageType::ArmorPiercing, 2, 10, 19);
        test_armor_hit(&mut kev_shirt(), 13, DamageType::ArmorPiercing, 4, 5, 9);
        test_armor_hit(&mut flak_vest(), 13, DamageType::Blunt, 0, 13, 20);
        test_armor_hit(&mut kev_shirt(), 13, DamageType::Blunt, 3, 10, 9);
        test_armor_hit(&mut flak_vest(), 13, DamageType::HollowPoint, 0, 7, 20);
        test_armor_hit(&mut kev_shirt(), 13, DamageType::HollowPoint, 0, 7, 10);
        test_armor_hit(&mut flak_vest(), 13, DamageType::Slashing, 0, 13, 20);
        test_armor_hit(&mut kev_shirt(), 13, DamageType::Slashing, 8, 5, 9);

        test_armor_hit(&mut flak_vest(), 0, DamageType::ArmorPiercing, 0, 0, 20);
        test_armor_hit(&mut kev_shirt(), 0, DamageType::ArmorPiercing, 0, 0, 10);
        test_armor_hit(&mut flak_vest(), 0, DamageType::Blunt, 0, 0, 20);
        test_armor_hit(&mut kev_shirt(), 0, DamageType::Blunt, 0, 0, 10);
        test_armor_hit(&mut flak_vest(), 0, DamageType::HollowPoint, 0, 0, 20);
        test_armor_hit(&mut kev_shirt(), 0, DamageType::HollowPoint, 0, 0, 10);
        test_armor_hit(&mut flak_vest(), 0, DamageType::Slashing, 0, 0, 20);
        test_armor_hit(&mut kev_shirt(), 0, DamageType::Slashing, 0, 0, 10);
    }

    #[test]
    fn test_armor_hit_nowhere() {
        let mut armor = kev_shirt();
        let damage_result = armor.hit(1000, HitZone::Head, DamageType::ArmorPiercing);
        assert_eq!(damage_result.remaining_damage, 1000);
        assert_eq!(damage_result.absorbed_damage, 0);
        assert_eq!(armor.protection_current.get(&HitZone::Chest), Some(&10));
    }

    fn test_armor_hit(
        armor: &mut Armor,
        damage: usize,
        damage_type: DamageType,
        expected_remaining_damage: usize,
        expected_absorbed_damage: usize,
        expected_remaining_protection: usize,
    ) {
        let context = format!("{} {:?} against {}", damage, damage_type, armor.item.name);
        let result = armor.hit(damage, HitZone::Chest, damage_type);
        assert_eq!(
            result.remaining_damage, expected_remaining_damage,
            "{}: expected {} remaining damage, but was {}",
            context, expected_remaining_damage, result.remaining_damage
        );
        assert_eq!(
            result.absorbed_damage, expected_absorbed_damage,
            "{}: expected {} absorbed damage, but was {}",
            context, expected_absorbed_damage, result.absorbed_damage
        );
        assert_eq!(
            armor.protection_current.get(&HitZone::Chest),
            Some(&expected_remaining_protection),
            "{}: expected {} remaining protection, but was {:?}",
            context,
            expected_remaining_protection,
            armor.protection_current.get(&HitZone::Chest)
        );
    }
}
