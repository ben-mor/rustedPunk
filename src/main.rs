use rusted_punk::{Attribute, Attributes, Character, Inventory, Item, List, Skill};

fn main() {
    character_test();
    skill_test();
    list_test();
}

fn character_test() {
    let mut cool_guy = Character {
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
    cool_guy.inventory.push(Item::new(
        "Broomstick".to_string(),
        1,
        1500,
        0,
        "Alright you primitive Screwheads, listen up, this is my BROOMSTICK".to_string(),
    ));
    cool_guy.print();

    cool_guy.print_skills();
}

fn skill_test() {
    let skill = Skill::new("Schleichen".to_string(), Attribute::Reflexes, 2, 3);
    skill.print()
}

// keine ahnung warum aber es geht
fn list_test() {
    let v = List(vec![
        Skill::new("schleichen".to_string(), Attribute::Reflexes, 2, 3),
        Skill::new("schiesen".to_string(), Attribute::Reflexes, 4, 3),
        Skill::new("werfen".to_string(), Attribute::Reflexes, 2, 3),
        Skill::new("Erste Hilfe".to_string(), Attribute::Tech, 2, 3),
    ]);
    println!("{}", v);
}
