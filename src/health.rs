use crate::character::Attribute;

/// Wound state per the house rules (Regeln → Gesundheit): health is divided
/// into blocks of four hit points.
///
/// 1st block: scratches — no malus. 2nd block: serious wounds — −2 REF.
/// 3rd block: critical — REF/INT/COOL halved (round up). Blocks 4–8: mortal
/// wounds — all stats except LUCK and EMP divided by 3 (round up). Beyond the
/// 8th block the body is too disintegrated to still count as human.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WoundState {
    Uninjured,
    /// 1–4 damage: scratches and minor stuff.
    Light,
    /// 5–8 damage: wounds and unpleasantness. −2 REF.
    Serious,
    /// 9–12 damage: critical injuries. REF/INT/COOL halved, round up.
    Critical,
    /// 13–32 damage: Mortal 0 through Mortal 4. All stats except LUCK and EMP
    /// divided by 3, round up.
    Mortal(i32),
    /// More than 32 damage.
    Dead,
}

impl WoundState {
    pub fn from_damage(damage: i32) -> Self {
        match damage {
            ..=0 => WoundState::Uninjured,
            1..=4 => WoundState::Light,
            5..=8 => WoundState::Serious,
            9..=12 => WoundState::Critical,
            13..=32 => WoundState::Mortal((damage - 13) / 4),
            _ => WoundState::Dead,
        }
    }

    /// Applies this wound state's penalty to an attribute value.
    /// Halving and thirding round up, as the house rules demand.
    pub fn modify_attribute(self, attr: Attribute, value: i32) -> i32 {
        match self {
            WoundState::Uninjured | WoundState::Light => value,
            WoundState::Serious => {
                if attr == Attribute::Reflexes {
                    value - 2
                } else {
                    value
                }
            }
            WoundState::Critical => match attr {
                Attribute::Reflexes | Attribute::Intelligence | Attribute::Coolness => {
                    (value + 1) / 2
                }
                _ => value,
            },
            WoundState::Mortal(_) | WoundState::Dead => match attr {
                Attribute::Luck | Attribute::Empathy => value,
                _ => (value + 2) / 3,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wound_state_blocks() {
        assert_eq!(WoundState::from_damage(0), WoundState::Uninjured);
        assert_eq!(WoundState::from_damage(1), WoundState::Light);
        assert_eq!(WoundState::from_damage(4), WoundState::Light);
        assert_eq!(WoundState::from_damage(5), WoundState::Serious);
        assert_eq!(WoundState::from_damage(8), WoundState::Serious);
        assert_eq!(WoundState::from_damage(9), WoundState::Critical);
        assert_eq!(WoundState::from_damage(12), WoundState::Critical);
        assert_eq!(WoundState::from_damage(13), WoundState::Mortal(0));
        assert_eq!(WoundState::from_damage(16), WoundState::Mortal(0));
        assert_eq!(WoundState::from_damage(17), WoundState::Mortal(1));
        assert_eq!(WoundState::from_damage(32), WoundState::Mortal(4));
        assert_eq!(WoundState::from_damage(33), WoundState::Dead);
    }

    #[test]
    fn test_serious_penalizes_only_reflexes() {
        let state = WoundState::Serious;
        assert_eq!(state.modify_attribute(Attribute::Reflexes, 8), 6);
        assert_eq!(state.modify_attribute(Attribute::Intelligence, 8), 8);
        assert_eq!(state.modify_attribute(Attribute::Move, 8), 8);
    }

    #[test]
    fn test_critical_halves_ref_int_cool_rounding_up() {
        let state = WoundState::Critical;
        assert_eq!(state.modify_attribute(Attribute::Reflexes, 7), 4);
        assert_eq!(state.modify_attribute(Attribute::Intelligence, 8), 4);
        assert_eq!(state.modify_attribute(Attribute::Coolness, 5), 3);
        assert_eq!(state.modify_attribute(Attribute::Body, 7), 7);
        assert_eq!(state.modify_attribute(Attribute::Empathy, 7), 7);
    }

    #[test]
    fn test_mortal_thirds_everything_except_luck_and_empathy() {
        let state = WoundState::Mortal(0);
        assert_eq!(state.modify_attribute(Attribute::Reflexes, 9), 3);
        assert_eq!(state.modify_attribute(Attribute::Body, 7), 3);
        assert_eq!(state.modify_attribute(Attribute::Intelligence, 4), 2);
        assert_eq!(state.modify_attribute(Attribute::Luck, 9), 9);
        assert_eq!(state.modify_attribute(Attribute::Empathy, 9), 9);
    }
}
