use crate::advantages::{validate_budget, AdvantageKind};
use crate::character::{Attribute, Character, Skill};
use crate::dice::DieRoller;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Which lifepath table set to use (Q29). The Desaster variant starts as a
/// copy of the classic tables and is meant to be edited over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifepathVariant {
    Classic,
    Desaster,
}

/// Attribute rules at creation: 60 points over the 9 attributes,
/// each between 2 and 9 (10 is not allowed at creation), INT and REF ≥ 5.
pub const ATTRIBUTE_POINTS: i32 = 60;

pub fn validate_attributes(character: &Character) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut total = 0;
    for (attribute, value) in character.attributes.iter() {
        total += value.base;
        if !(2..=9).contains(&value.base) {
            errors.push(format!(
                "{}: base value {} outside 2..=9 (10 is not allowed at creation)",
                attribute, value.base
            ));
        }
    }
    for required in [Attribute::Intelligence, Attribute::Reflexes] {
        if character.attributes.get(&required).unwrap().base < 5 {
            errors.push(format!("{} must be at least 5", required));
        }
    }
    if total != ATTRIBUTE_POINTS {
        errors.push(format!(
            "attribute points must total {}, got {}",
            ATTRIBUTE_POINTS, total
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Extra skill points from age (docs/rules/Alterspunkte.wiki).
/// Beyond the table's end (32) the +3 pattern continues; note that
/// characters older than 28 hit the still-unwritten aging rules (#22).
pub fn age_points(age: i32) -> i32 {
    match age {
        ..=15 => 0,
        16..=24 => age - 15,
        25 => 11,
        26 => 13,
        27 => 15,
        28 => 17,
        29 => 20,
        30 => 23,
        31 => 26,
        32 => 29,
        _ => 29 + 3 * (age - 32),
    }
}

/// The skill point pool at creation: INT + REF + 40 + age points.
pub fn skill_point_pool(character: &Character) -> i32 {
    character
        .attributes
        .get(&Attribute::Intelligence)
        .unwrap()
        .base
        + character.attributes.get(&Attribute::Reflexes).unwrap().base
        + 40
        + age_points(character.age)
}

/// Cost of a skill at creation: every level costs its number
/// (level 3 = 1 + 2 + 3 = 6 points).
pub fn skill_cost(level: i32) -> i32 {
    level * (level + 1) / 2
}

/// Validates the high-skill caps: one skill at 8, one at 7, two at 6 —
/// tradeable 1:2 downward (no 8 allows three 7s, etc.).
pub fn validate_skill_caps(skills: &[Skill]) -> Result<(), String> {
    let count_at = |level: i32| skills.iter().filter(|skill| skill.level == level).count() as i32;
    if let Some(skill) = skills.iter().find(|skill| skill.level > 8) {
        return Err(format!(
            "'{}' has level {}: above 8 is not allowed at creation",
            skill.name, skill.level
        ));
    }
    let mut slots_8 = 1;
    let used_8 = count_at(8);
    if used_8 > slots_8 {
        return Err(format!("only one skill at 8 allowed, found {}", used_8));
    }
    slots_8 -= used_8;
    let mut slots_7 = 1 + 2 * slots_8;
    let used_7 = count_at(7);
    if used_7 > slots_7 {
        return Err(format!(
            "too many skills at 7: {} (allowed {})",
            used_7, slots_7
        ));
    }
    slots_7 -= used_7;
    let slots_6 = 2 + 2 * slots_7;
    let used_6 = count_at(6);
    if used_6 > slots_6 {
        return Err(format!(
            "too many skills at 6: {} (allowed {})",
            used_6, slots_6
        ));
    }
    Ok(())
}

/// Validates the full skill/advantage budget: skill costs plus advantage CP
/// (1 CP = 1 skill point; disadvantages grant points) against the pool.
pub fn validate_skill_budget(character: &Character) -> Result<i32, String> {
    validate_budget(&character.advantages)?;
    let pool = skill_point_pool(character);
    let skill_costs: i32 = character
        .skills
        .iter()
        .map(|skill| skill_cost(skill.level))
        .sum();
    let advantage_costs: i32 = character
        .advantages
        .iter()
        .map(|advantage| match advantage.kind {
            AdvantageKind::Advantage => advantage.cp,
            AdvantageKind::Disadvantage => -advantage.cp,
        })
        .sum();
    let spent = skill_costs + advantage_costs;
    if spent > pool {
        Err(format!(
            "skill budget exceeded: {} spent (skills {} + advantages {}), pool is {}",
            spent, skill_costs, advantage_costs, pool
        ))
    } else {
        Ok(pool - spent)
    }
}

/// Starting money: the levels of three profession-defining skills
/// (player-chosen, GM veto) × 350 eb; the Reich advantage doubles the
/// factor per level (tag `reich`).
pub fn starting_money(character: &Character, profession_skills: &[&str; 3]) -> Result<i32, String> {
    let mut sum = 0;
    for name in profession_skills {
        let skill = character
            .skills
            .iter()
            .find(|skill| &skill.name == name)
            .ok_or_else(|| format!("profession skill '{}' not found on the character", name))?;
        sum += skill.level;
    }
    let factor = 350 << character.modifier_for_tag("reich").max(0);
    Ok(sum * factor)
}

/// Runs all creation validations at once and returns the collected errors.
pub fn validate_character(character: &Character) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if let Err(mut attribute_errors) = validate_attributes(character) {
        errors.append(&mut attribute_errors);
    }
    if let Err(error) = validate_skill_caps(&character.skills) {
        errors.push(error);
    }
    if let Err(error) = validate_skill_budget(character) {
        errors.push(error);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// --------------------------------------------------------------------------
// Lifepath
// --------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct LifepathData {
    /// Tables rolled once for the background, in this order.
    background: Vec<String>,
    /// Entry table for each year above 15.
    yearly: String,
    tables: BTreeMap<String, LifepathTable>,
}

#[derive(Debug, Deserialize)]
struct LifepathTable {
    title: String,
    entries: Vec<LifepathTableEntry>,
}

#[derive(Debug, Deserialize)]
struct LifepathTableEntry {
    min: i32,
    max: i32,
    text: String,
    /// Follow-up table to roll on afterwards.
    goto: Option<String>,
}

/// One resolved lifepath line: which table said what.
#[derive(Debug, PartialEq, Eq)]
pub struct LifepathEvent {
    /// Age the event happened at; 0 for background rolls.
    pub age: i32,
    pub table: String,
    pub text: String,
}

const LIFEPATH_CLASSIC: &str = include_str!("../data/lifepath_classic.toml");
const LIFEPATH_DESASTER: &str = include_str!("../data/lifepath_desaster.toml");

fn lifepath_data(variant: LifepathVariant) -> LifepathData {
    let raw = match variant {
        LifepathVariant::Classic => LIFEPATH_CLASSIC,
        LifepathVariant::Desaster => LIFEPATH_DESASTER,
    };
    toml::from_str(raw).expect("embedded lifepath data must parse")
}

fn resolve_table(
    data: &LifepathData,
    table_name: &str,
    age: i32,
    roller: &mut dyn DieRoller,
    events: &mut Vec<LifepathEvent>,
    depth: i32,
) {
    assert!(
        depth < 10,
        "lifepath table loop detected at '{}'",
        table_name
    );
    let table = data
        .tables
        .get(table_name)
        .unwrap_or_else(|| panic!("lifepath table '{}' missing", table_name));
    let roll = roller.d10();
    let entry = table
        .entries
        .iter()
        .find(|entry| (entry.min..=entry.max).contains(&roll))
        .unwrap_or_else(|| panic!("table '{}' has no entry for roll {}", table_name, roll));
    events.push(LifepathEvent {
        age,
        table: table.title.clone(),
        text: entry.text.clone(),
    });
    if let Some(next) = &entry.goto {
        resolve_table(data, next, age, roller, events, depth + 1);
    }
}

/// Rolls the one-time background tables (family, childhood, personality …).
pub fn roll_background(variant: LifepathVariant, roller: &mut dyn DieRoller) -> Vec<LifepathEvent> {
    let data = lifepath_data(variant);
    let mut events = Vec::new();
    for table_name in &data.background {
        resolve_table(&data, table_name, 0, roller, &mut events, 0);
    }
    events
}

/// Rolls the yearly life events for every year above 15 up to `age`.
pub fn roll_life_events(
    variant: LifepathVariant,
    age: i32,
    roller: &mut dyn DieRoller,
) -> Vec<LifepathEvent> {
    let data = lifepath_data(variant);
    let mut events = Vec::new();
    for year in 16..=age {
        resolve_table(&data, &data.yearly, year, roller, &mut events, 0);
    }
    events
}

// --------------------------------------------------------------------------
// NSC generation
// --------------------------------------------------------------------------

/// Generates a rules-valid NSC skeleton: random attributes (60 points,
/// INT/REF ≥ 5, all 2..=9), rolled background and life events.
/// Skills, advantages and gear stay empty — that's the GM's flavor work.
pub fn generate_nsc(
    name: String,
    role: String,
    age: i32,
    variant: LifepathVariant,
    roller: &mut dyn DieRoller,
) -> (Character, Vec<LifepathEvent>) {
    // start at the minimums, then spread the remaining points randomly
    let mut values: BTreeMap<Attribute, i32> = [
        (Attribute::Attractiveness, 2),
        (Attribute::Move, 2),
        (Attribute::Coolness, 2),
        (Attribute::Empathy, 2),
        (Attribute::Luck, 2),
        (Attribute::Intelligence, 5),
        (Attribute::Body, 2),
        (Attribute::Reflexes, 5),
        (Attribute::Tech, 2),
    ]
    .into();
    let attributes: Vec<Attribute> = values.keys().copied().collect();
    let mut remaining = ATTRIBUTE_POINTS - values.values().sum::<i32>();
    while remaining > 0 {
        let pick = attributes[(roller.d10() - 1) as usize % attributes.len()];
        let value = values.get_mut(&pick).unwrap();
        if *value < 9 {
            *value += 1;
            remaining -= 1;
        }
    }

    let character = Character::new(
        name,
        role,
        age,
        values[&Attribute::Attractiveness],
        values[&Attribute::Move],
        values[&Attribute::Coolness],
        values[&Attribute::Empathy],
        values[&Attribute::Luck],
        values[&Attribute::Intelligence],
        values[&Attribute::Body],
        values[&Attribute::Reflexes],
        values[&Attribute::Tech],
    );

    let mut events = roll_background(variant, roller);
    events.append(&mut roll_life_events(variant, age, roller));
    (character, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advantages::{Advantage, ModifierTarget};
    use crate::dice::{RandomRoller, SequenceRoller};

    fn valid_character() -> Character {
        // 60 points: 5+5+9+7+7+7+7+7+6 = 60
        Character::new(
            "Testfall".to_string(),
            "Tech".to_string(),
            25,
            7, // att
            7, // mov
            7, // coo
            7, // emp
            6, // luck
            5, // int
            9, // body
            5, // refl
            7, // tec
        )
    }

    #[test]
    fn test_validate_attributes_accepts_valid_spread() {
        assert_eq!(validate_attributes(&valid_character()), Ok(()));
    }

    #[test]
    fn test_validate_attributes_rejects_violations() {
        // INT 4 (< 5) and a 10: two errors plus whatever the sum says
        let character = Character::new(
            "K".to_string(),
            "R".to_string(),
            20,
            7,
            7,
            7,
            7,
            6,
            4,
            10,
            5,
            7,
        );
        let errors = validate_attributes(&character).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("Intelligence")));
        assert!(errors.iter().any(|error| error.contains("outside 2..=9")));
    }

    #[test]
    fn test_age_points_table() {
        assert_eq!(age_points(15), 0);
        assert_eq!(age_points(16), 1);
        assert_eq!(age_points(24), 9);
        assert_eq!(age_points(25), 11);
        assert_eq!(age_points(32), 29);
        assert_eq!(age_points(34), 35);
    }

    #[test]
    fn test_skill_point_pool() {
        // INT 5 + REF 5 + 40 + age 25 -> 11 = 61
        assert_eq!(skill_point_pool(&valid_character()), 61);
    }

    #[test]
    fn test_skill_cost_is_cumulative() {
        assert_eq!(skill_cost(1), 1);
        assert_eq!(skill_cost(3), 6);
        assert_eq!(skill_cost(8), 36);
    }

    fn skill(name: &str, level: i32) -> Skill {
        Skill::new(name.to_string(), Attribute::Reflexes, level, 1)
    }

    #[test]
    fn test_skill_caps_default_slots() {
        let skills = vec![skill("a", 8), skill("b", 7), skill("c", 6), skill("d", 6)];
        assert!(validate_skill_caps(&skills).is_ok());
        let too_many = vec![skill("a", 8), skill("b", 8)];
        assert!(validate_skill_caps(&too_many).is_err());
    }

    #[test]
    fn test_skill_caps_trade_down() {
        // wiki example: three 7s and two 6s, but no 8
        let skills = vec![
            skill("a", 7),
            skill("b", 7),
            skill("c", 7),
            skill("d", 6),
            skill("e", 6),
        ];
        assert!(validate_skill_caps(&skills).is_ok());
        // four 7s is one too many even after the trade
        let skills = vec![skill("a", 7), skill("b", 7), skill("c", 7), skill("d", 7)];
        assert!(validate_skill_caps(&skills).is_err());
    }

    #[test]
    fn test_skill_budget_with_advantages() {
        let mut character = valid_character(); // pool 61
        character.skills.push(skill("Pistole", 8)); // 36
        character.skills.push(skill("Fahren", 5)); // 15
        character.advantages.push(Advantage::new(
            "Lesen und Schreiben".to_string(),
            AdvantageKind::Advantage,
            10,
            String::new(),
        ));
        // 36 + 15 + 10 = 61: exactly the pool
        assert_eq!(validate_skill_budget(&character), Ok(0));

        // a disadvantage grants points back
        character.advantages.push(Advantage::new(
            "Ehrlich".to_string(),
            AdvantageKind::Disadvantage,
            5,
            String::new(),
        ));
        assert_eq!(validate_skill_budget(&character), Ok(5));
    }

    #[test]
    fn test_starting_money() {
        let mut character = valid_character();
        character.skills.push(skill("Basic Tech", 6));
        character.skills.push(skill("Elektrotechnik", 4));
        character.skills.push(skill("Schweißen", 2));
        let money =
            starting_money(&character, &["Basic Tech", "Elektrotechnik", "Schweißen"]).unwrap();
        assert_eq!(money, 12 * 350);

        // Reich level 2 quadruples the factor
        character.advantages.push(
            Advantage::new(
                "Reich".to_string(),
                AdvantageKind::Advantage,
                4,
                String::new(),
            )
            .with_level(2)
            .with_modifier(ModifierTarget::Tag("reich".to_string()), 2),
        );
        let money =
            starting_money(&character, &["Basic Tech", "Elektrotechnik", "Schweißen"]).unwrap();
        assert_eq!(money, 12 * 1400);

        assert!(starting_money(&character, &["Basic Tech", "Nope", "Schweißen"]).is_err());
    }

    #[test]
    fn test_lifepath_data_parses_and_resolves() {
        for variant in [LifepathVariant::Classic, LifepathVariant::Desaster] {
            let mut roller = RandomRoller;
            let background = roll_background(variant, &mut roller);
            assert!(!background.is_empty());
            // three years -> at least three events (subtables add more)
            let events = roll_life_events(variant, 18, &mut roller);
            assert!(events.len() >= 3);
            assert_eq!(
                events.iter().filter(|event| event.age == 16).count() >= 1,
                true
            );
        }
    }

    #[test]
    fn test_lifepath_is_deterministic_with_scripted_dice() {
        // year 16: master roll 9 -> "Nothing happened" (no subtable)
        let mut roller = SequenceRoller::new(vec![9]);
        let events = roll_life_events(LifepathVariant::Classic, 16, &mut roller);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].age, 16);
    }

    #[test]
    fn test_generate_nsc_is_rules_valid() {
        let mut roller = RandomRoller;
        for _ in 0..20 {
            let (character, events) = generate_nsc(
                "NSC".to_string(),
                "Ganger".to_string(),
                20,
                LifepathVariant::Desaster,
                &mut roller,
            );
            assert_eq!(validate_attributes(&character), Ok(()));
            assert!(!events.is_empty());
        }
    }
}
