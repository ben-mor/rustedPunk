use crate::character::Attribute;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Whether a trait helps or hinders. Both kinds count against the same
/// CP budget at character creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvantageKind {
    Advantage,
    Disadvantage,
}

/// What a modifier applies to.
///
/// `Tag` is a free-form hook: some tags are known to the engine
/// (see the `TAG_*` constants), others ("hören", "musik", …) are looked up by
/// the caller/GM when a fitting roll comes up, via
/// [`crate::Character::modifier_for_tag`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "of")]
pub enum ModifierTarget {
    Attribute(Attribute),
    /// Skill by name, applied automatically in `Character::check_skill`.
    Skill(String),
    Tag(String),
}

/// Engine-known tag: added to initiative rolls (Kampfreflexe).
pub const TAG_INITIATIVE: &str = "initiative";
/// Engine-known tag: enlarges the Prellschaden scale (Erhöhte Ausdauer).
pub const TAG_BRUISE_SCALE: &str = "prellschaden";
/// Engine-known tag: +1 = double healing rate (Schnelle Heilung),
/// −1 = half rate (Langsame Heilung).
pub const TAG_HEALING_RATE: &str = "heilrate";

/// A mechanical effect of an advantage or disadvantage.
// Field order matters for TOML: scalar `value` must serialize before the
// table-like `target`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifier {
    pub value: i32,
    pub target: ModifierTarget,
}

/// A GURPS-style Vor- oder Nachteil (see docs/rules/VorUndNachteile.wiki).
///
/// Many entries on the table's list are purely narrative — those simply have
/// no modifiers and the GM plays them out. Mechanical ones carry `modifiers`
/// that the engine applies automatically where it can.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advantage {
    pub name: String,
    pub kind: AdvantageKind,
    /// CP cost (always positive; disadvantages *grant* points at chargen,
    /// but count against the same budget).
    pub cp: i32,
    /// Level for leveled traits (Gute Ohren 2/4, Kampfreflexe light/big, …).
    /// 1 for unleveled ones.
    pub level: i32,
    pub description: String,
    pub modifiers: Vec<Modifier>,
}

impl Advantage {
    /// Creates a trait without mechanical effects (narrative-only).
    ///
    /// # Panics
    ///
    /// Panics if `cp` is negative — disadvantages use
    /// [`AdvantageKind::Disadvantage`] with a positive cost.
    pub fn new(name: String, kind: AdvantageKind, cp: i32, description: String) -> Self {
        assert!(
            cp >= 0,
            "Advantage '{}': cp must not be negative, got {} (use AdvantageKind::Disadvantage)",
            name,
            cp
        );
        Advantage {
            name,
            kind,
            cp,
            level: 1,
            description,
            modifiers: Vec::new(),
        }
    }

    pub fn with_modifier(mut self, target: ModifierTarget, value: i32) -> Self {
        self.modifiers.push(Modifier { target, value });
        self
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }
}

impl fmt::Display for Advantage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?}, {} CP)", self.name, self.kind, self.cp)
    }
}

/// Validates the chargen budget rule: up to 30 CP of traits in total, or
/// exactly ONE trait bigger than 30 plus at most 5 CP of others.
pub fn validate_budget(advantages: &[Advantage]) -> Result<(), String> {
    let total: i32 = advantages.iter().map(|a| a.cp).sum();
    if total <= 30 {
        return Ok(());
    }
    let big: Vec<&Advantage> = advantages.iter().filter(|a| a.cp > 30).collect();
    match big.len() {
        1 => {
            let rest = total - big[0].cp;
            if rest <= 5 {
                Ok(())
            } else {
                Err(format!(
                    "Advantage budget exceeded: '{}' ({} CP) allows at most 5 more CP, but the others total {}",
                    big[0].name, big[0].cp, rest
                ))
            }
        }
        0 => Err(format!(
            "Advantage budget exceeded: {} CP total, allowed are 30 (or one single trait above 30 plus 5)",
            total
        )),
        _ => Err(format!(
            "Advantage budget exceeded: only ONE trait above 30 CP is allowed, found {}",
            big.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(name: &str, cp: i32) -> Advantage {
        Advantage::new(
            name.to_string(),
            AdvantageKind::Advantage,
            cp,
            String::new(),
        )
    }

    #[test]
    fn test_budget_within_thirty() {
        let list = vec![
            plain("Literat", 10),
            plain("Kampfreflexe", 10),
            plain("Stimme", 10),
        ];
        assert!(validate_budget(&list).is_ok());
    }

    #[test]
    fn test_budget_one_big_plus_five() {
        // wiki example: Absolutes Gedächtnis (60) + Literat (5) is okay
        let list = vec![plain("Absolutes Gedächtnis", 60), plain("Lesen", 5)];
        assert!(validate_budget(&list).is_ok());
        // … Absolutes Gedächtnis + Literat + Höhere Bildung is not
        let list = vec![
            plain("Absolutes Gedächtnis", 60),
            plain("Lesen", 5),
            plain("Höhere Bildung", 15),
        ];
        assert!(validate_budget(&list).is_err());
    }

    #[test]
    fn test_budget_over_thirty_without_big_trait() {
        let list = vec![plain("A", 20), plain("B", 20)];
        let error = validate_budget(&list).unwrap_err();
        assert!(error.contains("40 CP total"), "{}", error);
    }

    #[test]
    fn test_budget_two_big_traits() {
        let list = vec![plain("A", 40), plain("B", 35)];
        assert!(validate_budget(&list).is_err());
    }

    #[test]
    #[should_panic(expected = "cp must not be negative")]
    fn test_negative_cp_panics() {
        plain("Broken", -10);
    }

    #[test]
    fn test_advantage_serialization() {
        let adv = Advantage::new(
            "Gute Ohren".to_string(),
            AdvantageKind::Advantage,
            1,
            "Alle Lausch-Würfe erhalten den Bonus".to_string(),
        )
        .with_level(2)
        .with_modifier(ModifierTarget::Tag("hören".to_string()), 2);
        let serialized = toml::to_string(&adv).unwrap();
        let deserialized: Advantage = toml::from_str(&serialized).unwrap();
        assert_eq!(adv, deserialized);
    }
}
