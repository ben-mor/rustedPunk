use crate::armor::Armor;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use uuid::Uuid;

/// IMPORTANT: When adding new types that implement InventoryItem, you MUST also:
/// 1. Add a variant to SerializableInventoryItem
/// 2. Handle it in Inventory's Serialize impl
/// 3. Handle it in Inventory's Deserialize impl
pub trait InventoryItem: fmt::Display {
    fn get_item(&self) -> &Item;
    fn get_item_mut(&mut self) -> &mut Item;
    fn is_armor(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn equals(&self, other: &dyn InventoryItem) -> bool;
}

pub struct Inventory {
    items: Vec<Box<dyn InventoryItem>>,
}

impl fmt::Debug for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inventory")
            .field("items", &self.items.len()) // Can't debug trait objects easily
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
struct SerializeableInventory {
    items: Vec<SerializeableInventoryItem>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum SerializeableInventoryItem {
    BasicItem(Item),
    ArmorItem(Armor),
}

impl PartialEq for Inventory {
    fn eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len()
            && self
                .items
                .iter()
                .zip(other.items.iter())
                .all(|(a, b)| a.equals(b.as_ref()))
    }
}

impl Eq for Inventory {}

impl Serialize for Inventory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializable_items: Vec<SerializeableInventoryItem> = Vec::new();

        for item in &self.items {
            if let Some(armor) = item.as_any().downcast_ref::<Armor>() {
                serializable_items.push(SerializeableInventoryItem::ArmorItem(armor.clone()));
            } else if let Some(basic) = item.as_any().downcast_ref::<Item>() {
                serializable_items.push(SerializeableInventoryItem::BasicItem(basic.clone()));
            } else {
                // Unknown type - shouldn't happen, but handle it
                panic!("Unknown inventory item type!");
            }
        }

        let container = SerializeableInventory {
            items: serializable_items,
        };
        container.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Inventory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let container = SerializeableInventory::deserialize(deserializer).unwrap();

        let mut items: Vec<Box<dyn InventoryItem>> = Vec::new();

        for item in container.items {
            match item {
                SerializeableInventoryItem::BasicItem(basic) => items.push(Box::new(basic)),
                SerializeableInventoryItem::ArmorItem(armor) => items.push(Box::new(armor)),
            }
        }

        Ok(Inventory { items })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "UncheckedItem")]
pub struct Item {
    pub uuid: Uuid,
    pub name: String,
    pub amount: i32,
    pub weight_grams: i32,
    pub price_eb: i32,
    pub comment: String,
}

/// Mirror of `Item` without validation, used as the deserialization input.
/// `Item` itself can only be produced through `TryFrom`, which rejects
/// negative quantities with a meaningful error message.
#[derive(Deserialize)]
struct UncheckedItem {
    uuid: Uuid,
    name: String,
    amount: i32,
    weight_grams: i32,
    price_eb: i32,
    comment: String,
}

impl TryFrom<UncheckedItem> for Item {
    type Error = String;

    fn try_from(raw: UncheckedItem) -> Result<Self, Self::Error> {
        for (field, value) in [
            ("amount", raw.amount),
            ("weight_grams", raw.weight_grams),
            ("price_eb", raw.price_eb),
        ] {
            if value < 0 {
                return Err(format!(
                    "Item '{}': {} must not be negative, got {}",
                    raw.name, field, value
                ));
            }
        }
        Ok(Item {
            uuid: raw.uuid,
            name: raw.name,
            amount: raw.amount,
            weight_grams: raw.weight_grams,
            price_eb: raw.price_eb,
            comment: raw.comment,
        })
    }
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

    fn equals(&self, other: &dyn InventoryItem) -> bool {
        if let Some(other_item) = other.as_any().downcast_ref::<Item>() {
            self == other_item
        } else {
            false
        }
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(f, "\n\t\t{}", item)?;
        }
        write!(f, "\n\tTotal weight: {}", self.calculate_total_weight())
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
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
        self.items
            .iter()
            .find(|item| item.get_item().uuid == uuid)
            .map(|item| item.as_ref())
    }

    pub fn get_item_mut(&mut self, uuid: Uuid) -> Option<&mut dyn InventoryItem> {
        self.items
            .iter_mut()
            .find(|item| item.get_item().uuid == uuid)
            .map(|item| &mut **item as &mut dyn InventoryItem)
    }

    pub fn calculate_total_weight(&self) -> i32 {
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
    /// Creates a new item.
    ///
    /// # Panics
    ///
    /// Panics if `amount`, `weight_grams` or `price_eb` is negative.
    pub fn new(
        uuid: Option<Uuid>,
        name: String,
        amount: i32,
        weight_grams: i32,
        price_eb: i32,
        comment: String,
    ) -> Self {
        for (field, value) in [
            ("amount", amount),
            ("weight_grams", weight_grams),
            ("price_eb", price_eb),
        ] {
            assert!(
                value >= 0,
                "Item '{}': {} must not be negative, got {}",
                name,
                field,
                value
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_serialization() {
        let item = Item::new(
            None,
            "Broomstick".to_string(),
            1,
            1500,
            0,
            "Test item".to_string(),
        );
        let serialized = toml::to_string(&item).unwrap();
        let deserialized: Item = toml::from_str(&serialized).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    #[should_panic(expected = "Item 'Broomstick': weight_grams must not be negative, got -1500")]
    fn test_item_new_rejects_negative_weight() {
        Item::new(
            None,
            "Broomstick".to_string(),
            1,
            -1500,
            0,
            "Test item".to_string(),
        );
    }

    #[test]
    fn test_item_deserialization_rejects_negative_amount() {
        let toml_str = format!(
            "uuid = \"{}\"\nname = \"Ammo 9mm\"\namount = -30\nweight_grams = 10\nprice_eb = 1\ncomment = \"\"\n",
            Uuid::new_v4()
        );
        let result: Result<Item, _> = toml::from_str(&toml_str);
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("Item 'Ammo 9mm': amount must not be negative, got -30"),
            "unexpected error message: {}",
            error
        );
    }

    fn create_simple_inventory() -> Inventory {
        let mut inv = Inventory::new();
        inv.push(Box::new(Item::new(
            None,
            "Broomstick".to_string(),
            1,
            1500,
            0,
            "Test item".to_string(),
        )));

        inv
    }

    fn create_simple_inventory_with_armor() -> Inventory {
        use crate::armor::tests::flak_vest;
        let mut inv = create_simple_inventory();
        inv.push(Box::new(flak_vest()));
        inv.push(Box::new(flak_vest()));

        inv
    }

    #[test]
    fn test_inventory_serialization() {
        let inv = create_simple_inventory();

        let serialized = toml::to_string(&inv).unwrap();
        println!("serialized: {}", serialized);
        let deserialized: Inventory = toml::from_str(&serialized).unwrap();
        assert_eq!(inv, deserialized);
    }

    #[test]
    fn test_calc_weight() {
        let inv = create_simple_inventory_with_armor();

        let total_weight = inv.calculate_total_weight();
        assert_eq!(total_weight, 1500 + 1000 + 1000); // one broomstick and two flak vests
    }
}
