use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash)]
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
