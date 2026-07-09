use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Source of raw die rolls.
///
/// The rules engine never calls the RNG directly; it always goes through this
/// trait so tests can script exact die sequences and a future server can
/// record and replay rolls.
pub trait DieRoller {
    /// Rolls a single die with `sides` faces, returning a value in 1..=sides.
    fn die(&mut self, sides: i32) -> i32;

    /// Rolls a single d10, returning a value in 1..=10.
    fn d10(&mut self) -> i32 {
        self.die(10)
    }
}

/// The real thing: uniformly random dice.
pub struct RandomRoller;

impl DieRoller for RandomRoller {
    fn die(&mut self, sides: i32) -> i32 {
        rand::random_range(1..=sides)
    }
}

/// Scripted roller for tests and replays: returns the given values in order.
///
/// # Panics
///
/// Panics when more rolls are requested than values were provided, or when a
/// value is outside the requested die's range.
pub struct SequenceRoller {
    values: Vec<i32>,
    next: usize,
}

impl SequenceRoller {
    pub fn new(values: Vec<i32>) -> Self {
        SequenceRoller { values, next: 0 }
    }
}

impl DieRoller for SequenceRoller {
    fn die(&mut self, sides: i32) -> i32 {
        let value = *self
            .values
            .get(self.next)
            .expect("SequenceRoller ran out of scripted values");
        assert!(
            (1..=sides).contains(&value),
            "SequenceRoller value out of d{} range: {}",
            sides,
            value
        );
        self.next += 1;
        value
    }
}

/// A dice expression like `5d6` or `2d6+2`, used for weapon and
/// martial-arts damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceExpr {
    pub count: i32,
    pub sides: i32,
    pub bonus: i32,
}

impl DiceExpr {
    pub fn new(count: i32, sides: i32, bonus: i32) -> Self {
        DiceExpr {
            count,
            sides,
            bonus,
        }
    }

    pub fn roll(&self, roller: &mut dyn DieRoller) -> i32 {
        (0..self.count).map(|_| roller.die(self.sides)).sum::<i32>() + self.bonus
    }

    /// The table's "average damage" as used by the shot-noise formula:
    /// `count * (sides / 2) + bonus` with integer halving
    /// (AK 5d6 → 15, Grach 2d6+2 → 8, matching the wiki examples).
    pub fn average(&self) -> i32 {
        self.count * (self.sides / 2) + self.bonus
    }
}

impl fmt::Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)?;
        if self.bonus > 0 {
            write!(f, "+{}", self.bonus)?;
        } else if self.bonus < 0 {
            write!(f, "{}", self.bonus)?;
        }
        Ok(())
    }
}

impl FromStr for DiceExpr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = || format!("Invalid dice expression: '{}'", s);
        let (count, rest) = s.split_once(['d', 'D']).ok_or_else(error)?;
        let (sides, bonus) = if let Some((sides, bonus)) = rest.split_once('+') {
            (sides, bonus.parse::<i32>().map_err(|_| error())?)
        } else if let Some((sides, malus)) = rest.split_once('-') {
            (sides, -malus.parse::<i32>().map_err(|_| error())?)
        } else {
            (rest, 0)
        };
        Ok(DiceExpr {
            count: count.parse().map_err(|_| error())?,
            sides: sides.parse().map_err(|_| error())?,
            bonus,
        })
    }
}

impl Serialize for DiceExpr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DiceExpr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Rolls a uniform number in 0..=999 from three d10s (each die read as a
/// digit, 10 = 0). Used for per-mille chances like the crippling roll.
pub fn roll_per_mille(roller: &mut dyn DieRoller) -> i32 {
    (roller.d10() % 10) * 100 + (roller.d10() % 10) * 10 + (roller.d10() % 10)
}

/// Standard difficulties from the house rules: easy 10+, normal 15+, hard 20+.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Custom(i32),
}

impl Difficulty {
    pub fn target(self) -> i32 {
        match self {
            Difficulty::Easy => 10,
            Difficulty::Normal => 15,
            Difficulty::Hard => 20,
            Difficulty::Custom(target) => target,
        }
    }
}

/// How a skill check ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Attribute + skill + committed luck already met the target: no roll,
    /// always succeeds — even in combat.
    AutoSuccess,
    Success,
    /// Failed by not reaching the target.
    Failure,
    /// Fumble (die showed 1), confirmation die 2-5: failed and looks silly.
    EmbarrassingFailure,
    /// Fumble (die showed 1), confirmation die 1: damage to yourself or your team.
    CriticalFailure,
}

impl Outcome {
    pub fn is_success(self) -> bool {
        matches!(self, Outcome::AutoSuccess | Outcome::Success)
    }
}

/// Full record of a skill check, kept for display and later replay.
#[derive(Debug, PartialEq, Eq)]
pub struct CheckResult {
    pub outcome: Outcome,
    /// attribute + skill + luck + die total (0 dice for an auto-success).
    pub total: i32,
    pub target: i32,
    /// Natural die values in roll order, explosion re-rolls included.
    pub die_rolls: Vec<i32>,
    /// The severity die after a fumble, if one happened.
    pub fumble_confirmation: Option<i32>,
}

