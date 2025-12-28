use crate::inventory::InventoryItem;
use crate::inventory::Item;
use std::collections::HashMap;
use std::fmt;

pub struct Armor {
    pub item: Item,
    pub protection_max: usize,
    pub protection_current: HashMap<HitZone, usize>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
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
    ) -> Self {
        let mut protection_current = HashMap::new();
        for zone in protected_zones {
            protection_current.insert(zone, protection_max);
        }
        Armor {
            item: Item::new(name, amount, weight_grams, price_eb, comment),
            protection_max,
            protection_current: protection_current,
        }
    }
}
