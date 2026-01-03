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
            item: Item::new(name, amount, weight_grams, price_eb, comment),
            protection_max,
            protection_current: protection_current,
            is_hard: is_hard,
            encumbrance: encumbrance,
        }
    }

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
mod tests {
    use super::*;

    fn flak_vest() -> Armor {
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
    fn kev_shirt() -> Armor {
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
