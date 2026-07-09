use rusted_punk::{Armor, Attribute, Character, DamageType, HitZone, Item, List, Skill};

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
        4,
        vec![HitZone::Chest],
        false,
        0,
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
        1,
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
    let mut cool_guy = Character::new(
        "Erwin Müller".to_string(),
        "Corporate".to_string(),
        23,
        3,
        4,
        3,
        3,
        3,
        10,
        7,
        6,
        10,
    );
    cool_guy.inventory.push(Box::new(Item::new(
        None,
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
        4,
        vec![HitZone::Chest],
        false,
        0,
    )));
    cool_guy.print();
    cool_guy.hit(15, HitZone::Chest, DamageType::ArmorPiercing);
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
