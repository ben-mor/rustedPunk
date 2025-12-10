mod armor;
mod inventory;
pub use self::armor::Armor;
pub use self::inventory::Inventory;
pub use self::inventory::Item;
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
    pub fn print(self) {
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
