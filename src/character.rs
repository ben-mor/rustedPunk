use std::ops::{Deref, DerefMut};
use crate::{armor::HitZone, Armor};
use crate::{inventory::Inventory, DamageType};
use std::collections::BTreeMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_with::{serde_as, DisplayFromStr};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    pub age: usize,
    pub current_damage: usize,
    pub damage_notes: String,
    pub worn_armor: Vec<Uuid>,
    pub skills: Vec<Skill>,
    pub attributes: Attributes,
    pub inventory: Inventory,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Attribute {
    Attractiveness,
    Move,
    Coolness,
    Empathy,
    Luck,
    Intelligence,
    Body,
    Reflexes,
    Tech,
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Attribute {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "attractiveness" => Ok(Self::Attractiveness),
            "move" => Ok(Self::Move),
            "coolness" => Ok(Self::Coolness),
            "empathy" => Ok(Self::Empathy),
            "luck" => Ok(Self::Luck),
            "intelligence" => Ok(Self::Intelligence),
            "body" => Ok(Self::Body),
            "reflexes" => Ok(Self::Reflexes),
            "tech" => Ok(Self::Tech),
            _ => Err(format!("Unknown attribute: {}", s)),
        }
    }
}

#[serde_as]
#[derive (Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Attributes(
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    BTreeMap<Attribute, AttributeValue>
);

impl Deref for Attributes {
    type Target = BTreeMap<Attribute, AttributeValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (attr, value) in &self.0 {
            writeln!(f, "\t{}: {}", attr, value)?;
        }
        Ok(())
    }
}

/// Represents a character attribute with base and current values.
///
/// - `base`: The attribute value at character creation (natural/starting value)
/// - `actual`: The "current" value including semi-permanent modifications
///   (cyberware, training, long-term injuries). This is what appears on the
///   character sheet and persists between sessions.
///
/// For momentary effects (drugs, encumbrance, combat boosts), use
/// `Character::effective_attribute()` which calculates the actual roll value.
#[derive (Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeValue {
    pub base: usize,
    pub actual: usize,
}

impl AttributeValue {
    pub fn new(actual: usize, base: usize) -> Self {
        AttributeValue { base, actual }
    }
}

impl fmt::Display for AttributeValue {
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
        let mut character = Character {
            name,
            role,
            age,
            attributes: Attributes(BTreeMap::new()),
            inventory: Inventory::new(),
            worn_armor: Vec::new(),
            current_damage: 0,
            damage_notes: "".to_string(),
            skills: Vec::new(),
        };

        character.attributes.insert(Attribute::Attractiveness, AttributeValue { base: att, actual: att });
        character.attributes.insert(Attribute::Move, AttributeValue { base: mov, actual: mov });
        character.attributes.insert(Attribute::Coolness, AttributeValue { base: coo, actual: coo });
        character.attributes.insert(Attribute::Empathy, AttributeValue { base: emp, actual: emp });
        character.attributes.insert(Attribute::Luck, AttributeValue { base: luck, actual: luck });
        character.attributes.insert(Attribute::Intelligence, AttributeValue { base: int, actual: int });
        character.attributes.insert(Attribute::Body, AttributeValue { base: body, actual: body });
        character.attributes.insert(Attribute::Reflexes, AttributeValue { base: refl, actual: refl });
        character.attributes.insert(Attribute::Tech, AttributeValue { base: tec, actual: tec });

