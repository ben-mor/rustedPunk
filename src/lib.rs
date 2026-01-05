mod inventory;
pub use self::inventory::Inventory;
pub use self::inventory::Item;

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

// Name: Erwin Müller
// Role: Corporate Age: 23
// Att:3/3
// mov: 4/4
// Coo: 3/3
// Emp: 3/3
// luck: 3/3
// Int: 10/10
// body: 7/7 Ref: 6/6 Tch: 10/10
// Ressourc:  5 = 15   | Schaden: OOOO.0000.0000.0000.0000.0000.0000.0000.0000
// HumPerce:  2 =  5  | KO       220X1X4362X23032X334X0105XX2X610XX7XXXX8XXXX9
// 0GFight :  1 =  7      | DEATH                   X336110X6210XX3XX0X4332X5XXX06
// Composit:  1 = 11

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    pub age: usize,
    pub attributes: Attributes,
    pub inventory: Inventory,
    pub skills: Vec<Skill>,
}

#[serde_as]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes(
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")] BTreeMap<Attribute, AttributeValue>,
);

impl Attributes {
    pub fn new() -> Self {
        let mut m = BTreeMap::new();

        m.insert(Attribute::Attractiveness, AttributeValue::new(0, 0));
        m.insert(Attribute::Move, AttributeValue::new(0, 0));
        m.insert(Attribute::Coolness, AttributeValue::new(0, 0));
        m.insert(Attribute::Empathy, AttributeValue::new(0, 0));
        m.insert(Attribute::Luck, AttributeValue::new(0, 0));
        m.insert(Attribute::Intelligence, AttributeValue::new(0, 0));
        m.insert(Attribute::Body, AttributeValue::new(0, 0));
        m.insert(Attribute::Reflexes, AttributeValue::new(0, 0));
        m.insert(Attribute::Tech, AttributeValue::new(0, 0));

        Attributes(m)
    }

    pub fn get(&self, attr: &Attribute) -> Option<&AttributeValue> {
        self.0.get(attr)
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Attributes {{")?;
        for (key, value) in &self.0 {
            writeln!(f, "\t{:?}: {}", key, value)?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn print(&self) {
        println!(
            "\
Character {{ \n\
\tname: {}\n\
\trole: {}\n\
\tage: {}\n\
\t{}\n
\tInventory: {}\n\
}}",
            self.name, self.role, self.age, self.attributes, self.inventory,
        );
    }

    pub fn print_skills(&self) {
        println!("Skills:");
        for skill in &self.skills {
            let attr = self.attributes.get(&skill.base).unwrap();
            let total = skill.level + attr.actual;
            println!("\t {}: {}", skill.name, total);
        }
    }

    /// Write the character to the provided `io::Write`er.
    pub fn write(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        let serialized = toml::to_string_pretty(self).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("failed to serialize character: {}", err),
            )
        })?;

        w.write_all(serialized.as_bytes())
    }

    /// Reads a character from the provided `io::Read`er.
    pub fn from_reader(r: &mut impl std::io::Read) -> std::io::Result<Self> {
        let mut serialized = String::new();
        r.read_to_string(&mut serialized)?;
        let character = toml::from_str(&serialized).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("failed to deserialize character: {}", err),
            )
        })?;

        Ok(character)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub base: Attribute,
    pub level: usize,
    pub level_up_modifierer: usize,
}

impl Skill {
    pub fn print(self) {
        println!(
            "Skillname {} {{
            \tbase: {:?} 
            \tlevel: {} 
            \tlevel up modifeier: {}
        }}",
            self.name, self.base, self.level, self.level_up_modifierer
        )
    }

    pub fn new(name: String, base: Attribute, level: usize, level_up_modifierer: usize) -> Self {
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
        write!(
            f,
            "name: {} base: {:?} level: {}",
            self.name, self.base, self.level
        )
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

    use std::io::{Seek, SeekFrom};

    #[test]
    fn test_char_ser_de() {
        let mut erwin = Character {
            name: "Erwin Müller".to_string(),
            role: "Corporate".to_string(),
            age: 23,
            attributes: Attributes::new(),
            inventory: Inventory::new(),
            skills: vec![Skill::new(
                "schleichen".to_string(),
                Attribute::Reflexes,
                2,
                3,
            )],
        };
        erwin.inventory.push(Item::new(
            "Broomstick".to_string(),
            1,
            1500,
            0,
            "Alright you primitive Screwheads, listen up, this is my BROOMSTICK".to_string(),
        ));

        let serialized_erwin = toml::to_string_pretty(&erwin).unwrap();
        assert_eq!(
            serialized_erwin,
            r#"name = 'Erwin Müller'
role = 'Corporate'
age = 23
[attributes.Attractiveness]
base = 0
actual = 0

[attributes.Move]
base = 0
actual = 0

[attributes.Coolness]
base = 0
actual = 0

[attributes.Empathy]
base = 0
actual = 0

[attributes.Luck]
base = 0
actual = 0

[attributes.Intelligence]
base = 0
actual = 0

[attributes.Body]
base = 0
actual = 0

[attributes.Reflexes]
base = 0
actual = 0

[attributes.Tech]
base = 0
actual = 0
[[inventory.items]]
name = 'Broomstick'
amount = 1
weight_grams = 1500
price_eb = 0
comment = 'Alright you primitive Screwheads, listen up, this is my BROOMSTICK'

[[skills]]
name = 'schleichen'
base = 'Reflexes'
level = 2
level_up_modifierer = 3
"#
        );
        let deserialized_erwin: Character = toml::from_str(&serialized_erwin).unwrap();
        assert_eq!(deserialized_erwin, erwin);
    }

    #[test]
    fn test_char_io() {
        let mut erwin = Character {
            name: "Erwin Müller".to_string(),
            role: "Corporate".to_string(),
            age: 23,
            attributes: Attributes::new(),
            inventory: Inventory::new(),
            skills: vec![Skill::new(
                "schleichen".to_string(),
                Attribute::Reflexes,
                2,
                3,
            )],
        };
        erwin.inventory.push(Item::new(
            "Broomstick".to_string(),
            1,
            1500,
            0,
            "Alright you primitive Screwheads, listen up, this is my BROOMSTICK".to_string(),
        ));

        // create a temporary file, which will be deleted at the end of this test, when it goes out of scope
        let mut file = tempfile::tempfile().unwrap();

        // write the erwin char to the file
        erwin.write(&mut file).unwrap();

        // reset the file, by seeking to pos 0
        file.seek(SeekFrom::Start(0)).unwrap();

        // load the character from the file
        let new_erwin = Character::from_reader(&mut file).unwrap();

        // check that the original character and the loaded are the same
        assert_eq!(erwin, new_erwin);
    }
}
