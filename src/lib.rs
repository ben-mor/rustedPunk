mod inventory;
pub use self::inventory::Inventory;
pub use self::inventory::Item;

use std::collections::HashMap;
use std::fmt;

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

pub struct Character {
    pub name: String,
    pub role: String,
    pub age: usize,
    pub attributes: Attributes,
    pub inventory: Inventory,
    pub skills: Vec<Skill>,
}

pub struct Attributes(HashMap<Attribute, AttributeValue>);

impl Attributes {
    pub fn new() -> Self {
        let mut m = HashMap::new();

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

#[derive(Debug, PartialEq, Eq, Hash)]
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
}

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