        character
    }

    pub fn print(&self) {
        println!(
            "\
Character {{ \n\
\tname: {}\n\
\trole: {}\n\
\tage: {}\n\
\tAttributes: {}\n\
\tInventory: {}\n\
}}",
            self.name,
            self.role,
            self.age,
            self.attributes,
            self.inventory,
        );
    }

    /// Body Type Modifier
    /// How much damage gets reduced when being hit, based on the body stat.
    pub fn btm(&self) -> usize {
        match self.attributes.get(&Attribute::Body).unwrap().actual {
            0..=2 => 0,
            3..=4 => 1,
            5..=7 => 2,
            8..=9 => 3,
            10 => 4,
            _ => 5,
        }
    }

    /// Returns the effective attribute value for dice rolls, including all
    /// temporary modifiers (encumbrance, drugs, combat effects, etc.).
    pub fn effective_attribute(&self, attr: Attribute) -> usize {
        let base_value = self.attributes[&attr].actual;

        match attr {
            Attribute::Reflexes => base_value.saturating_sub(self.encumberance()),
            Attribute::Move => base_value.saturating_sub(self.encumberance()),
            _ => base_value
        }
    }

    /// Calculates the malus to movement and reflexes based on encumberance
    pub fn encumberance(&self) -> usize {
        let inventory_weight = self.inventory.calculate_total_weight();
        let capacity = self.carry_capacity();
        match (inventory_weight*10) / capacity {
            0..=4 => 0,
            5..=6 => 1,
            7..=9 => 2,
            10..=12 => 4,
            13..=15 => 6,
            _ => 8,
        }
    }

    /// Looks up the carry capacity of the character
    /// Returns grams.
    pub fn carry_capacity(&self) -> usize {
        self.attributes.get(&Attribute::Body).unwrap().actual * 10000
    }

    /// Looks up the deadlift capacity of the character
    /// Returns grams.
    pub fn deadlift(&self) -> usize {
        self.carry_capacity() * 4
    }

    pub fn wear_armor(&mut self, armor_uuid: Uuid, underneath: Option<Uuid>) {
        if self.inventory.get_item(armor_uuid).is_none() {
            unreachable!("Armor_uuid not found in inventory");
        }
        if underneath.is_some() {
            if self.inventory.get_item(underneath.unwrap()).is_none() {
                unreachable!("underneath_uuid not found in inventory");
            }
            if let Some(index) = self
                .worn_armor
                .iter()
                .position(|&uuid| uuid == underneath.unwrap())
            {
                // Insert the new armor at that index (pushes existing armor one position higher)
                self.worn_armor.insert(index, armor_uuid);
            } else {
                panic!("tried to wear something underneath an armor that isn't worn.");
            }
        } else {
            self.worn_armor.push(armor_uuid);
        }
    }

    /// Hit the character with some damage
    ///
    /// This will apply damage to the armor (outer to inner) and then to the character.
    ///
    pub fn hit(&mut self, damage: usize, zone: HitZone, damage_type: DamageType) {
        let mut remaining_damage = damage;
        let mut absorbed_damage = 0;

        for i in (0..self.worn_armor.len()).rev() {
            let armor_uuid = self.worn_armor[i];
            let armor_item = self.inventory.get_item_mut(armor_uuid);
            let armor_opt = armor_item.expect(&format!("There was an Armor Uuid in the worn armor list ({}), but no corresponding item in the inventory.", armor_uuid))
                .as_any_mut().downcast_mut::<Armor>();
            let armor = armor_opt.expect(&format!(
                "There was an Armor in the worn_armor list ({}), that wasn't an Armor in the Inventory.",
                armor_uuid
            ));
            let damage_result = armor.hit(remaining_damage, zone, damage_type);
            remaining_damage = damage_result.remaining_damage;
            absorbed_damage += damage_result.absorbed_damage;
        }
        // House rule: 20% of absorbed damage becomes real damage through blunt trauma.
        // TODO: implement variable for this so it rotates through subsequent hits as well.
        // TODO: implement different handlings of DamageType (HollowPoint double damage)
        // TODO: implement full penetration (can't do more then 4+1d10 damage, everything else goes out on the back)
        remaining_damage += absorbed_damage / 5;
        self.take_damage(remaining_damage, zone);
    }

    /// This ignores all armor and applies damage directly.
    /// It will subtract the BTM first.
    pub fn take_damage(&mut self, damage: usize, zone: HitZone) {
        let mut remaining_damage = damage;
        if remaining_damage > 0 {
            remaining_damage = remaining_damage.saturating_sub(self.btm());
            if remaining_damage < 1 {
                remaining_damage = 1;
            }

            // TODO: Need to implement the House Rule that this doubling only takes effect after the check if it will kill you immediately.
            if zone == HitZone::Head {
                remaining_damage = remaining_damage * 2;
            }
            self.current_damage += remaining_damage;
            if remaining_damage >= 8 {
                if matches!(zone, HitZone::Head | HitZone::Chest | HitZone::Vitals) {
                    self.current_damage = 100;
                    self.damage_notes = format!("YOU ARE DEAD!\n{}", self.damage_notes);
                } else {
                    self.damage_notes = format!(
                        "HitZone {} destroyed. You are now at least Mortal 0 and about to die.\n{}",
                        zone, self.damage_notes
                    );
                    if self.current_damage <= 12 {
                        self.current_damage = 13;
                    }
                }
            }
        }
    }
    pub fn print_skills(&self) {
        println!("Skills:");
        for skill in &self.skills {
            let attr = self.attributes.get(&skill.base).unwrap();
            let total = skill.level + attr.actual;
            println!("\t {}: {}", skill.name, total);
        }
    }
}

