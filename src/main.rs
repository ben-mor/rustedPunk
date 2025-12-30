use rusted_punk::{Armor, Attribute, Character, DamageType, HitZone, Inventory, Item, List, Skill};

fn main() {
    armor_test();
    character_test();
    skill_test();
    list_test();
}

fn armor_test() {
    let mut leather = Armor::new(
        "Leather Armor".to_string(),
        2,
        1000,
        100,
        "A simple leather armor".to_string(),
        5,
        vec![HitZone::Chest],
        false,
    );
    leather.hit(1, HitZone::Chest, DamageType::ArmorPiercing);
    leather.print();
    leather.hit(6, HitZone::Chest, DamageType::Blunt);
    leather.print();
    leather.hit(3, HitZone::Chest, DamageType::ArmorPiercing);
    leather.print();

    let mut flak_vest = Armor::new(
        "Flak Vest".to_string(),
        2,
        1000,
        100,
        "A simple leather armor".to_string(),
        20,
        vec![
            HitZone::Chest,
            HitZone::LeftArm,
            HitZone::RightArm,
            HitZone::Vitals,
            HitZone::Stomach,
        ],
        true,
    );
    println!(
        "ArmorPiercing {:?}",
        flak_vest.hit(10, HitZone::Chest, DamageType::ArmorPiercing)
    );
    flak_vest.print();
    flak_vest.hit(10, HitZone::Chest, DamageType::Blunt);
    flak_vest.print();
    flak_vest.hit(30, HitZone::Chest, DamageType::Blunt);
    flak_vest.print();
    flak_vest.hit(10, HitZone::Chest, DamageType::ArmorPiercing);
    flak_vest.print();
}

fn character_test() {
    let mut cool_guy = Character {
        name: "Erwin Müller".to_string(),
        role: "Corporate".to_string(),
        age: 23,
        att: Attribute::new(3, 3),
        mov: Attribute::new(4, 4),
        coo: Attribute::new(1, 3),
        emp: Attribute::new(3, 3),
        luck: Attribute::new(3, 3),
        int: Attribute::new(10, 10),
        body: Attribute::new(7, 7),
        refl: Attribute::new(6, 6),
        tec: Attribute::new(10, 10),
        inventory: Inventory::new(),
    };
    cool_guy.inventory.push(Box::new(Item::new(
        "Broomstick".to_string(),
        1,
        1500,
        0,
        "Alright you primitive Screwheads, listen up, this is my BROOMSTICK".to_string(),
    )));
    cool_guy.inventory.push(Box::new(Armor::new(
        "Leather Armor".to_string(),
        2,
        1000,
        100,
        "A simple leather armor".to_string(),
        5,
        vec![HitZone::Chest],
        false,
    )));
    cool_guy.print();
    cool_guy.hit(15, HitZone::Chest, DamageType::ArmorPiercing);
    cool_guy.print();
}

fn skill_test() {
    let skill = Skill::new("Schleichen".to_string(), 7, 2, 3);
    skill.print()
}

// keine ahnung warum aber es geht
fn list_test() {
    let v = List(vec![
        Skill::new("schleichen".to_string(), 4, 2, 3),
        Skill::new("schiesen".to_string(), 7, 4, 3),
        Skill::new("werfen".to_string(), 6, 2, 3),
    ]);
    println!("{}", v);
}
