use crate::inventory::InventoryItem;
use crate::inventory::Item;
use std::fmt;

pub struct Armor {
    pub item: Item,
    pub protection_max: usize,
    pub protection_current: usize,
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
        write!(
            f,
            "{} SP: {}/{}",
            self.item, self.protection_current, self.protection_max
        )
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
    ) -> Self {
        Armor {
            item: Item::new(name, amount, weight_grams, price_eb, comment),
            protection_max,
            protection_current: protection_max,
        }
    }
}