/// Rolls a skill check against a difficulty, applying all dice house rules:
///
/// * **Auto-success**: if `attribute + skill + luck` already reaches the
///   target, no die is rolled.
/// * **Luck** modifies the first die directly (a rolled 9 plus 1 luck counts
///   as a natural 10 and explodes; a rolled 1 plus luck is no fumble).
///   Enforcing the per-evening luck budget is the caller's job.
/// * **Exploding 10**: a (luck-adjusted) 10 adds and re-rolls until no 10
///   comes up.
/// * **Fumble**: a (luck-adjusted) 1 always fails; a confirmation die decides
///   the severity: 1 critical, 2-5 embarrassing, 6-10 normal failure.
pub fn skill_check(
    attribute: i32,
    skill: i32,
    luck: i32,
    difficulty: Difficulty,
    roller: &mut dyn DieRoller,
) -> CheckResult {
    let target = difficulty.target();
    let base = attribute + skill + luck;

    if base >= target {
        return CheckResult {
            outcome: Outcome::AutoSuccess,
            total: base,
            target,
            die_rolls: Vec::new(),
            fumble_confirmation: None,
        };
    }

    let first = roller.d10();
    let mut die_rolls = vec![first];
    let adjusted_first = first + luck;

    if adjusted_first <= 1 {
        let confirmation = roller.d10();
        let outcome = match confirmation {
            1 => Outcome::CriticalFailure,
            2..=5 => Outcome::EmbarrassingFailure,
            _ => Outcome::Failure,
        };
        return CheckResult {
            outcome,
            total: attribute + skill + adjusted_first,
            target,
            die_rolls,
            fumble_confirmation: Some(confirmation),
        };
    }

    let mut die_total = adjusted_first;
    let mut last = adjusted_first;
    while last >= 10 {
        last = roller.d10();
        die_rolls.push(last);
        die_total += last;
    }

    let total = attribute + skill + die_total;
    let outcome = if total >= target {
        Outcome::Success
    } else {
        Outcome::Failure
    };
    CheckResult {
        outcome,
        total,
        target,
        die_rolls,
        fumble_confirmation: None,
    }
}

/// Record of an open-ended roll (no target): total speaks for itself.
#[derive(Debug, PartialEq, Eq)]
pub struct OpenRollResult {
    /// attribute + skill + luck-adjusted die total.
    pub total: i32,
    /// Natural die values in roll order, explosion re-rolls included.
    pub die_rolls: Vec<i32>,
    /// True when the (luck-adjusted) die showed 1: an open roll can still fumble.
    pub is_fumble: bool,
    pub fumble_confirmation: Option<i32>,
}

