mod advantages;
mod armor;
mod character;
mod chargen;
mod dice;
mod health;
mod inventory;
mod melee;
mod weapons;

pub use self::advantages::{
    advantage_catalog, validate_budget, Advantage, AdvantageKind, Modifier, ModifierTarget,
    TAG_BRUISE_SCALE, TAG_HEALING_RATE, TAG_INITIATIVE,
};
pub use self::armor::{Armor, HitZone};
pub use self::character::{Attribute, AttributeValue, Character, HitOutcome, List, Skill};
pub use self::chargen::{
    age_points, generate_nsc, roll_background, roll_life_events, skill_cost, skill_point_pool,
    starting_money, validate_attributes, validate_character, validate_skill_budget,
    validate_skill_caps, LifepathEvent, LifepathVariant, ATTRIBUTE_POINTS,
};
pub use self::dice::{open_roll, skill_check};
pub use self::dice::{
    CheckResult, DiceExpr, DieRoller, Difficulty, OpenRollResult, Outcome, RandomRoller,
    SequenceRoller,
};
pub use self::health::WoundState;
pub use self::inventory::{Inventory, Item};
pub use self::melee::{
    dam_for_body, MartialArtsAction, MartialArtsStyle, MeleeClass, MELEE_GENERAL_CAP,
    SKILL_MELEE_GENERAL,
};
pub use self::weapons::{
    autofire_attack_modifier, autofire_hits, burst_hits, shot_noise_bonus, Availability,
    Concealability, DamageType, Reliability, Weapon, WeaponCategory, BURST_ATTACK_BONUS,
};
