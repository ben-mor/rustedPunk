use rusted_punk::{
    generate_nsc, Armor, Attribute, Character, DamageType, Difficulty, HitZone, Item,
    LifepathVariant, List, RandomRoller, Skill,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "chargen" {
        chargen_command(&args[2..]);
        return;
    }
    armor_test();
    character_test();
    skill_test();
    list_test();
    dice_test();
}

/// `cargo run -- chargen [--classic|--desaster] [name] [role] [age]`
/// Rolls an NSC skeleton: valid random attributes plus lifepath (Q29).
fn chargen_command(args: &[String]) {
    let variant = if args.iter().any(|arg| arg == "--desaster") {
        LifepathVariant::Desaster
    } else {
        LifepathVariant::Classic
    };
    let positional: Vec<&String> = args.iter().filter(|arg| !arg.starts_with("--")).collect();
    let name = positional.first().map_or("NSC", |s| s.as_str()).to_string();
    let role = positional
        .get(1)
        .map_or("Ganger", |s| s.as_str())
        .to_string();
    let age = positional.get(2).and_then(|s| s.parse().ok()).unwrap_or(22);

    let mut roller = RandomRoller;
    let (character, events) = generate_nsc(name, role, age, variant, &mut roller);
    character.print();
    println!("Lifepath ({:?}):", variant);
    for event in events {
        if event.age > 0 {
            println!("  [{}] {}: {}", event.age, event.table, event.text);
        } else {
            println!("  {}: {}", event.table, event.text);
        }
    }
}

fn dice_test() {
    let mut character = Character::new(
        "Testschütze".to_string(),
        "Solo".to_string(),
        25,
        5,
        6,
        5,
        5,
        5,
        5,
        7,
        8,
        5,
    );
    character
        .skills
        .push(Skill::new("Pistole".to_string(), Attribute::Reflexes, 4, 1));

    let mut roller = RandomRoller;
    for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
        let result = character
            .check_skill("Pistole", 0, difficulty, &mut roller)
            .unwrap();
        println!(
            "Pistole vs {:?} ({}): {:?} — total {} (dice {:?})",
            difficulty, result.target, result.outcome, result.total, result.die_rolls
        );
    }
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
    let mut roller = RandomRoller;
    cool_guy.hit(
        15,
        HitZone::Chest,
        DamageType::ArmorPiercing,
        true,
        &mut roller,
    );
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
