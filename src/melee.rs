use crate::dice::DiceExpr;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Name of the shared melee base skill ("Nahkampf allgemein", includes Dodge).
/// Capped at level 3; above that the weapon classes specialize.
pub const SKILL_MELEE_GENERAL: &str = "Nahkampf";

/// The general melee level cap: beyond this, only specializations grow.
pub const MELEE_GENERAL_CAP: i32 = 3;

/// Melee weapon classes, the specializations above general level 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeleeClass {
    /// Knives etc.
    Short,
    /// Swords, clubs etc.
    Medium,
    /// Spears, battle staffs etc.
    Long,
}

impl MeleeClass {
    /// The skill name of this specialization on the character sheet.
    pub fn skill_name(self) -> &'static str {
        match self {
            MeleeClass::Short => "Nahkampf Kurz",
            MeleeClass::Medium => "Nahkampf Mittel",
            MeleeClass::Long => "Nahkampf Lang",
        }
    }
}

impl fmt::Display for MeleeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.skill_name())
    }
}

/// Martial arts styles at the table (Q26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MartialArtsStyle {
    /// Brawling: no key attacks.
    Pruegeln,
    /// Boxing: +3 Punch, +3 Sweep, +1 Block.
    Boxen,
    /// Wrestling: +2 Sweep, +4 Grapple, +3 Throw, +4 Hold, +2 Choke, +4 Escape.
    Ringen,
}

impl MartialArtsStyle {
    pub fn skill_name(self) -> &'static str {
        match self {
            MartialArtsStyle::Pruegeln => "Kampfkunst Prügeln",
            MartialArtsStyle::Boxen => "Kampfkunst Boxen",
            MartialArtsStyle::Ringen => "Kampfkunst Ringen",
        }
    }

    /// The style's key-attack bonus for an action, added to the attack roll.
    pub fn key_attack_bonus(self, action: MartialArtsAction) -> i32 {
        use MartialArtsAction::*;
        match self {
            MartialArtsStyle::Pruegeln => 0,
            MartialArtsStyle::Boxen => match action {
                Punch | Sweep => 3,
                Block => 1,
                _ => 0,
            },
            MartialArtsStyle::Ringen => match action {
                Grapple | Hold | Escape => 4,
                Throw => 3,
                Sweep | Choke => 2,
                _ => 0,
            },
        }
    }
}

impl fmt::Display for MartialArtsStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.skill_name())
    }
}

/// Martial arts actions (wiki Kampfkunstaktionen + the styles' key attacks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MartialArtsAction {
    /// Strike: 1d3 + DAM + skill (head, torso, arms).
    Punch,
    /// 1d6 + DAM + skill (legs, torso, arms).
    Kick,
    /// Leg sweep: sends the opponent to the ground, no damage.
    Sweep,
    /// Reduces damage.
    Block,
    /// Evades the damage entirely (−2 on the attack).
    Dodge,
    /// Knocks the opponent's weapon away.
    Disarm,
    /// Preparation for Throw, Hold or Choke.
    Grapple,
    /// After a grapple: 1d6 + DAM + skill, stun check at −2.
    Throw,
    /// Pins the opponent; +1 for every round the hold persists.
    Hold,
    /// Counter to Grapple or Hold.
    Escape,
    /// 1d6 + skill per round at the grapple/hold position.
    Choke,
}

impl MartialArtsAction {
    /// Base damage dice of this action, before DAM and skill level.
    /// `None` for actions that don't deal damage directly.
    pub fn base_damage(self) -> Option<DiceExpr> {
        use MartialArtsAction::*;
        match self {
            Punch => Some(DiceExpr::new(1, 3, 0)),
            Kick | Throw | Choke => Some(DiceExpr::new(1, 6, 0)),
            _ => None,
        }
    }

    /// Whether the DAM body modifier applies to this action's damage
    /// (it doesn't for choking — that's technique, not mass).
    pub fn applies_dam(self) -> bool {
        !matches!(self, MartialArtsAction::Choke)
    }
}

/// The hand-to-hand damage modifier (DAM / "DAMAGE MOD" on the character
/// sheet), per the CP2020 damage modifiers table in Reference Book 5.
pub fn dam_for_body(body: i32) -> i32 {
    match body {
        ..=2 => -2,
        3..=4 => -1,
        5..=7 => 0,
        8..=9 => 1,
        10 => 2,
        11..=12 => 4,
        13..=14 => 6,
        _ => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dam_table_boundaries() {
        assert_eq!(dam_for_body(2), -2);
        assert_eq!(dam_for_body(3), -1);
        assert_eq!(dam_for_body(5), 0);
        assert_eq!(dam_for_body(8), 1);
        assert_eq!(dam_for_body(10), 2);
        assert_eq!(dam_for_body(11), 4);
        assert_eq!(dam_for_body(14), 6);
        assert_eq!(dam_for_body(15), 8);
    }

    #[test]
    fn test_key_attack_bonuses() {
        use MartialArtsAction::*;
        use MartialArtsStyle::*;
        assert_eq!(Pruegeln.key_attack_bonus(Punch), 0);
        assert_eq!(Boxen.key_attack_bonus(Punch), 3);
        assert_eq!(Boxen.key_attack_bonus(Sweep), 3);
        assert_eq!(Boxen.key_attack_bonus(Block), 1);
        assert_eq!(Boxen.key_attack_bonus(Grapple), 0);
        assert_eq!(Ringen.key_attack_bonus(Grapple), 4);
        assert_eq!(Ringen.key_attack_bonus(Throw), 3);
        assert_eq!(Ringen.key_attack_bonus(Hold), 4);
        assert_eq!(Ringen.key_attack_bonus(Choke), 2);
        assert_eq!(Ringen.key_attack_bonus(Escape), 4);
        assert_eq!(Ringen.key_attack_bonus(Punch), 0);
    }

    #[test]
    fn test_base_damage() {
        assert_eq!(
            MartialArtsAction::Punch.base_damage(),
            Some(DiceExpr::new(1, 3, 0))
        );
        assert_eq!(
            MartialArtsAction::Kick.base_damage(),
            Some(DiceExpr::new(1, 6, 0))
        );
        assert_eq!(MartialArtsAction::Block.base_damage(), None);
        assert_eq!(MartialArtsAction::Sweep.base_damage(), None);
    }
}
