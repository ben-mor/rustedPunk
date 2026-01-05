use crate::armor::Armor;
use std::fmt;
use uuid::Uuid;

pub trait InventoryItem: fmt::Display {
    fn get_item(&self) -> &Item;
    fn get_item_mut(&mut self) -> &mut Item;
    fn is_armor(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct Inventory {
    items: Vec<Box<dyn InventoryItem>>,
}

pub struct Item {
    pub uuid: Uuid,
    pub name: String,
    pub amount: usize,
    pub weight_grams: usize,
    pub price_eb: usize,
    pub comment: String,
}

impl InventoryItem for Item {
    fn get_item(&self) -> &Item {
        self
    }

    fn get_item_mut(&mut self) -> &mut Item {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(f, "\n\t\t{}", item)?;
        }
        write!(f, "\n\tTotal weight: {}", self.calc_total_weight())
    }
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory { items: Vec::new() }
    }

    pub fn get_all_armor(&self) -> Vec<&Armor> {
        self.items
            .iter()
            .filter_map(|item| item.as_any().downcast_ref::<Armor>())
            .collect()
    }

    pub fn get_item(&self, uuid: Uuid) -> Option<&dyn InventoryItem> {
        let item = self.items.iter().find(|item| item.get_item().uuid == uuid);
        if item.is_some() {
            return Some(item.unwrap().as_ref());
        }
        None
    }

    pub fn get_item_mut(&mut self, uuid: Uuid) -> Option<&mut dyn InventoryItem> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.get_item().uuid == uuid);
        if item.is_some() {
            return Some(item.unwrap().as_mut());
        }
        None
    }

    pub fn calc_total_weight(&self) -> usize {
        let mut total = 0;
        for item in &self.items {
            total += item.get_item().amount * item.get_item().weight_grams;
        }
        total
    }

    pub fn push(&mut self, item: Box<dyn InventoryItem>) {
        self.items.push(item);
    }
}

impl Item {
    pub fn new(
        uuid: Option<Uuid>,
        name: String,
        amount: usize,
        weight_grams: usize,
        price_eb: usize,
        comment: String,
    ) -> Self {
        Item {
            uuid: uuid.unwrap_or(Uuid::new_v4()),
            name,
            amount,
            weight_grams,
            price_eb,
            comment,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} \n\t\t\t{}\n\t\t\t{}g, {}eb",
            self.amount,
            self.name,
            self.comment,
            self.weight_grams * self.amount,
            self.price_eb
        )
    }
}
