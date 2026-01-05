use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Item>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub amount: usize,
    pub weight_grams: usize,
    pub price_eb: usize,
    pub comment: String,
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(f, "{}", item)?;
        }
        write!(f, " Total weight: {}", self.calc_total_weight())
    }
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory { items: Vec::new() }
    }

    pub fn calc_total_weight(&self) -> usize {
        let mut total = 0;
        for item in &self.items {
            total += item.amount * item.weight_grams;
        }
        total
    }

    pub fn push(&mut self, item: Item) {
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
            "{} {}, {}g, {}eb \n{}",
            self.amount,
            self.name,
            self.weight_grams * self.amount,
            self.price_eb,
            self.comment
        )
    }
}
