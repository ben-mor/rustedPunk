use crate::armor::HitZone;
use crate::{inventory::Inventory, DamageType};
use std::fmt;
use uuid::Uuid;

pub struct Character {
    pub name: String,
    pub role: String,
    pub age: usize,
    pub att: Attribute,
    pub mov: Attribute,
    pub coo: Attribute,
    pub emp: Attribute,
    pub luck: Attribute,
    pub int: Attribute,
    pub body: Attribute,
    pub refl: Attribute,
    pub tec: Attribute,
    pub inventory: Inventory,
    pub worn_armor: Vec<Uuid>,
}

pub struct Attribute {
    pub base: usize,
    pub actual: usize,
}

impl Attribute {
    pub fn new(actual: usize, base: usize) -> Self {
        Attribute { base, actual }
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.actual, self.base)
    }
}

impl Character {
    pub fn new(
        name: String,
        role: String,
        age: usize,
        att: usize,
        mov: usize,
        coo: usize,
        emp: usize,
        luck: usize,
        int: usize,
        body: usize,
        refl: usize,
        tec: usize,
    ) -> Self {
        Character {
            name,
            role,
            age,
            att: Attribute::new(att, att),
            mov: Attribute::new(mov, mov),
            coo: Attribute::new(coo, coo),
            emp: Attribute::new(emp, emp),
            luck: Attribute::new(luck, luck),
            int: Attribute::new(int, int),
            body: Attribute::new(body, body),
            refl: Attribute::new(refl, refl),
            tec: Attribute::new(tec, tec),
            inventory: Inventory::new(),
            worn_armor: Vec::new(),
        }
    }

    pub fn print(&self) {
        println!(
            "\
Character {{ \n\
\tname: {}\n\
\trole: {}\n\
\tage: {}\n\
\tatt: {}\n\
\tmov: {}\n\
\tcoo: {}\n\
\temp: {}\n\
\tluck: {}\n\
\tint: {}\n\
\tbody: {}\n\
\tref: {}\n\
\ttec: {}\n\
\tInventory: {}\n\
}}",
            self.name,
            self.role,
            self.age,
            self.att,
            self.mov,
            self.coo,
            self.emp,
            self.luck,
            self.int,
            self.body,
            self.refl,
            self.tec,
            self.inventory,
        );
    }

    pub fn wear_armor(&mut self, armor_uuid: Uuid, underneath: Option<Uuid>) {
        if self.inventory.get_item(armor_uuid).is_none() {
            panic!("Armor_uuid not found in inventory");
        }
        if underneath.is_some() {
            if self.inventory.get_item(underneath.unwrap()).is_none() {
                panic!("underneath_uuid not found in inventory");
            }
            if let Some(index) = self
                .worn_armor
                .iter()
                .position(|&uuid| uuid == underneath.unwrap())
            {
                // Insert the new armor at that index (pushes existing armor one position higher)
                self.worn_armor.insert(index, armor_uuid);
            } else {
                self.worn_armor.push(armor_uuid);
            }
        } else {
            self.worn_armor.push(armor_uuid);
        }
    }

    #[allow(unused_variables)]
    pub fn hit(&mut self, damage: usize, zone: HitZone, damage_type: DamageType) {
        todo!();
    }
}

pub struct Skill {
    pub name: String,
    pub base: usize,
    pub level: usize,
    pub level_up_modifierer: usize,
}

impl Skill {
    pub fn print(self) {
        println!(
            "Skillname {} {{
            \ttotal: {}
            \tbase: {}
            \tlevel: {}
            \tlevel up modifeier: {}
        }}",
            self.name,
            self.base + self.level,
            self.base,
            self.level,
            self.level_up_modifierer
        )
    }

    pub fn new(name: String, base: usize, level: usize, level_up_modifierer: usize) -> Self {
        Skill {
            name,
            base,
            level,
            level_up_modifierer,
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {} total: {}", self.name, self.level + self.base)
    }
}

pub struct List(pub Vec<Skill>);

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Extract the value using tuple indexing,
        // and create a reference to `vec`.
        let vec = &self.0;

        write!(f, "")?;

        // Iterate over `v` in `vec` while enumerating the iteration
        // count in `count`.
        for (count, v) in vec.iter().enumerate() {
            // For every element except the first, add a comma.
            // Use the ? operator to return on errors.
            if count != 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", v)?;
        }

        // Close the opened bracket and return a fmt::Result value.
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor::Armor;

    fn kevlar_shirt() -> Armor {
        Armor::new(
            "Kevlar Shirt".to_string(),
            1,
            500,
            200,
            "Soft body armor".to_string(),
            10,
            vec![
                HitZone::Chest,
                HitZone::Shoulders,
                HitZone::Vitals,
                HitZone::Stomach,
            ],
            false,
            1,
        )
    }

    fn kevlar_tights() -> Armor {
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

    fn braces() -> Armor {
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

    fn long_leather_cloak() -> Armor {
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

    fn leather_boots() -> Armor {
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

    fn helmet() -> Armor {
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
    fn test_wear_armor() {
        let mut character = Character::new(
            "Name".to_string(),
            "Role".to_string(),
            25,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
        );

        let inner_armor = leather_boots();
        let outer_armor = long_leather_cloak();

        let outer_armor_uuid = outer_armor.item.uuid;
        let inner_armor_uuid = inner_armor.item.uuid;

        character.inventory.push(Box::new(inner_armor));
        character.inventory.push(Box::new(outer_armor));

        character.wear_armor(outer_armor_uuid, None);
        character.wear_armor(inner_armor_uuid, Some(outer_armor_uuid));
        assert_eq!(
            character.worn_armor,
            vec![inner_armor_uuid, outer_armor_uuid]
        );
    }
}
