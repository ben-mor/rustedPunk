use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum DamageType {
    Blunt,
    Slashing,
    ArmorPiercing,
    HollowPoint,
}

impl fmt::Display for DamageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_type_serialization() {
        let damage_type = DamageType::ArmorPiercing;
        let serialized = toml::to_string(&damage_type).unwrap();
        let deserialized: DamageType = toml::from_str(&serialized).unwrap();
        assert_eq!(damage_type, deserialized);
    }
}
