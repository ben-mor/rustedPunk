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
                DamageType::Blunt => {
                    if self.protection_current[&zone] > remaining_damage {
                        absorbed_damage = remaining_damage;
                        remaining_damage = 0;
                    } else {
                        absorbed_damage = self.protection_current[&zone];
                        self.protection_current
                            .insert(zone, self.protection_current[&zone] - 1);
                        remaining_damage -= absorbed_damage;
                    }
                }
                DamageType::Slashing => {
                    let mut protection = self.protection_current[&zone];
                    if !self.is_hard {
                        protection = protection / 2;
                    }

                    if protection > remaining_damage {
                        absorbed_damage = remaining_damage;
                        remaining_damage = 0;
                    } else {
                        absorbed_damage = self.protection_current[&zone];
                        self.protection_current
                            .insert(zone, self.protection_current[&zone] - 1);
                        remaining_damage -= absorbed_damage;
                    }
                }
                DamageType::ArmorPiercing => {
                    let protection = self.protection_current[&zone] / 2;

                    if protection > remaining_damage {
                        absorbed_damage = remaining_damage;
                        remaining_damage = 0;
                    } else {
                        absorbed_damage = protection;
                        self.protection_current
                            .insert(zone, self.protection_current[&zone] - 1);
                        remaining_damage -= absorbed_damage;
                        remaining_damage = remaining_damage / 2;
                    }
                }
                DamageType::HollowPoint => {
                    let protection = self.protection_current[&zone];

                    if protection > remaining_damage * 2 {
                        absorbed_damage = remaining_damage;
                        remaining_damage = 0;
                    } else {
                        absorbed_damage = self.protection_current[&zone];
                        self.protection_current
                            .insert(zone, self.protection_current[&zone] - 1);
                        remaining_damage -= absorbed_damage;
                        remaining_damage = remaining_damage / 2;
                    }
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
    fn kevlar_shirt() -> Armor {
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
    fn test_armor_hit_ap_hard() {
        let mut vest = flak_vest();
        let damage = 10;
        let result = vest.hit(damage, HitZone::Chest, DamageType::ArmorPiercing);
        assert_eq!(result.remaining_damage, 0);
        assert_eq!(result.absorbed_damage, 10);
        assert_eq!(vest.protection_current.get(&HitZone::Chest), Some(&19));
    }

    #[test]
    fn test_armor_hit_ap_soft() {
        let mut vest = kevlar_shirt();
        let damage = 10;
        let result = vest.hit(damage, HitZone::Chest, DamageType::ArmorPiercing);
        assert_eq!(result.remaining_damage, 2);
        assert_eq!(result.absorbed_damage, 5);
        assert_eq!(vest.protection_current.get(&HitZone::Chest), Some(&9));
    }
}