/// Rolls openly (no difficulty): exploding 10s, luck and fumbles apply,
/// interpretation of the total is up to the GM.
pub fn open_roll(
    attribute: i32,
    skill: i32,
    luck: i32,
    roller: &mut dyn DieRoller,
) -> OpenRollResult {
    let first = roller.d10();
    let mut die_rolls = vec![first];
    let adjusted_first = first + luck;

    if adjusted_first <= 1 {
        let confirmation = roller.d10();
        return OpenRollResult {
            total: attribute + skill + adjusted_first,
            die_rolls,
            is_fumble: true,
            fumble_confirmation: Some(confirmation),
        };
    }

    let mut die_total = adjusted_first;
    let mut last = adjusted_first;
    while last >= 10 {
        last = roller.d10();
        die_rolls.push(last);
        die_total += last;
    }

    OpenRollResult {
        total: attribute + skill + die_total,
        die_rolls,
        is_fumble: false,
        fumble_confirmation: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(attr: i32, skill: i32, luck: i32, target: i32, rolls: Vec<i32>) -> CheckResult {
        let mut roller = SequenceRoller::new(rolls);
        skill_check(attr, skill, luck, Difficulty::Custom(target), &mut roller)
    }

    #[test]
    fn test_auto_success_without_rolling() {
        // 7 + 8 = 15 reaches Normal: no dice consumed, always succeeds.
        let result = check(7, 8, 0, 15, vec![]);
        assert_eq!(result.outcome, Outcome::AutoSuccess);
        assert_eq!(result.total, 15);
        assert!(result.die_rolls.is_empty());
    }

    #[test]
    fn test_luck_counts_toward_auto_success() {
        let result = check(7, 6, 2, 15, vec![]);
        assert_eq!(result.outcome, Outcome::AutoSuccess);
    }

    #[test]
    fn test_plain_success_and_failure() {
        let success = check(5, 4, 0, 15, vec![7]);
        assert_eq!(success.outcome, Outcome::Success);
        assert_eq!(success.total, 16);

        let failure = check(5, 4, 0, 15, vec![5]);
        assert_eq!(failure.outcome, Outcome::Failure);
        assert_eq!(failure.total, 14);
    }

    #[test]
    fn test_exploding_tens_cascade() {
        // 10 -> 10 -> 3 = die total 23.
        let result = check(2, 0, 0, 20, vec![10, 10, 3]);
        assert_eq!(result.outcome, Outcome::Success);
        assert_eq!(result.total, 25);
        assert_eq!(result.die_rolls, vec![10, 10, 3]);
    }

    #[test]
    fn test_luck_turns_nine_into_exploding_ten() {
        // Rolled 9 + 1 luck counts as a natural 10 and explodes (house rule).
        let result = check(2, 2, 1, 20, vec![9, 6]);
        assert_eq!(result.die_rolls, vec![9, 6]);
        // 2 + 2 + (9 + 1) + 6 = 20
        assert_eq!(result.total, 20);
        assert_eq!(result.outcome, Outcome::Success);
    }

    #[test]
    fn test_fumble_severities() {
        let critical = check(5, 5, 0, 15, vec![1, 1]);
        assert_eq!(critical.outcome, Outcome::CriticalFailure);
        assert_eq!(critical.fumble_confirmation, Some(1));

        let embarrassing = check(5, 5, 0, 15, vec![1, 4]);
        assert_eq!(embarrassing.outcome, Outcome::EmbarrassingFailure);

        let normal = check(5, 5, 0, 15, vec![1, 8]);
        assert_eq!(normal.outcome, Outcome::Failure);
        assert_eq!(normal.fumble_confirmation, Some(8));
    }

    #[test]
    fn test_fumble_fails_even_when_total_would_succeed() {
        // 7 + 7 + 1 = 15 would exactly reach the target,
        // but a natural 1 is always a failure.
        let result = check(7, 7, 0, 15, vec![1, 8]);
        assert_eq!(result.outcome, Outcome::Failure);
        assert_eq!(result.total, 15);
    }

    #[test]
    fn test_luck_prevents_fumble() {
        // Rolled 1 + 1 luck counts as 2: no fumble, evaluated normally.
        let result = check(5, 5, 1, 15, vec![1]);
        assert_eq!(result.outcome, Outcome::Failure);
        assert_eq!(result.fumble_confirmation, None);
        assert_eq!(result.total, 12);
    }

    #[test]
    fn test_difficulty_targets() {
        assert_eq!(Difficulty::Easy.target(), 10);
        assert_eq!(Difficulty::Normal.target(), 15);
        assert_eq!(Difficulty::Hard.target(), 20);
        assert_eq!(Difficulty::Custom(35).target(), 35);
    }

    #[test]
    fn test_open_roll_explodes_and_sums() {
        let mut roller = SequenceRoller::new(vec![10, 4]);
        let result = open_roll(6, 3, 0, &mut roller);
        assert_eq!(result.total, 23);
        assert_eq!(result.die_rolls, vec![10, 4]);
        assert!(!result.is_fumble);
    }

    #[test]
    fn test_open_roll_fumbles() {
        let mut roller = SequenceRoller::new(vec![1, 3]);
        let result = open_roll(6, 3, 0, &mut roller);
        assert!(result.is_fumble);
        assert_eq!(result.fumble_confirmation, Some(3));
    }

    #[test]
    fn test_random_roller_stays_in_range() {
        let mut roller = RandomRoller;
        for _ in 0..1000 {
            let roll = roller.d10();
            assert!((1..=10).contains(&roll), "d10 out of range: {}", roll);
        }
    }

    #[test]
    #[should_panic(expected = "ran out of scripted values")]
    fn test_sequence_roller_exhaustion_panics() {
        let mut roller = SequenceRoller::new(vec![5]);
        roller.d10();
        roller.d10();
    }

    #[test]
    fn test_dice_expr_parse_display_roundtrip() {
        for s in ["5d6", "2d6+2", "1d3", "4d10", "2d6-1"] {
            let expr: DiceExpr = s.parse().unwrap();
            assert_eq!(expr.to_string(), s);
        }
        assert!("d6".parse::<DiceExpr>().is_err());
        assert!("2w6".parse::<DiceExpr>().is_err());
    }

    #[test]
    fn test_dice_expr_average_matches_wiki_examples() {
        // AK 5d6 -> 15, 9mm 2d6+2 -> 8 (noise formula examples)
        assert_eq!("5d6".parse::<DiceExpr>().unwrap().average(), 15);
        assert_eq!("2d6+2".parse::<DiceExpr>().unwrap().average(), 8);
    }

    #[test]
    fn test_dice_expr_roll() {
        let expr: DiceExpr = "2d6+3".parse().unwrap();
        let mut roller = SequenceRoller::new(vec![4, 6]);
        assert_eq!(expr.roll(&mut roller), 13);
    }

    #[test]
    fn test_random_roller_die_range() {
        let mut roller = RandomRoller;
        for sides in [3, 6, 10] {
            for _ in 0..200 {
                let roll = roller.die(sides);
                assert!((1..=sides).contains(&roll));
            }
        }
    }
}