#[derive (Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub base: Attribute,
    pub level: usize,
    pub level_up_modifier: usize,
}

impl Skill {
    pub fn print(self) {
        println!(
            "Skillname {} {{
            \tbase: {}
            \tlevel: {}
            \tlevel up modifier: {}
        }}",
            self.name,
            self.base,
            self.level,
            self.level_up_modifier
        )
    }

    pub fn new(name: String, base: Attribute, level: usize, level_up_modifierer: usize) -> Self {
        Skill {
            name,
            base,
            level,
            level_up_modifier: level_up_modifierer,
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, level: {}, base: {}", self.name, self.level, self.base)
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
    use crate::armor::tests::*;
    use toml;

    fn populated_character() -> Character {
        let mut character = Character::new(
            "test-Name".to_string(),
            "test-Role".to_string(),
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
        let kevlar_shirt = kev_shirt();
        let kevlar_shirt_uuid = kevlar_shirt.item.uuid;
        let flak_vest = flak_vest();
        let flak_vest_uuid = flak_vest.item.uuid;
        let kevlar_tights = kevlar_tights();
        let kevlar_tights_uuid = kevlar_tights.item.uuid;
        let leather_boots = leather_boots();
        let leather_boots_uuid = leather_boots.item.uuid;
        let long_leather_cloak = long_leather_cloak();
        let long_leather_cloak_uuid = long_leather_cloak.item.uuid;
        let braces = braces();
        let braces_uuid = braces.item.uuid;
        let helmet = helmet();
        let helmet_uuid = helmet.item.uuid;

        character.inventory.push(Box::new(kevlar_shirt));
        character.inventory.push(Box::new(flak_vest));
        character.inventory.push(Box::new(kevlar_tights));
        character.inventory.push(Box::new(leather_boots));
        character.inventory.push(Box::new(long_leather_cloak));
        character.inventory.push(Box::new(braces));
        character.inventory.push(Box::new(helmet));

        character.wear_armor(kevlar_shirt_uuid, None);
        character.wear_armor(flak_vest_uuid, None);
        character.wear_armor(kevlar_tights_uuid, Some(flak_vest_uuid));
        character.wear_armor(leather_boots_uuid, None);
        character.wear_armor(long_leather_cloak_uuid, None);
        character.wear_armor(braces_uuid, Some(flak_vest_uuid));
        character.wear_armor(helmet_uuid, None);

        character
    }

    #[test]
    #[rustfmt::skip]
    fn test_multi_layer_armor_penetration_blunt_arm_noncrippling() {
        let mut character = populated_character();
        let zone = HitZone::RightArm;
        let damage = 45;
        character.hit(damage, zone, DamageType::Blunt);
        let test_context = "multi-layer-arm";
        assert_armor_protection(&character,test_context,"Kevlar Shirt",       zone,  0, HitZone::Chest,   10,);
        assert_armor_protection(&character,test_context,"Flak Vest",          zone, 19, HitZone::Chest,   20,);
        assert_armor_protection(&character,test_context,"Kevlar Tights",      zone,  0, HitZone::LeftLeg, 10,);
        assert_armor_protection(&character,test_context,"Leather Boots",      zone,  0, HitZone::LeftFoot, 4,);
        assert_armor_protection(&character,test_context,"Long Leather Cloak", zone,  3, HitZone::Chest,    4,);
        assert_armor_protection(&character,test_context,"Braces",             zone,  9, HitZone::LeftArm, 10,);
        assert_armor_protection(&character,test_context,"Helmet",             zone,  0, HitZone::Head,    15,);

        let expected_armor_on_arm = 20 + 10 + 4;
        let expected_blunt_trauma = expected_armor_on_arm / 5;
        let body_type_modifier = 4;

        let expected_damage = (damage - expected_armor_on_arm) + expected_blunt_trauma - body_type_modifier;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    #[test]
    #[rustfmt::skip]
    fn test_multi_layer_armor_penetration_blunt_head_noncrippling() {
        let mut character = populated_character();
        let zone = HitZone::Head;
        let damage = 16;
        character.hit(damage, zone, DamageType::Blunt);
        let test_context = "multi-layer-head";
        assert_armor_protection(&character,test_context,"Kevlar Shirt",       zone,  0, HitZone::Chest,   10,);
        assert_armor_protection(&character,test_context,"Flak Vest",          zone,  0, HitZone::Chest,   20,);
        assert_armor_protection(&character,test_context,"Kevlar Tights",      zone,  0, HitZone::LeftLeg, 10,);
        assert_armor_protection(&character,test_context,"Leather Boots",      zone,  0, HitZone::LeftFoot, 4,);
        assert_armor_protection(&character,test_context,"Long Leather Cloak", zone,  0, HitZone::Chest,    4,);
        assert_armor_protection(&character,test_context,"Braces",             zone,  0, HitZone::LeftArm, 10,);
        assert_armor_protection(&character,test_context,"Helmet",             zone, 14, HitZone::Vitals,   0,);

        // armor on head: 15
        // blunt trauma: 3
        // body type modifier: 4 (but can't reduce lower than 0)
        // damage on head is doubled

        let expected_damage = 2;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    #[test]
    fn test_take_crippling_arm_damage() {
        let mut character = populated_character();
        let zone = HitZone::LeftArm;
        let damage = 15; // -4 BTM!
        character.take_damage(damage, zone);
        let test_context = "crippling-arm";

        // damage of 8 or more is crippling and the person immediately goes into mortal 0 state, BTM or not.

        let expected_damage = 13;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    #[test]
    fn test_take_crippling_vitals_damage() {
        let mut character = populated_character();
        let zone = HitZone::Vitals;
        let damage = 12;
        character.take_damage(damage, zone);
        let test_context = "crippling-vitals";

        // damage of 8 or more is crippling and on the vitals you just die.

        let expected_damage = 100;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    #[test]
    fn test_take_crippling_head_damage() {
        let mut character = populated_character();
        let zone = HitZone::Head;
        let damage = 8;
        character.take_damage(damage, zone);
        let test_context = "crippling-head";

        // damage of 8 or more is crippling on the head you just die, also damage is doubled.

        let expected_damage = 100;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    fn assert_armor_protection(
        character: &Character,
        test_context: &str,
        armor_name: &str,
        hit_zone: HitZone,
        expected_remaining_protection: usize,
        unhit_zone: HitZone,
        expected_original_protection_in_unhit_zone: usize,
    ) {
        let armor_list = character.inventory.get_all_armor();
        let armor = armor_list
            .iter()
            .find(|armor_piece| armor_piece.item.name.eq(&armor_name))
            .unwrap();
        assert_armor_on_hit_zone(
            test_context,
            armor_name,
            hit_zone,
            expected_remaining_protection,
            armor,
        );
        assert_armor_on_hit_zone(
            test_context,
            armor_name,
            unhit_zone,
            expected_original_protection_in_unhit_zone,
            armor,
        );
    }

    fn assert_armor_on_hit_zone(
        test_context: &str,
        armor_name: &str,
        hit_zone: HitZone,
        expected_remaining_protection: usize,
        armor: &Armor,
    ) {
        let current_protection = armor.protection_current.get(&hit_zone);
        if expected_remaining_protection == 0 {
            assert!(
                current_protection.is_none() || current_protection.unwrap() == &0,
                "{}: Expected no protection, but some was found on {} {}",
                test_context,
                armor_name,
                hit_zone
            );
        } else {
            assert!(
                current_protection.is_some() && current_protection.unwrap() == &expected_remaining_protection,
                "{}: Expected some protection, but none was found on {} {}",
                test_context,
                armor_name,
                hit_zone
            )
        }
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
        assert_eq!(character.worn_armor, vec![inner_armor_uuid, outer_armor_uuid]);
    }

    #[test]
    fn test_attribute_value_serialization() {
        let attribute_value = AttributeValue::new(5, 5);
        let serialized_attribute = toml::to_string(&attribute_value).unwrap();
        let deserialized_attribute: AttributeValue = toml::from_str(&serialized_attribute).unwrap();
        assert_eq!(attribute_value, deserialized_attribute);
    }

    #[test]
    fn test_attribute_enum_serialization() {
        let attribute = Attribute::Reflexes;
        let serialized = toml::to_string(&attribute).unwrap();
        let deserialized: Attribute = toml::from_str(&serialized).unwrap();
        assert_eq!(attribute, deserialized);
    }

    #[test]
    fn test_skill_serialization() {
        let skill = Skill::new("Schleichen".to_string(), Attribute::Reflexes, 2, 3);
        let serialized = toml::to_string(&skill).unwrap();
        let deserialized: Skill = toml::from_str(&serialized).unwrap();
        assert_eq!(skill, deserialized);
    }

    #[test]
    fn test_attributes_serialization() {
        let mut attributes = Attributes(BTreeMap::new());
        attributes.insert(Attribute::Body, AttributeValue::new(7, 7));
        attributes.insert(Attribute::Reflexes, AttributeValue::new(6, 6));
        let serialized = toml::to_string(&attributes).unwrap();
        let deserialized: Attributes = toml::from_str(&serialized).unwrap();
        assert_eq!(attributes, deserialized);
    }

    #[test]
    fn test_character_serialization() {
        let character = populated_character();
        let serialized_character_option = toml::to_string(&character);
        let serialized_character = serialized_character_option.unwrap();
        print!("serialized_character: {:?}", serialized_character);
        let deserialized_character: Character = toml::from_str(&serialized_character).unwrap();
        assert_eq!(character, deserialized_character, "character serialization round-trip didn't work {}", character);
    }

    #[test]
    fn test_character_carry_capacity() {
        let character = populated_character();
        let carry_capacity = character.carry_capacity();
        assert_eq!(carry_capacity, 100_000);
    }

    #[test]
    fn test_character_deadlift() {
        let character = populated_character();
        let deadlift = character.deadlift();
        assert_eq!(deadlift, 400_000);
    }

    #[test]
    fn test_character_encumberance() {
        let mut character = populated_character();
        assert_eq!(character.encumberance(), 0);
        assert_eq!(character.inventory.calculate_total_weight(), 5_900);
        for _i in 0..44 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 49_900);
        assert_eq!(character.encumberance(), 0);
        character.inventory.push(Box::new(kev_shirt()));
        assert_eq!(character.inventory.calculate_total_weight(), 50_900);
        assert_eq!(character.encumberance(), 1);
        assert_eq!(character.effective_attribute(Attribute::Reflexes), 9);
        assert_eq!(character.effective_attribute(Attribute::Move), 9);
        for _i in 0..20 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 70_900);
        assert_eq!(character.encumberance(), 2);
        for _i in 0..20 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 90_900);
        assert_eq!(character.encumberance(), 2);
        for _i in 0..9 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 99_900);
        assert_eq!(character.encumberance(), 2);
        character.inventory.push(Box::new(kev_shirt()));
        assert_eq!(character.inventory.calculate_total_weight(), 100_900);
        assert_eq!(character.encumberance(), 4);
        for _i in 0..30 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 130_900);
        assert_eq!(character.encumberance(), 6);
        for _i in 0..30 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 160_900);
        assert_eq!(character.encumberance(), 8);
        assert_eq!(character.effective_attribute(Attribute::Reflexes), 2);
        assert_eq!(character.effective_attribute(Attribute::Move), 2);
    }
}
