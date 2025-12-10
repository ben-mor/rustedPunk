use std::fmt;

pub trait InventoryItem: fmt::Display {
    fn get_item(&self) -> &Item;
    fn get_item_mut(&mut self) -> &mut Item;
}

pub struct Inventory {
    items: Vec<Box<dyn InventoryItem>>,
}

pub struct Item {
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
        name: String,
        amount: usize,
        weight_grams: usize,
        price_eb: usize,
        comment: String,
    ) -> Self {
        Item {
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
