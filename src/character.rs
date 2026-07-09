use crate::advantages::{
    Advantage, ModifierTarget, TAG_BRUISE_SCALE, TAG_HEALING_RATE, TAG_INITIATIVE,
};
use crate::dice::{skill_check, CheckResult, DieRoller, Difficulty};
use crate::health::WoundState;
use crate::melee::{
    dam_for_body, MartialArtsAction, MartialArtsStyle, MeleeClass, MELEE_GENERAL_CAP,
    SKILL_MELEE_GENERAL,
};
use crate::{armor::HitZone, Armor};
use crate::{inventory::Inventory, DamageType};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    pub age: i32,
    pub current_damage: i32,
    /// Prellschaden scale: accumulated bruise points (0–4). Every 5 points
    /// convert into 1 real damage and the scale resets.
    pub current_bruise: i32,
    /// Malus from pure bruise damage, applied to (and consumed by) the
    /// character's next check.
    pub pending_roll_malus: i32,
    /// Healing progress in half-points: a healer day counts 2, a day without
    /// counts 1; every 2 half-points heal 1 damage (see [`Character::rest_day`]).
    pub healing_progress: i32,
    /// Available luck points. A persistent pool: spending survives the session,
    /// [`Character::start_session`] regenerates half the current base LUCK
    /// (rounded up). See [`Character::start_session`] for the three luck levels.
    pub current_luck: i32,
    pub damage_notes: String,
    pub worn_armor: Vec<Uuid>,
    pub skills: Vec<Skill>,
    pub advantages: Vec<Advantage>,
    pub attributes: Attributes,
    pub inventory: Inventory,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Attribute {
    Attractiveness,
    Move,
    Coolness,
    Empathy,
    Luck,
    Intelligence,
    Body,
    Reflexes,
    Tech,
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Attribute {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "attractiveness" => Ok(Self::Attractiveness),
            "move" => Ok(Self::Move),
            "coolness" => Ok(Self::Coolness),
            "empathy" => Ok(Self::Empathy),
            "luck" => Ok(Self::Luck),
            "intelligence" => Ok(Self::Intelligence),
            "body" => Ok(Self::Body),
            "reflexes" => Ok(Self::Reflexes),
            "tech" => Ok(Self::Tech),
            _ => Err(format!("Unknown attribute: {}", s)),
        }
    }
}

#[serde_as]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Attributes(
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")] BTreeMap<Attribute, AttributeValue>,
);

impl Deref for Attributes {
    type Target = BTreeMap<Attribute, AttributeValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (attr, value) in &self.0 {
            writeln!(f, "\t{}: {}", attr, value)?;
        }
        Ok(())
    }
}

/// Represents a character attribute with base and current values.
///
/// - `base`: The attribute value at character creation (natural/starting value)
/// - `actual`: The "current" value including semi-permanent modifications
///   (cyberware, training, long-term injuries). This is what appears on the
///   character sheet and persists between sessions.
///
/// For momentary effects (drugs, encumbrance, combat boosts), use
/// `Character::effective_attribute()` which calculates the actual roll value.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeValue {
    pub base: i32,
    pub actual: i32,
}

impl AttributeValue {
    pub fn new(actual: i32, base: i32) -> Self {
        AttributeValue { base, actual }
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.actual, self.base)
    }
}

/// Report of what a hit (or direct damage) did to the character.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HitOutcome {
    /// Real damage added to `current_damage` (head doubling and converted
    /// bruise damage included).
    pub real_damage: i32,
    /// Prellschaden points this hit put on the bruise scale
    /// (soft-armor absorption + BTM conversion).
    pub bruise_added: i32,
    /// Real damage that came from the bruise scale filling up
    /// (already part of `real_damage`).
    pub converted_bruise_damage: i32,
    /// The hit caused real damage (directly or via a full bruise scale):
    /// a KO check is required, see [`Character::ko_check`].
    pub ko_check_required: bool,
    /// The penetration cap kicked in: the shot exited through the back.
    pub through_and_through: bool,
}

impl Character {
    pub fn new(
        name: String,
        role: String,
        age: i32,
        att: i32,
        mov: i32,
        coo: i32,
        emp: i32,
        luck: i32,
        int: i32,
        body: i32,
        refl: i32,
        tec: i32,
    ) -> Self {
        let mut character = Character {
            name,
            role,
            age,
            attributes: Attributes(BTreeMap::new()),
            inventory: Inventory::new(),
            worn_armor: Vec::new(),
            current_damage: 0,
            current_bruise: 0,
            pending_roll_malus: 0,
            healing_progress: 0,
            current_luck: luck,
            damage_notes: "".to_string(),
            skills: Vec::new(),
            advantages: Vec::new(),
        };

        character.attributes.insert(
            Attribute::Attractiveness,
            AttributeValue {
                base: att,
                actual: att,
            },
        );
        character.attributes.insert(
            Attribute::Move,
            AttributeValue {
                base: mov,
                actual: mov,
            },
        );
        character.attributes.insert(
            Attribute::Coolness,
            AttributeValue {
                base: coo,
                actual: coo,
            },
        );
        character.attributes.insert(
            Attribute::Empathy,
            AttributeValue {
                base: emp,
                actual: emp,
            },
        );
        character.attributes.insert(
            Attribute::Luck,
            AttributeValue {
                base: luck,
                actual: luck,
            },
        );
        character.attributes.insert(
            Attribute::Intelligence,
            AttributeValue {
                base: int,
                actual: int,
            },
        );
        character.attributes.insert(
            Attribute::Body,
            AttributeValue {
                base: body,
                actual: body,
            },
        );
        character.attributes.insert(
            Attribute::Reflexes,
            AttributeValue {
                base: refl,
                actual: refl,
            },
        );
        character.attributes.insert(
            Attribute::Tech,
            AttributeValue {
                base: tec,
                actual: tec,
            },
        );

        character
    }

    pub fn print(&self) {
        println!(
            "\
Character {{ \n\
\tname: {}\n\
\trole: {}\n\
\tage: {}\n\
\tAttributes: {}\n\
\tInventory: {}\n\
}}",
            self.name, self.role, self.age, self.attributes, self.inventory,
        );
    }

    /// Body Type Modifier
    /// How much damage gets reduced when being hit, based on the body stat.
    pub fn btm(&self) -> i32 {
        match self.attributes.get(&Attribute::Body).unwrap().actual {
            ..=2 => 0,
            3..=4 => 1,
            5..=7 => 2,
            8..=9 => 3,
            10 => 4,
            _ => 5,
        }
    }

    /// The character's wound state, derived from the current damage.
    pub fn wound_state(&self) -> WoundState {
        WoundState::from_damage(self.current_damage)
    }

    /// Sum of all advantage/disadvantage modifiers for an attribute.
    pub fn modifier_for_attribute(&self, attr: Attribute) -> i32 {
        self.modifier_sum(|target| matches!(target, ModifierTarget::Attribute(a) if *a == attr))
    }

    /// Sum of all advantage/disadvantage modifiers for a skill (by name).
    pub fn modifier_for_skill(&self, skill_name: &str) -> i32 {
        self.modifier_sum(|target| matches!(target, ModifierTarget::Skill(s) if s == skill_name))
    }

    /// Sum of all advantage/disadvantage modifiers for a free-form tag.
    /// The engine applies the `TAG_*` tags itself; situational ones
    /// ("hören", "musik", …) are looked up by the caller when they fit a roll.
    pub fn modifier_for_tag(&self, tag: &str) -> i32 {
        self.modifier_sum(|target| matches!(target, ModifierTarget::Tag(t) if t == tag))
    }

    fn modifier_sum(&self, matches: impl Fn(&ModifierTarget) -> bool) -> i32 {
        self.advantages
            .iter()
            .flat_map(|advantage| &advantage.modifiers)
            .filter(|modifier| matches(&modifier.target))
            .map(|modifier| modifier.value)
            .sum()
    }

    /// Size of the Prellschaden scale: 5, enlarged by advantages like
    /// Erhöhte Ausdauer (modifier on the `prellschaden` tag).
    pub fn bruise_capacity(&self) -> i32 {
        5 + self.modifier_for_tag(TAG_BRUISE_SCALE)
    }

    /// Rolls initiative: 1d10 + effective REF, plus advantage modifiers on the
    /// `initiative` tag (Kampfreflexe).
    pub fn roll_initiative(&self, roller: &mut dyn DieRoller) -> i32 {
        roller.d10()
            + self.effective_attribute(Attribute::Reflexes)
            + self.modifier_for_tag(TAG_INITIATIVE)
    }

    /// A day of rest: heals 1 damage per day with a healer present,
    /// 1 per two days without (progress carries over between days).
    /// Schnelle Heilung (healing-rate tag +1) doubles the rate,
    /// Langsame Heilung (−1) halves it.
    pub fn rest_day(&mut self, healer_present: bool) {
        if self.current_damage == 0 || self.wound_state() == WoundState::Dead {
            return;
        }
        // progress is counted in quarter-days: 4 quarters heal 1 damage
        let mut increment = if healer_present { 4 } else { 2 };
        match self.modifier_for_tag(TAG_HEALING_RATE) {
            rate if rate > 0 => increment *= 2,
            rate if rate < 0 => increment /= 2,
            _ => {}
        }
        self.healing_progress += increment;
        self.current_damage = (self.current_damage - self.healing_progress / 4).max(0);
        self.healing_progress %= 4;
        if self.current_damage == 0 {
            self.healing_progress = 0;
        }
    }

    /// The hand-to-hand damage modifier (DAM, "DAMAGE MOD" on the sheet),
    /// based on the sheet BODY plus advantage modifiers.
    pub fn dam(&self) -> i32 {
        dam_for_body(
            self.attributes.get(&Attribute::Body).unwrap().actual
                + self.modifier_for_attribute(Attribute::Body),
        )
    }

    fn skill_level(&self, name: &str) -> Option<i32> {
        self.skills
            .iter()
            .find(|skill| skill.name == name)
            .map(|skill| skill.level)
    }

    /// The melee level used with a weapon class: the specialization if the
    /// character has it, otherwise the shared general skill (capped at 3).
    pub fn melee_level(&self, class: MeleeClass) -> i32 {
        self.skill_level(class.skill_name()).unwrap_or_else(|| {
            self.skill_level(SKILL_MELEE_GENERAL)
                .unwrap_or(0)
                .min(MELEE_GENERAL_CAP)
        })
    }

    /// Rolls a melee attack with a weapon class. Unfamiliar weapons within
    /// the class raise the difficulty by 3 (Q25).
    pub fn check_melee(
        &mut self,
        class: MeleeClass,
        familiar: bool,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let target = difficulty.target() + if familiar { 0 } else { 3 };
        let (attribute_value, level) = (
            self.effective_attribute(Attribute::Reflexes),
            self.melee_level(class),
        );
        self.spend_luck(luck)?;
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value - malus,
            level,
            luck,
            Difficulty::Custom(target),
            roller,
        ))
    }

    /// Rolls a dodge: always uses the general melee level (capped at 3),
    /// specializations don't help here (Q24).
    pub fn check_dodge(
        &mut self,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let (attribute_value, level) = (
            self.effective_attribute(Attribute::Reflexes),
            self.skill_level(SKILL_MELEE_GENERAL)
                .unwrap_or(0)
                .min(MELEE_GENERAL_CAP),
        );
        self.spend_luck(luck)?;
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value - malus,
            level,
            luck,
            difficulty,
            roller,
        ))
    }

    /// Rolls a martial arts action: REF + style skill + the style's
    /// key-attack bonus. Errors if the character doesn't know the style.
    pub fn check_martial_arts(
        &mut self,
        style: MartialArtsStyle,
        action: MartialArtsAction,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let level = self.skill_level(style.skill_name()).ok_or_else(|| {
            format!(
                "Character '{}' has no skill named '{}'",
                self.name,
                style.skill_name()
            )
        })?;
        let attribute_value = self.effective_attribute(Attribute::Reflexes);
        self.spend_luck(luck)?;
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value + style.key_attack_bonus(action) - malus,
            level,
            luck,
            difficulty,
            roller,
        ))
    }

    /// Rolls the damage of a martial arts action: base dice + DAM (where the
    /// body's mass matters) + style skill level 1:1 (Q26).
    /// `None` for actions that don't deal damage. Minimum 0.
    pub fn martial_arts_damage(
        &self,
        style: MartialArtsStyle,
        action: MartialArtsAction,
        roller: &mut dyn DieRoller,
    ) -> Option<i32> {
        let base = action.base_damage()?;
        let level = self.skill_level(style.skill_name()).unwrap_or(0);
        let dam = if action.applies_dam() { self.dam() } else { 0 };
        Some((base.roll(roller) + dam + level).max(0))
    }

    /// Rolls a ranged attack with a weapon: like [`Character::check_skill`]
    /// plus the weapon accuracy (WA) and +1 if the weapon has a scope.
    /// Autofire/burst modifiers are the caller's business
    /// (see `weapons::autofire_attack_modifier` / `BURST_ATTACK_BONUS`).
    pub fn check_ranged_attack(
        &mut self,
        weapon: &crate::weapons::Weapon,
        skill_name: &str,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let skill = self
            .skills
            .iter()
            .find(|skill| skill.name == skill_name)
            .ok_or_else(|| {
                format!(
                    "Character '{}' has no skill named '{}'",
                    self.name, skill_name
                )
            })?;
        let (attribute_value, skill_level) = (self.effective_attribute(skill.base), skill.level);
        let advantage_bonus = self.modifier_for_skill(skill_name);
        let weapon_bonus = weapon.weapon_accuracy + if weapon.scope { 1 } else { 0 };
        self.spend_luck(luck)?;
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value + advantage_bonus + weapon_bonus - malus,
            skill_level,
            luck,
            difficulty,
            roller,
        ))
    }

    /// Rolls a weapon's damage. Melee weapons add the wielder's DAM.
    pub fn weapon_damage(
        &self,
        weapon: &crate::weapons::Weapon,
        roller: &mut dyn DieRoller,
    ) -> i32 {
        let dam = if weapon.category.melee_class().is_some() {
            self.dam()
        } else {
            0
        };
        (weapon.damage.roll(roller) + dam).max(0)
    }

    /// The KO check after taking damage: BODY against 10, modified by the
    /// wound category's Stun malus (Light −0, Serious −1, Critical −2,
    /// Mortal 0 −3, one more per Mortal step).
    ///
    /// Uses the sheet BODY (plus advantage modifiers) WITHOUT the wound
    /// thirding — the category malus already covers the wound effect;
    /// applying both would double-count.
    ///
    /// On a failure the character is out of the fight (KO, on the floor
    /// screaming, …) and may repeat the roll every round; the first success
    /// means recovery. On a critical failure the GM decides — usually out
    /// for longer.
    pub fn ko_check(&self, roller: &mut dyn DieRoller) -> CheckResult {
        let body = self.attributes.get(&Attribute::Body).unwrap().actual
            + self.modifier_for_attribute(Attribute::Body)
            - self.wound_state().ko_malus();
        skill_check(body, 0, 0, Difficulty::Custom(10), roller)
    }

    /// The crippling check for one critical-or-worse injury, rolled directly
    /// after the fight (Q22): 5% chance without proper medical care, 0.5%
    /// with; Schnelle Heilung (healing-rate tag > 0) lowers that to 1% /
    /// 0.1%. Returns true when the body part is crippled — what that means
    /// concretely is the GM's call, the tool only reports it.
    pub fn crippling_check(&self, medical_care: bool, roller: &mut dyn DieRoller) -> bool {
        let fast_healer = self.modifier_for_tag(TAG_HEALING_RATE) > 0;
        let per_mille = match (medical_care, fast_healer) {
            (false, false) => 50,
            (true, false) => 5,
            (false, true) => 10,
            (true, true) => 1,
        };
        crate::dice::roll_per_mille(roller) < per_mille
    }

    /// The morning-after complication check: BODY against 10 + current damage.
    /// With a healer present (practically always) no check is needed and
    /// `None` is returned. A failed check means complications — interpreting
    /// them is up to the GM.
    pub fn complication_check(
        &mut self,
        healer_present: bool,
        roller: &mut dyn DieRoller,
    ) -> Option<CheckResult> {
        if healer_present {
            return None;
        }
        let difficulty = Difficulty::Custom(10 + self.current_damage);
        Some(skill_check(
            self.effective_attribute(Attribute::Body),
            0,
            0,
            difficulty,
            roller,
        ))
    }

    /// Returns the effective attribute value for dice rolls, including all
    /// temporary modifiers (advantages, wound penalties, encumbrance, etc.).
    ///
    /// Order: advantage modifiers adjust the sheet value, wound penalties
    /// modify that, encumbrance maluses are subtracted afterwards, the result
    /// never drops below 0 (see Q19).
    pub fn effective_attribute(&self, attr: Attribute) -> i32 {
        let mut value = self.wound_state().modify_attribute(
            attr,
            self.attributes[&attr].actual + self.modifier_for_attribute(attr),
        );

        match attr {
            Attribute::Reflexes => {
                value -= self.encumberance() + self.calculate_armor_encumberance()
            }
            Attribute::Move => value -= self.encumberance(),
            _ => {}
        }
        value.max(0)
    }

    /// Calculates the malus to movement and reflexes based on encumberance
    pub fn encumberance(&self) -> i32 {
        let inventory_weight = self.inventory.calculate_total_weight();
        let capacity = self.carry_capacity();
        match (inventory_weight * 10) / capacity {
            ..=4 => 0,
            5..=6 => 1,
            7..=9 => 2,
            10..=12 => 4,
            13..=15 => 6,
            _ => 8,
        }
    }

    /// Looks up the carry capacity of the character
    /// Returns grams.
    pub fn carry_capacity(&self) -> i32 {
        self.attributes.get(&Attribute::Body).unwrap().actual.max(0) * 10000
    }

    /// Looks up the deadlift capacity of the character
    /// Returns grams.
    pub fn deadlift(&self) -> i32 {
        self.carry_capacity() * 4
    }

    pub fn wear_armor(&mut self, armor_uuid: Uuid, underneath: Option<Uuid>) {
        if self.inventory.get_item(armor_uuid).is_none() {
            unreachable!("Armor_uuid not found in inventory");
        }
        if let Some(underneath_uuid) = underneath {
            if self.inventory.get_item(underneath_uuid).is_none() {
                unreachable!("underneath_uuid not found in inventory");
            }
            if let Some(index) = self
                .worn_armor
                .iter()
                .position(|&uuid| uuid == underneath_uuid)
            {
                // Insert the new armor at that index (pushes existing armor one position higher)
                self.worn_armor.insert(index, armor_uuid);
            } else {
                panic!("tried to wear something underneath an armor that isn't worn.");
            }
        } else {
            self.worn_armor.push(armor_uuid);
        }
    }

    /// Hit the character with some damage
    ///
    /// This will apply damage to the armor (outer to inner) and then to the character.
    ///
    pub fn hit(
        &mut self,
        damage: i32,
        zone: HitZone,
        damage_type: DamageType,
        is_gunshot: bool,
        roller: &mut dyn DieRoller,
    ) -> HitOutcome {
        let mut remaining_damage = damage;
        let mut soft_absorbed = 0;

        for i in (0..self.worn_armor.len()).rev() {
            let armor_uuid = self.worn_armor[i];
            let armor_item = self.inventory.get_item_mut(armor_uuid);
            let armor_opt = armor_item.unwrap_or_else(|| panic!("There was an Armor Uuid in the worn armor list ({}), but no corresponding item in the inventory.", armor_uuid))
                .as_any_mut().downcast_mut::<Armor>();
            let armor = armor_opt.unwrap_or_else(|| panic!("There was an Armor in the worn_armor list ({}), that wasn't an Armor in the Inventory.",
                armor_uuid));
            let is_hard = armor.is_hard;
            let damage_result = armor.hit(remaining_damage, zone, damage_type);
            remaining_damage = damage_result.remaining_damage;
            // House rule: only hits caught by SOFT armor go onto the bruise
            // scale; hard armor absorbs without consequence.
            if !is_hard {
                soft_absorbed += damage_result.absorbed_damage;
            }
        }

        let mut through_and_through = false;
        if damage_type == DamageType::HollowPoint {
            // Q17: the mushroomed projectile doubles its damage as soon as it
            // enters flesh — and doesn't exit the body, so no penetration cap.
            remaining_damage *= 2;
        } else if is_gunshot && remaining_damage > 4 {
            // House rule: a gunshot doing more than 4 damage rolls 1d10 — that
            // is the maximum extra damage before the shot exits through the back.
            let cap = 4 + roller.d10();
            if remaining_damage > cap {
                remaining_damage = cap;
                through_and_through = true;
            }
        }

        // fire damage ignores the ">8 damage" crippling rule
        let can_cripple = damage_type != DamageType::Fire;
        let mut outcome = self.resolve_damage(remaining_damage, soft_absorbed, zone, can_cripple);
        outcome.through_and_through = through_and_through;
        outcome
    }

    /// This ignores all armor and applies damage directly.
    /// It will subtract the BTM first.
    pub fn take_damage(&mut self, damage: i32, zone: HitZone) -> HitOutcome {
        self.resolve_damage(damage, 0, zone, true)
    }

    /// Core damage resolution after armor: BTM conversion, bruise scale,
    /// crippling check and head doubling.
    ///
    /// House rules applied here:
    /// - BTM is subtracted from the incoming damage (at least 1 real damage
    ///   remains) and the subtracted amount becomes Prellschaden.
    /// - Every 5 Prellschaden points convert into 1 real damage and require a
    ///   KO check. A hit that causes ONLY Prellschaden instead puts a malus of
    ///   that amount on the character's next roll.
    /// - The crippling check (8+ zone damage after BTM) uses the UNDOUBLED
    ///   value; head doubling is applied afterwards.
    fn resolve_damage(
        &mut self,
        incoming: i32,
        armor_bruise: i32,
        zone: HitZone,
        can_cripple: bool,
    ) -> HitOutcome {
        let mut real_damage = incoming.max(0);
        let mut bruise = armor_bruise;

        if real_damage > 0 {
            let converted_by_btm = self.btm().min(real_damage - 1).max(0);
            real_damage -= converted_by_btm;
            bruise += converted_by_btm;
        }

        let capacity = self.bruise_capacity();
        self.current_bruise += bruise;
        let converted_bruise_damage = self.current_bruise / capacity;
        self.current_bruise %= capacity;
        // taking real damage requires a KO check (see Character::ko_check);
        // pure Prellschaden doesn't — it gives the next-roll malus instead
        let ko_check_required = real_damage > 0 || converted_bruise_damage > 0;

        if real_damage == 0 && converted_bruise_damage == 0 && bruise > 0 {
            self.pending_roll_malus += bruise;
        }

        // Crippling check against the unmodified zone damage, doubling after.
        // Fire never cripples (can_cripple = false, Q: Hausregeln Niederbrennen).
        let crippled = can_cripple && real_damage >= 8;
        let mut applied_damage = real_damage;
        if zone == HitZone::Head {
            applied_damage *= 2;
        }
        applied_damage += converted_bruise_damage;
        self.current_damage += applied_damage;

        if crippled {
            if matches!(zone, HitZone::Head | HitZone::Chest | HitZone::Vitals) {
                self.current_damage = 100;
                self.damage_notes = format!("YOU ARE DEAD!\n{}", self.damage_notes);
            } else {
                self.damage_notes = format!(
                    "HitZone {} destroyed. You are now at least Mortal 0 and about to die.\n{}",
                    zone, self.damage_notes
                );
                if self.current_damage <= 12 {
                    self.current_damage = 13;
                }
            }
        }

        HitOutcome {
            real_damage: applied_damage,
            bruise_added: bruise,
            converted_bruise_damage,
            ko_check_required,
            through_and_through: false,
        }
    }

    pub fn calculate_armor_encumberance(&self) -> i32 {
        let mut encumberance = 0;
        let mut covered_zones = HashSet::new();
        for i in (0..self.worn_armor.len()).rev() {
            let armor = self.get_armor_ref_on_index(i);
            let zones: Vec<HitZone> = armor.protection_current.keys().copied().collect();
            // if there is already armor covering that zone, add the encumberance of the new
            // armor, but at least 1
            let mut any_zone_covered = false;
            for zone in zones {
                if covered_zones.contains(&zone) {
                    any_zone_covered = true;
                } else {
                    covered_zones.insert(zone);
                }
            }
            if any_zone_covered {
                encumberance += armor.encumberance.max(1);
            } else {
                encumberance += armor.encumberance;
            }
        }
        encumberance
    }

    fn get_armor_ref_on_index(&self, i: usize) -> &Armor {
        let armor_uuid = self.worn_armor[i];
        let armor_item = self.inventory.get_item(armor_uuid);
        let armor_opt = armor_item.unwrap_or_else(|| panic!("There was an Armor Uuid in the worn armor list ({}), but no corresponding item in the inventory.", armor_uuid))
            .as_any().downcast_ref::<Armor>();
        let armor = armor_opt.unwrap_or_else(|| panic!("There was an Armor in the worn_armor list ({}), that wasn't an Armor in the Inventory.",
            armor_uuid));
        armor
    }

    /// Spends luck points from the character's pool.
    ///
    /// Returns an error when the pool doesn't cover it (or `points` is negative);
    /// the pool is unchanged in that case.
    pub fn spend_luck(&mut self, points: i32) -> Result<(), String> {
        if points < 0 {
            return Err(format!(
                "Cannot spend a negative amount of luck: {}",
                points
            ));
        }
        if points > self.current_luck {
            return Err(format!(
                "Character '{}' has only {} luck points, tried to spend {}",
                self.name, self.current_luck, points
            ));
        }
        self.current_luck -= points;
        Ok(())
    }

    /// Starts a new game session: regenerates luck points.
    ///
    /// Luck has three levels:
    /// - starting base (`AttributeValue.base`): the chargen value, never changes
    /// - current base (`AttributeValue.actual`): permanently reducible via
    ///   [`Character::sacrifice_luck`]; regeneration rate and cap come from here
    /// - current pool (`Character.current_luck`): fluctuates with every roll
    ///
    /// Regenerates half the current base (rounded up), capped at the current
    /// base. Example: current base 9, 8 already spent (1 left) → +5 → 6.
    pub fn start_session(&mut self) {
        let luck = self.attributes.get(&Attribute::Luck).unwrap();
        let regenerated = (luck.actual + 1) / 2;
        self.current_luck = (self.current_luck + regenerated).min(luck.actual);
    }

    /// Permanently sacrifices luck for an extreme "the world now turns in your
    /// favor" event: lowers the current base LUCK (`actual`), which also lowers
    /// the regeneration rate and cap. The starting base is untouched. The
    /// current pool is clamped to the new base if it now exceeds it.
    pub fn sacrifice_luck(&mut self, points: i32) -> Result<(), String> {
        if points < 0 {
            return Err(format!(
                "Cannot sacrifice a negative amount of luck: {}",
                points
            ));
        }
        let luck = self.attributes.get_mut(&Attribute::Luck).unwrap();
        if points > luck.actual {
            return Err(format!(
                "Character '{}' has only {} base luck, tried to sacrifice {}",
                self.name, luck.actual, points
            ));
        }
        luck.actual -= points;
        self.current_luck = self.current_luck.min(luck.actual);
        Ok(())
    }

    /// Rolls a check on one of the character's skills.
    ///
    /// Uses the effective base attribute (encumbrance and other temporary
    /// maluses included) plus the skill level. `luck` is the number of luck
    /// points the player commits before the roll; they are deducted from
    /// [`Character::current_luck`] (also on an auto-success — committed is
    /// committed) and the check fails with an error if the pool is too small.
    ///
    /// Returns an error naming the skill if the character doesn't have it —
    /// rolling untrained is a deliberate decision, not a fallback:
    /// use [`Character::check_attribute`] for that.
    pub fn check_skill(
        &mut self,
        skill_name: &str,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let skill = self
            .skills
            .iter()
            .find(|skill| skill.name == skill_name)
            .ok_or_else(|| {
                format!(
                    "Character '{}' has no skill named '{}'",
                    self.name, skill_name
                )
            })?;
        let (attribute_value, skill_level) = (self.effective_attribute(skill.base), skill.level);
        let advantage_bonus = self.modifier_for_skill(skill_name);
        self.spend_luck(luck)?;
        // pure bruise damage puts a malus on the NEXT roll — consume it
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value + advantage_bonus - malus,
            skill_level,
            luck,
            difficulty,
            roller,
        ))
    }

    /// Rolls a check on a bare attribute (untrained, skill level 0).
    /// Committed luck is deducted like in [`Character::check_skill`].
    pub fn check_attribute(
        &mut self,
        attribute: Attribute,
        luck: i32,
        difficulty: Difficulty,
        roller: &mut dyn DieRoller,
    ) -> Result<CheckResult, String> {
        let attribute_value = self.effective_attribute(attribute);
        self.spend_luck(luck)?;
        let malus = std::mem::take(&mut self.pending_roll_malus);
        Ok(skill_check(
            attribute_value - malus,
            0,
            luck,
            difficulty,
            roller,
        ))
    }

    pub fn print_skills(&self) {
        println!("Skills:");
        for skill in &self.skills {
            let attr = self.attributes.get(&skill.base).unwrap();
            let total = skill.level + attr.actual;
            println!("\t {}: {}", skill.name, total);
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub base: Attribute,
    pub level: i32,
    pub level_up_modifier: i32,
}

impl Skill {
    pub fn print(self) {
        println!(
            "Skillname {} {{
            \tbase: {}
            \tlevel: {}
            \tlevel up modifier: {}
        }}",
            self.name, self.base, self.level, self.level_up_modifier
        )
    }

    pub fn new(name: String, base: Attribute, level: i32, level_up_modifierer: i32) -> Self {
        Skill {
            name,
            base,
            level,
            level_up_modifier: level_up_modifierer,
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name: {}, level: {}, base: {}",
            self.name, self.level, self.base
        )
    }
}

pub struct List(pub Vec<Skill>);

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Extract the value using tuple indexing,
        // and create a reference to `vec`.
        let vec = &self.0;

        write!(f, "")?;

        // Iterate over `v` in `vec` while enumerating the iteration
        // count in `count`.
        for (count, v) in vec.iter().enumerate() {
            // For every element except the first, add a comma.
            // Use the ? operator to return on errors.
            if count != 0 {
                writeln!(f)?;
            }
            write!(f, "{}", v)?;
        }

        // Close the opened bracket and return a fmt::Result value.
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor::tests::*;
    use crate::inventory::Item;
    use toml;

    fn populated_character() -> Character {
        let mut character = Character::new(
            "test-Name".to_string(),
            "test-Role".to_string(),
            25,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
        );
        let kevlar_shirt = kev_shirt();
        let kevlar_shirt_uuid = kevlar_shirt.item.uuid;
        let flak_vest = flak_vest();
        let flak_vest_uuid = flak_vest.item.uuid;
        let kevlar_tights = kevlar_tights();
        let kevlar_tights_uuid = kevlar_tights.item.uuid;
        let leather_boots = leather_boots();
        let leather_boots_uuid = leather_boots.item.uuid;
        let long_leather_cloak = long_leather_cloak();
        let long_leather_cloak_uuid = long_leather_cloak.item.uuid;
        let braces = braces();
        let braces_uuid = braces.item.uuid;
        let helmet = helmet();
        let helmet_uuid = helmet.item.uuid;

        character.inventory.push(Box::new(kevlar_shirt));
        character.inventory.push(Box::new(flak_vest));
        character.inventory.push(Box::new(kevlar_tights));
        character.inventory.push(Box::new(leather_boots));
        character.inventory.push(Box::new(long_leather_cloak));
        character.inventory.push(Box::new(braces));
        character.inventory.push(Box::new(helmet));

        character.wear_armor(kevlar_shirt_uuid, None);
        character.wear_armor(flak_vest_uuid, None);
        character.wear_armor(kevlar_tights_uuid, Some(flak_vest_uuid));
        character.wear_armor(leather_boots_uuid, None);
        character.wear_armor(long_leather_cloak_uuid, None);
        character.wear_armor(braces_uuid, Some(flak_vest_uuid));
        character.wear_armor(helmet_uuid, None);

        character
    }

    #[test]
    #[rustfmt::skip]
    fn test_multi_layer_armor_penetration_blunt_arm_noncrippling() {
        let mut character = populated_character();
        let zone = HitZone::RightArm;
        let damage = 45;
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let outcome = character.hit(damage, zone, DamageType::Blunt, false, &mut roller);
        let test_context = "multi-layer-arm";
        assert_armor_protection(&character,test_context,"Kevlar Shirt",       zone,  0, HitZone::Chest,   10,);
        assert_armor_protection(&character,test_context,"Flak Vest",          zone, 19, HitZone::Chest,   20,);
        assert_armor_protection(&character,test_context,"Kevlar Tights",      zone,  0, HitZone::LeftLeg, 10,);
        assert_armor_protection(&character,test_context,"Leather Boots",      zone,  0, HitZone::LeftFoot, 4,);
        assert_armor_protection(&character,test_context,"Long Leather Cloak", zone,  3, HitZone::Chest,    4,);
        assert_armor_protection(&character,test_context,"Braces",             zone,  9, HitZone::LeftArm, 10,);
        assert_armor_protection(&character,test_context,"Helmet",             zone,  0, HitZone::Head,    15,);

        // soft armor on arm (braces 10 + cloak 4) = 14 -> Prellschaden
        // hard armor on arm (flak vest) absorbs 20 without consequence
        // remaining zone damage: 45 - 34 = 11; BTM 4 converts to Prellschaden -> 7 real
        // 7 < 8: NOT crippling. Prellschaden 14 + 4 = 18 -> 3 real damage, scale at 3
        let expected_damage = 7 + 3;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
        assert_eq!(character.current_bruise, 3);
        assert!(outcome.ko_check_required);
        assert!(!outcome.through_and_through);
        assert_eq!(character.pending_roll_malus, 0);
        assert_eq!(character.damage_notes, "", "7 zone damage after BTM must not cripple");
    }

    #[test]
    #[rustfmt::skip]
    fn test_multi_layer_armor_penetration_blunt_head_noncrippling() {
        let mut character = populated_character();
        let zone = HitZone::Head;
        let damage = 16;
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        character.hit(damage, zone, DamageType::Blunt, false, &mut roller);
        let test_context = "multi-layer-head";
        assert_armor_protection(&character,test_context,"Kevlar Shirt",       zone,  0, HitZone::Chest,   10,);
        assert_armor_protection(&character,test_context,"Flak Vest",          zone,  0, HitZone::Chest,   20,);
        assert_armor_protection(&character,test_context,"Kevlar Tights",      zone,  0, HitZone::LeftLeg, 10,);
        assert_armor_protection(&character,test_context,"Leather Boots",      zone,  0, HitZone::LeftFoot, 4,);
        assert_armor_protection(&character,test_context,"Long Leather Cloak", zone,  0, HitZone::Chest,    4,);
        assert_armor_protection(&character,test_context,"Braces",             zone,  0, HitZone::LeftArm, 10,);
        assert_armor_protection(&character,test_context,"Helmet",             zone, 14, HitZone::Vitals,   0,);

        // hard helmet absorbs 15 without Prellschaden; 1 remains.
        // BTM can't reduce below 1 real damage. Head damage is doubled.

        let expected_damage = 2;
        assert_eq!(character.current_damage, expected_damage, "{}: Expected {} damage but was {}", test_context, expected_damage, character.current_damage);
    }

    #[test]
    fn test_take_crippling_arm_damage() {
        let mut character = populated_character();
        let zone = HitZone::LeftArm;
        let damage = 15; // -4 BTM!
        character.take_damage(damage, zone);
        let test_context = "crippling-arm";

        // damage of 8 or more is crippling and the person immediately goes into mortal 0 state, BTM or not.

        let expected_damage = 13;
        assert_eq!(
            character.current_damage, expected_damage,
            "{}: Expected {} damage but was {}",
            test_context, expected_damage, character.current_damage
        );
    }

    #[test]
    fn test_take_crippling_vitals_damage() {
        let mut character = populated_character();
        let zone = HitZone::Vitals;
        let damage = 12;
        character.take_damage(damage, zone);
        let test_context = "crippling-vitals";

        // damage of 8 or more is crippling and on the vitals you just die.

        let expected_damage = 100;
        assert_eq!(
            character.current_damage, expected_damage,
            "{}: Expected {} damage but was {}",
            test_context, expected_damage, character.current_damage
        );
    }

    #[test]
    fn test_take_crippling_head_damage() {
        let mut character = populated_character();
        let zone = HitZone::Head;
        // 12 incoming - 4 BTM = 8 after BTM: crippling on the head means death.
        let damage = 12;
        character.take_damage(damage, zone);
        let test_context = "crippling-head";

        let expected_damage = 100;
        assert_eq!(
            character.current_damage, expected_damage,
            "{}: Expected {} damage but was {}",
            test_context, expected_damage, character.current_damage
        );
    }

    #[test]
    fn test_gunshot_penetration_cap() {
        let mut character = unencumbered_shooter(); // BODY 10, BTM 4, no armor
                                                    // 20 incoming, gunshot: cap roll d10 = 3 -> max 4 + 3 = 7 before through-and-through
        let mut roller = crate::dice::SequenceRoller::new(vec![3]);
        let outcome = character.hit(20, HitZone::Stomach, DamageType::Blunt, true, &mut roller);
        assert!(outcome.through_and_through);
        // capped at 7, BTM converts 4 -> 3 real damage + 4 bruise
        assert_eq!(character.current_damage, 3);
        assert_eq!(character.current_bruise, 4);
    }

    #[test]
    fn test_melee_hit_is_not_capped() {
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let outcome = character.hit(20, HitZone::Stomach, DamageType::Blunt, false, &mut roller);
        assert!(!outcome.through_and_through);
        // 20 - BTM 4 = 16 real: 8+ -> stomach crippled, at least Mortal 0
        assert!(outcome.real_damage >= 16);
    }

    #[test]
    fn test_pure_bruise_hit_gives_next_roll_malus() {
        let mut character = unencumbered_shooter();
        // soft armor that swallows the whole hit
        let vest = kev_shirt();
        let vest_uuid = vest.item.uuid;
        character.inventory.push(Box::new(vest));
        character.wear_armor(vest_uuid, None);

        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let outcome = character.hit(3, HitZone::Chest, DamageType::Blunt, true, &mut roller);
        assert_eq!(outcome.real_damage, 0);
        assert_eq!(outcome.bruise_added, 3);
        assert!(!outcome.ko_check_required);
        assert_eq!(character.pending_roll_malus, 3);

        // the malus applies to exactly one (the next) roll and is then consumed
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character
            .check_skill("Pistole", 0, Difficulty::Normal, &mut roller)
            .unwrap();
        // REF 8 + Pistole 4 - malus 3 + die 5 = 14
        assert_eq!(result.total, 14);
        assert_eq!(character.pending_roll_malus, 0);
    }

    #[test]
    fn test_bruise_scale_converts_every_five_points() {
        let mut character = unencumbered_shooter();
        let outcome = character.take_damage(0, HitZone::Chest);
        assert_eq!(outcome.real_damage, 0);

        // three hits of 3 damage each: BTM 4 can only convert damage-1 = 2
        // -> each hit: 1 real damage + 2 bruise
        for _ in 0..2 {
            character.take_damage(3, HitZone::Chest);
        }
        assert_eq!(character.current_damage, 2);
        assert_eq!(character.current_bruise, 4);

        let outcome = character.take_damage(3, HitZone::Chest);
        // third hit: bruise reaches 6 -> 1 converted damage, scale at 1
        assert_eq!(outcome.converted_bruise_damage, 1);
        assert!(outcome.ko_check_required);
        assert_eq!(character.current_damage, 4);
        assert_eq!(character.current_bruise, 1);
    }

    #[test]
    fn test_wound_penalties_in_effective_attributes() {
        let mut character = unencumbered_shooter(); // REF 8, INT 5, BODY 10
        character.current_damage = 6; // Serious: -2 REF
        assert_eq!(character.effective_attribute(Attribute::Reflexes), 6);
        assert_eq!(character.effective_attribute(Attribute::Intelligence), 5);

        character.current_damage = 10; // Critical: REF/INT/COOL halved, round up
        assert_eq!(character.effective_attribute(Attribute::Reflexes), 4);
        assert_eq!(character.effective_attribute(Attribute::Intelligence), 3);
        assert_eq!(character.effective_attribute(Attribute::Body), 10);

        character.current_damage = 14; // Mortal 0: thirds, except LUCK/EMP
        assert_eq!(character.effective_attribute(Attribute::Reflexes), 3);
        assert_eq!(character.effective_attribute(Attribute::Body), 4);
        assert_eq!(character.effective_attribute(Attribute::Luck), 5);
        assert_eq!(
            character.wound_state(),
            crate::health::WoundState::Mortal(0)
        );
    }

    #[test]
    fn test_hollow_point_doubles_in_flesh_and_never_exits() {
        // Ben's Q17 example: 8 damage against 4-SP soft armor. The armor
        // consumes 4 (-> Prellschaden); the remaining 4 double in the flesh,
        // so 8 (minus BTM) reach the character. No through-and-through.
        let mut character = unencumbered_shooter(); // BODY 10, BTM 4
        let cloak = long_leather_cloak(); // soft, 4 SP, covers Chest
        let cloak_uuid = cloak.item.uuid;
        character.inventory.push(Box::new(cloak));
        character.wear_armor(cloak_uuid, None);

        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let outcome = character.hit(
            8,
            HitZone::Chest,
            DamageType::HollowPoint,
            true,
            &mut roller,
        );

        assert!(!outcome.through_and_through, "hollow point never exits");
        // doubled to 8, BTM converts 4 -> 4 real; bruise = 4 (armor) + 4 (BTM)
        // = 8 -> 1 converted damage, scale at 3
        assert_eq!(character.current_damage, 4 + 1);
        assert_eq!(character.current_bruise, 3);
        assert!(outcome.ko_check_required);
    }

    #[test]
    fn test_rest_day_healing_rates() {
        let mut character = unencumbered_shooter();
        character.current_damage = 6;

        // without a healer: 1 point per two days
        character.rest_day(false);
        assert_eq!(character.current_damage, 6);
        character.rest_day(false);
        assert_eq!(character.current_damage, 5);

        // with a healer: 1 point per day (carried half-progress stays intact)
        character.rest_day(true);
        assert_eq!(character.current_damage, 4);

        // healing stops at 0 and never goes negative
        character.current_damage = 1;
        character.rest_day(true);
        character.rest_day(true);
        assert_eq!(character.current_damage, 0);
    }

    #[test]
    fn test_complication_check_only_without_healer() {
        let mut character = unencumbered_shooter(); // BODY 10
        character.current_damage = 4;
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        assert!(character.complication_check(true, &mut roller).is_none());

        // without healer: BODY vs 10 + damage = 14. BODY 10 + die 5 = 15: passed.
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character.complication_check(false, &mut roller).unwrap();
        assert_eq!(result.target, 14);
        assert!(result.outcome.is_success());

        // heavily wounded: wound penalties make BODY worse (Mortal thirds it)
        character.current_damage = 14;
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character.complication_check(false, &mut roller).unwrap();
        // effective BODY 4 + die 5 = 9 vs 24: failed
        assert!(!result.outcome.is_success());
    }

    #[test]
    fn test_advantage_modifiers_in_skill_checks() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget};
        let mut character = unencumbered_shooter(); // REF 8, Pistole 4
        character.advantages.push(
            Advantage::new(
                "Maschinenflüsterer".to_string(),
                AdvantageKind::Advantage,
                10,
                "+3 auf Wartung und Reparatur".to_string(),
            )
            .with_modifier(ModifierTarget::Skill("Pistole".to_string()), 3),
        );
        // the bonus makes REF 8 + Pistole 4 + 3 = 15: auto-success vs Normal
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let result = character
            .check_skill("Pistole", 0, Difficulty::Normal, &mut roller)
            .unwrap();
        assert_eq!(result.outcome, crate::dice::Outcome::AutoSuccess);

        // vs Hard the die is rolled and the bonus still counts
        let mut roller = crate::dice::SequenceRoller::new(vec![3]);
        let result = character
            .check_skill("Pistole", 0, Difficulty::Hard, &mut roller)
            .unwrap();
        // REF 8 + Pistole 4 + Bonus 3 + die 3 = 18
        assert_eq!(result.total, 18);
    }

    #[test]
    fn test_advantage_attribute_modifier() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget};
        let mut character = unencumbered_shooter(); // ATT 5
        character.advantages.push(
            Advantage::new(
                "Erweitertes Sichtfeld".to_string(),
                AdvantageKind::Advantage,
                1,
                "210° Sicht, -2 ATTR".to_string(),
            )
            .with_modifier(ModifierTarget::Attribute(Attribute::Attractiveness), -2),
        );
        assert_eq!(character.effective_attribute(Attribute::Attractiveness), 3);
    }

    #[test]
    fn test_erhoehte_ausdauer_enlarges_bruise_scale() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget, TAG_BRUISE_SCALE};
        let mut character = unencumbered_shooter();
        character.advantages.push(
            Advantage::new(
                "Erhöhte Ausdauer".to_string(),
                AdvantageKind::Advantage,
                4,
                "2 CP / Punkt".to_string(),
            )
            .with_level(2)
            .with_modifier(ModifierTarget::Tag(TAG_BRUISE_SCALE.to_string()), 2),
        );
        assert_eq!(character.bruise_capacity(), 7);
        // 6 bruise points: would convert on a 5-scale, not on a 7-scale
        character.take_damage(0, HitZone::Chest);
        character.current_bruise = 6;
        let outcome = character.take_damage(0, HitZone::Chest);
        assert_eq!(outcome.converted_bruise_damage, 0);
        assert_eq!(character.current_bruise, 6);
    }

    #[test]
    fn test_healing_rate_advantages() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget, TAG_HEALING_RATE};
        // Schnelle Heilung: double rate -> 2 per day with a healer
        let mut character = unencumbered_shooter();
        character.advantages.push(
            Advantage::new(
                "Schnelle Heilung".to_string(),
                AdvantageKind::Advantage,
                5,
                "doppelte Heilrate".to_string(),
            )
            .with_modifier(ModifierTarget::Tag(TAG_HEALING_RATE.to_string()), 1),
        );
        character.current_damage = 6;
        character.rest_day(true);
        assert_eq!(character.current_damage, 4);

        // Langsame Heilung: half rate -> 1 per four days without a healer
        let mut character = unencumbered_shooter();
        character.advantages.push(
            Advantage::new(
                "Langsame Heilung".to_string(),
                AdvantageKind::Disadvantage,
                5,
                "halbe Heilrate".to_string(),
            )
            .with_modifier(ModifierTarget::Tag(TAG_HEALING_RATE.to_string()), -1),
        );
        character.current_damage = 6;
        for _ in 0..3 {
            character.rest_day(false);
        }
        assert_eq!(character.current_damage, 6);
        character.rest_day(false);
        assert_eq!(character.current_damage, 5);
    }

    #[test]
    fn test_initiative_with_kampfreflexe() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget, TAG_INITIATIVE};
        let mut character = unencumbered_shooter(); // REF 8
        let mut roller = crate::dice::SequenceRoller::new(vec![6]);
        assert_eq!(character.roll_initiative(&mut roller), 14);

        character.advantages.push(
            Advantage::new(
                "Kampfreflexe".to_string(),
                AdvantageKind::Advantage,
                5,
                "+3 Initiative und Wahrnehmung in Konflikten".to_string(),
            )
            .with_modifier(ModifierTarget::Tag(TAG_INITIATIVE.to_string()), 3),
        );
        let mut roller = crate::dice::SequenceRoller::new(vec![6]);
        assert_eq!(character.roll_initiative(&mut roller), 17);
    }

    #[test]
    fn test_situational_tag_lookup() {
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget};
        let mut character = unencumbered_shooter();
        character.advantages.push(
            Advantage::new(
                "Gute Ohren".to_string(),
                AdvantageKind::Advantage,
                1,
                "Lausch-Bonus".to_string(),
            )
            .with_level(2)
            .with_modifier(ModifierTarget::Tag("hören".to_string()), 2),
        );
        assert_eq!(character.modifier_for_tag("hören"), 2);
        assert_eq!(character.modifier_for_tag("sehen"), 0);
    }

    #[test]
    fn test_ko_check_maluses_follow_wound_track() {
        let mut character = unencumbered_shooter(); // BODY 10

        // uninjured: BODY 10 vs 10 -> auto-success, no roll needed
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let result = character.ko_check(&mut roller);
        assert_eq!(result.outcome, crate::dice::Outcome::AutoSuccess);

        // Serious (Stun -1): BODY 9 + die 2 = 11 vs 10 -> recovers
        character.current_damage = 6;
        let mut roller = crate::dice::SequenceRoller::new(vec![2]);
        assert!(character.ko_check(&mut roller).outcome.is_success());

        // Mortal 2 (Stun -5): BODY 5 + die 4 = 9 -> stays down this round...
        character.current_damage = 22;
        let mut roller = crate::dice::SequenceRoller::new(vec![4]);
        assert!(!character.ko_check(&mut roller).outcome.is_success());
        // ...but may repeat every round and recovers on the first success
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        assert!(character.ko_check(&mut roller).outcome.is_success());

        // critical failure is reported for the GM to decide (out for longer)
        let mut roller = crate::dice::SequenceRoller::new(vec![1, 1]);
        assert_eq!(
            character.ko_check(&mut roller).outcome,
            crate::dice::Outcome::CriticalFailure
        );
    }

    #[test]
    fn test_real_damage_requires_ko_check() {
        let mut character = unencumbered_shooter();
        let outcome = character.take_damage(6, HitZone::Chest);
        assert!(outcome.ko_check_required);

        // pure Prellschaden: no KO check, next-roll malus instead
        let mut character = unencumbered_shooter();
        let vest = kev_shirt();
        let vest_uuid = vest.item.uuid;
        character.inventory.push(Box::new(vest));
        character.wear_armor(vest_uuid, None);
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let outcome = character.hit(3, HitZone::Chest, DamageType::Blunt, true, &mut roller);
        assert!(!outcome.ko_check_required);
        assert_eq!(character.pending_roll_malus, 3);
    }

    #[test]
    fn test_melee_specialization_continues_the_scale() {
        use crate::melee::{MeleeClass, SKILL_MELEE_GENERAL};
        let mut character = unencumbered_shooter(); // REF 8
        character.skills.push(Skill::new(
            SKILL_MELEE_GENERAL.to_string(),
            Attribute::Reflexes,
            3,
            1,
        ));
        character.skills.push(Skill::new(
            MeleeClass::Short.skill_name().to_string(),
            Attribute::Reflexes,
            4,
            1,
        ));
        // Ben's Q23 example: 1d10+4 with short weapons, 1d10+3 with medium
        assert_eq!(character.melee_level(MeleeClass::Short), 4);
        assert_eq!(character.melee_level(MeleeClass::Medium), 3);
        assert_eq!(character.melee_level(MeleeClass::Long), 3);

        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character
            .check_melee(MeleeClass::Short, true, 0, Difficulty::Hard, &mut roller)
            .unwrap();
        // REF 8 + Kurz 4 + die 5 = 17
        assert_eq!(result.total, 17);
    }

    #[test]
    fn test_melee_general_is_capped_at_three() {
        use crate::melee::{MeleeClass, SKILL_MELEE_GENERAL};
        let mut character = unencumbered_shooter();
        // an (invalid) general level above the cap doesn't leak into rolls
        character.skills.push(Skill::new(
            SKILL_MELEE_GENERAL.to_string(),
            Attribute::Reflexes,
            5,
            1,
        ));
        assert_eq!(character.melee_level(MeleeClass::Medium), 3);
    }

    #[test]
    fn test_unfamiliar_weapon_raises_difficulty() {
        use crate::melee::MeleeClass;
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character
            .check_melee(MeleeClass::Short, false, 0, Difficulty::Normal, &mut roller)
            .unwrap();
        assert_eq!(result.target, 18);
    }

    #[test]
    fn test_dodge_ignores_specialization() {
        use crate::melee::{MeleeClass, SKILL_MELEE_GENERAL};
        let mut character = unencumbered_shooter();
        character.skills.push(Skill::new(
            SKILL_MELEE_GENERAL.to_string(),
            Attribute::Reflexes,
            3,
            1,
        ));
        character.skills.push(Skill::new(
            MeleeClass::Short.skill_name().to_string(),
            Attribute::Reflexes,
            6,
            1,
        ));
        let mut roller = crate::dice::SequenceRoller::new(vec![5]);
        let result = character
            .check_dodge(0, Difficulty::Hard, &mut roller)
            .unwrap();
        // REF 8 + general 3 (NOT Kurz 6) + die 5 = 16
        assert_eq!(result.total, 16);
    }

    #[test]
    fn test_martial_arts_check_and_damage() {
        use crate::melee::{MartialArtsAction, MartialArtsStyle};
        let mut character = unencumbered_shooter(); // REF 8, BODY 10 -> DAM +2
        character.skills.push(Skill::new(
            MartialArtsStyle::Boxen.skill_name().to_string(),
            Attribute::Reflexes,
            4,
            1,
        ));

        // Boxen Punch: REF 8 + skill 4 + key attack +3 + die 4 = 19
        let mut roller = crate::dice::SequenceRoller::new(vec![4]);
        let result = character
            .check_martial_arts(
                MartialArtsStyle::Boxen,
                MartialArtsAction::Punch,
                0,
                Difficulty::Hard,
                &mut roller,
            )
            .unwrap();
        assert_eq!(result.total, 19);

        // Punch damage: 1d3 (2) + DAM 2 + skill 4 = 8
        let mut roller = crate::dice::SequenceRoller::new(vec![2]);
        let damage = character
            .martial_arts_damage(
                MartialArtsStyle::Boxen,
                MartialArtsAction::Punch,
                &mut roller,
            )
            .unwrap();
        assert_eq!(damage, 8);

        // Sweep deals no direct damage
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        assert!(character
            .martial_arts_damage(
                MartialArtsStyle::Boxen,
                MartialArtsAction::Sweep,
                &mut roller
            )
            .is_none());

        // unknown style errors
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let error = character
            .check_martial_arts(
                MartialArtsStyle::Ringen,
                MartialArtsAction::Grapple,
                0,
                Difficulty::Normal,
                &mut roller,
            )
            .unwrap_err();
        assert!(error.contains("Kampfkunst Ringen"), "{}", error);
    }

    #[test]
    fn test_crippling_check_percentages() {
        let character = unencumbered_shooter();
        // no care: 5% -> 049 cripples, 050 doesn't
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 4, 9]);
        assert!(character.crippling_check(false, &mut roller));
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 5, 10]);
        assert!(!character.crippling_check(false, &mut roller));
        // with care: 0.5% -> 004 cripples, 005 doesn't
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 10, 4]);
        assert!(character.crippling_check(true, &mut roller));
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 10, 5]);
        assert!(!character.crippling_check(true, &mut roller));

        // Schnelle Heilung: 1% / 0.1%
        use crate::advantages::{Advantage, AdvantageKind, ModifierTarget, TAG_HEALING_RATE};
        let mut character = unencumbered_shooter();
        character.advantages.push(
            Advantage::new(
                "Schnelle Heilung".to_string(),
                AdvantageKind::Advantage,
                5,
                String::new(),
            )
            .with_modifier(ModifierTarget::Tag(TAG_HEALING_RATE.to_string()), 1),
        );
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 10, 9]);
        assert!(character.crippling_check(false, &mut roller));
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 1, 10]);
        assert!(!character.crippling_check(false, &mut roller));
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 10, 10]);
        assert!(character.crippling_check(true, &mut roller)); // 000 < 1
        let mut roller = crate::dice::SequenceRoller::new(vec![10, 10, 1]);
        assert!(!character.crippling_check(true, &mut roller));
    }

    #[test]
    fn test_ranged_attack_applies_weapon_accuracy_and_scope() {
        use crate::weapons::{Weapon, WeaponCategory};
        let mut character = unencumbered_shooter(); // REF 8, Pistole 4
        let catalog = Weapon::catalog();
        let mut smg = catalog
            .iter()
            .find(|weapon| weapon.category == WeaponCategory::Smg)
            .unwrap()
            .clone();
        // Uzi WA +1: REF 8 + Pistole 4 + WA 1 + die 4 = 17
        let mut roller = crate::dice::SequenceRoller::new(vec![4]);
        let result = character
            .check_ranged_attack(&smg, "Pistole", 0, Difficulty::Hard, &mut roller)
            .unwrap();
        assert_eq!(result.total, 17);

        // scope adds +1
        smg.scope = true;
        let mut roller = crate::dice::SequenceRoller::new(vec![4]);
        let result = character
            .check_ranged_attack(&smg, "Pistole", 0, Difficulty::Hard, &mut roller)
            .unwrap();
        assert_eq!(result.total, 18);
    }

    #[test]
    fn test_melee_weapon_damage_adds_dam() {
        use crate::weapons::{Weapon, WeaponCategory};
        let character = unencumbered_shooter(); // BODY 10 -> DAM +2
        let catalog = Weapon::catalog();
        let knife = catalog
            .iter()
            .find(|weapon| weapon.category == WeaponCategory::Knife)
            .unwrap();
        // Combat Knife 1d6+3: die 4 + 3 + DAM 2 = 9
        let mut roller = crate::dice::SequenceRoller::new(vec![4]);
        assert_eq!(character.weapon_damage(knife, &mut roller), 9);

        // ranged weapons don't add DAM
        let ak = catalog
            .iter()
            .find(|weapon| weapon.category == WeaponCategory::Rifle)
            .unwrap();
        let mut roller = crate::dice::SequenceRoller::new(vec![3, 3, 3, 3, 3]);
        assert_eq!(character.weapon_damage(ak, &mut roller), 15);
    }

    #[test]
    fn test_fire_damage_never_cripples() {
        let mut character = unencumbered_shooter(); // BTM 4
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        // 20 fire damage, no armor: 20 - 4 BTM = 16 zone damage -> would
        // cripple/kill for any other type, but fire ignores the 8+ rule
        let outcome = character.hit(20, HitZone::Chest, DamageType::Fire, false, &mut roller);
        assert_eq!(character.damage_notes, "");
        assert!(outcome.real_damage >= 16);
        assert_ne!(character.current_damage, 100);
    }

    #[test]
    fn test_head_damage_below_crippling_doubles_but_does_not_kill() {
        let mut character = populated_character();
        // 8 incoming - 4 BTM = 4 after BTM: below the crippling limit.
        // The crippling check uses the UNDOUBLED value; the damage doubles after.
        let outcome = character.take_damage(8, HitZone::Head);
        assert_eq!(character.current_damage, 8);
        assert_eq!(outcome.real_damage, 8);
        assert_eq!(character.damage_notes, "");
        // the 4 points BTM took went onto the bruise scale
        assert_eq!(character.current_bruise, 4);
    }

    fn assert_armor_protection(
        character: &Character,
        test_context: &str,
        armor_name: &str,
        hit_zone: HitZone,
        expected_remaining_protection: i32,
        unhit_zone: HitZone,
        expected_original_protection_in_unhit_zone: i32,
    ) {
        let armor_list = character.inventory.get_all_armor();
        let armor = armor_list
            .iter()
            .find(|armor_piece| armor_piece.item.name.eq(&armor_name))
            .unwrap();
        assert_armor_on_hit_zone(
            test_context,
            armor_name,
            hit_zone,
            expected_remaining_protection,
            armor,
        );
        assert_armor_on_hit_zone(
            test_context,
            armor_name,
            unhit_zone,
            expected_original_protection_in_unhit_zone,
            armor,
        );
    }

    fn assert_armor_on_hit_zone(
        test_context: &str,
        armor_name: &str,
        hit_zone: HitZone,
        expected_remaining_protection: i32,
        armor: &Armor,
    ) {
        let current_protection = armor.protection_current.get(&hit_zone);
        if expected_remaining_protection == 0 {
            assert!(
                current_protection.is_none() || current_protection.unwrap() == &0,
                "{}: Expected no protection, but some was found on {} {}",
                test_context,
                armor_name,
                hit_zone
            );
        } else {
            assert!(
                current_protection.is_some()
                    && current_protection.unwrap() == &expected_remaining_protection,
                "{}: Expected some protection, but none was found on {} {}",
                test_context,
                armor_name,
                hit_zone
            )
        }
    }

    #[test]
    fn test_wear_armor() {
        let mut character = Character::new(
            "Name".to_string(),
            "Role".to_string(),
            25,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
            10,
        );

        let inner_armor = leather_boots();
        let outer_armor = long_leather_cloak();

        let outer_armor_uuid = outer_armor.item.uuid;
        let inner_armor_uuid = inner_armor.item.uuid;

        character.inventory.push(Box::new(inner_armor));
        character.inventory.push(Box::new(outer_armor));

        character.wear_armor(outer_armor_uuid, None);
        character.wear_armor(inner_armor_uuid, Some(outer_armor_uuid));
        assert_eq!(
            character.worn_armor,
            vec![inner_armor_uuid, outer_armor_uuid]
        );
    }

    #[test]
    fn test_attribute_value_serialization() {
        let attribute_value = AttributeValue::new(5, 5);
        let serialized_attribute = toml::to_string(&attribute_value).unwrap();
        let deserialized_attribute: AttributeValue = toml::from_str(&serialized_attribute).unwrap();
        assert_eq!(attribute_value, deserialized_attribute);
    }

    #[test]
    fn test_attribute_enum_serialization() {
        let attribute = Attribute::Reflexes;
        let serialized = toml::to_string(&attribute).unwrap();
        let deserialized: Attribute = toml::from_str(&serialized).unwrap();
        assert_eq!(attribute, deserialized);
    }

    #[test]
    fn test_skill_serialization() {
        let skill = Skill::new("Schleichen".to_string(), Attribute::Reflexes, 2, 3);
        let serialized = toml::to_string(&skill).unwrap();
        let deserialized: Skill = toml::from_str(&serialized).unwrap();
        assert_eq!(skill, deserialized);
    }

    #[test]
    fn test_attributes_serialization() {
        let mut attributes = Attributes(BTreeMap::new());
        attributes.insert(Attribute::Body, AttributeValue::new(7, 7));
        attributes.insert(Attribute::Reflexes, AttributeValue::new(6, 6));
        let serialized = toml::to_string(&attributes).unwrap();
        let deserialized: Attributes = toml::from_str(&serialized).unwrap();
        assert_eq!(attributes, deserialized);
    }

    #[test]
    fn test_character_serialization() {
        let character = populated_character();
        let serialized_character_option = toml::to_string(&character);
        let serialized_character = serialized_character_option.unwrap();
        print!("serialized_character: {:?}", serialized_character);
        let deserialized_character: Character = toml::from_str(&serialized_character).unwrap();
        assert_eq!(
            character, deserialized_character,
            "character serialization round-trip didn't work {}",
            character
        );
    }

    fn unencumbered_shooter() -> Character {
        let mut character = Character::new(
            "Shooter".to_string(),
            "Solo".to_string(),
            25,
            5,
            6,
            5,
            5,
            5,
            5,
            10,
            8,
            5,
        );
        character
            .skills
            .push(Skill::new("Pistole".to_string(), Attribute::Reflexes, 4, 1));
        character
    }

    #[test]
    fn test_check_skill_uses_attribute_and_level() {
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![3]);
        let result = character
            .check_skill("Pistole", 0, Difficulty::Normal, &mut roller)
            .unwrap();
        // REF 8 + Pistole 4 + die 3 = 15
        assert_eq!(result.total, 15);
        assert!(result.outcome.is_success());
    }

    #[test]
    fn test_check_skill_unknown_skill_errors() {
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let error = character
            .check_skill(
                "Unterwasserkorbflechten",
                0,
                Difficulty::Normal,
                &mut roller,
            )
            .unwrap_err();
        assert_eq!(
            error,
            "Character 'Shooter' has no skill named 'Unterwasserkorbflechten'"
        );
    }

    #[test]
    fn test_check_skill_spends_committed_luck() {
        let mut character = unencumbered_shooter();
        assert_eq!(character.current_luck, 5);
        let mut roller = crate::dice::SequenceRoller::new(vec![3]);
        character
            .check_skill("Pistole", 2, Difficulty::Hard, &mut roller)
            .unwrap();
        assert_eq!(character.current_luck, 3);
    }

    #[test]
    fn test_check_skill_rejects_overspent_luck() {
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let error = character
            .check_skill("Pistole", 6, Difficulty::Normal, &mut roller)
            .unwrap_err();
        assert_eq!(
            error,
            "Character 'Shooter' has only 5 luck points, tried to spend 6"
        );
        assert_eq!(character.current_luck, 5);
    }

    #[test]
    fn test_start_session_regenerates_half_current_base_luck() {
        let mut character = unencumbered_shooter();
        // current base LUCK 9 to mirror the example from the table rules
        character
            .attributes
            .insert(Attribute::Luck, AttributeValue::new(9, 9));
        character.current_luck = 1;
        character.start_session();
        // 1 + ceil(9/2) = 6
        assert_eq!(character.current_luck, 6);

        // regeneration is capped at the current base
        character.current_luck = 8;
        character.start_session();
        assert_eq!(character.current_luck, 9);
    }

    #[test]
    fn test_sacrifice_luck_lowers_current_base_and_regen() {
        let mut character = unencumbered_shooter();
        // starting base 9, still at full current base and pool
        character
            .attributes
            .insert(Attribute::Luck, AttributeValue::new(9, 9));
        character.current_luck = 9;

        character.sacrifice_luck(4).unwrap();
        let luck = character.attributes.get(&Attribute::Luck).unwrap();
        // starting base untouched, current base lowered, pool clamped
        assert_eq!(luck.base, 9);
        assert_eq!(luck.actual, 5);
        assert_eq!(character.current_luck, 5);

        // regen now runs off the current base: 0 -> ceil(5/2) = 3, capped at 5
        character.current_luck = 0;
        character.start_session();
        assert_eq!(character.current_luck, 3);
        character.start_session();
        character.start_session();
        assert_eq!(character.current_luck, 5);
    }

    #[test]
    fn test_sacrifice_luck_rejects_more_than_current_base() {
        let mut character = unencumbered_shooter();
        let error = character.sacrifice_luck(6).unwrap_err();
        assert_eq!(
            error,
            "Character 'Shooter' has only 5 base luck, tried to sacrifice 6"
        );
    }

    #[test]
    fn test_check_skill_applies_encumbrance_malus() {
        let mut character = unencumbered_shooter();
        // Body 10 -> capacity 100kg; 50kg load -> encumbrance malus 1 on REF.
        character.inventory.push(Box::new(Item::new(
            None,
            "Schrottkiste".to_string(),
            1,
            50_000,
            0,
            "heavy junk".to_string(),
        )));
        let mut roller = crate::dice::SequenceRoller::new(vec![3]);
        let result = character
            .check_skill("Pistole", 0, Difficulty::Normal, &mut roller)
            .unwrap();
        // effective REF 7 + Pistole 4 + die 3 = 14: the malus costs the success
        assert_eq!(result.total, 14);
        assert!(!result.outcome.is_success());
    }

    #[test]
    fn test_check_attribute_auto_success() {
        let mut character = unencumbered_shooter();
        let mut roller = crate::dice::SequenceRoller::new(vec![]);
        let result = character
            .check_attribute(Attribute::Body, 0, Difficulty::Custom(10), &mut roller)
            .unwrap();
        assert_eq!(result.outcome, crate::dice::Outcome::AutoSuccess);
    }

    #[test]
    fn test_character_carry_capacity() {
        let character = populated_character();
        let carry_capacity = character.carry_capacity();
        assert_eq!(carry_capacity, 100_000);
    }

    #[test]
    fn test_character_deadlift() {
        let character = populated_character();
        let deadlift = character.deadlift();
        assert_eq!(deadlift, 400_000);
    }

    #[test]
    fn test_character_armor_encumberance() {
        let mut character = populated_character();
        assert_eq!(character.calculate_armor_encumberance(), 4);
        for _i in 0..3 {
            let kev_shirt = kev_shirt();
            let uuid = kev_shirt.item.uuid;
            character.inventory.push(Box::new(kev_shirt));
            character.worn_armor.push(uuid);
        }
        assert_eq!(character.calculate_armor_encumberance(), 7); // +1 per layer
        let flak_vest = flak_vest();
        let uuid = flak_vest.item.uuid;
        character.inventory.push(Box::new(flak_vest));
        character.worn_armor.push(uuid);
        assert_eq!(character.calculate_armor_encumberance(), 9); // +1 per layer +1 vest
    }

    #[test]
    fn test_character_encumberance() {
        let mut character = populated_character();
        let basic_ref = character.effective_attribute(Attribute::Reflexes);
        assert_eq!(character.encumberance(), 0);
        assert_eq!(character.inventory.calculate_total_weight(), 5_900);
        for _i in 0..44 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 49_900);
        assert_eq!(character.encumberance(), 0);
        character.inventory.push(Box::new(kev_shirt()));
        assert_eq!(character.inventory.calculate_total_weight(), 50_900);
        assert_eq!(character.encumberance(), 1);
        assert_eq!(
            character.effective_attribute(Attribute::Reflexes),
            basic_ref - 1
        );
        assert_eq!(character.effective_attribute(Attribute::Move), 9);
        for _i in 0..20 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 70_900);
        assert_eq!(character.encumberance(), 2);
        for _i in 0..20 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 90_900);
        assert_eq!(character.encumberance(), 2);
        for _i in 0..9 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 99_900);
        assert_eq!(character.encumberance(), 2);
        character.inventory.push(Box::new(kev_shirt()));
        assert_eq!(character.inventory.calculate_total_weight(), 100_900);
        assert_eq!(character.encumberance(), 4);
        for _i in 0..30 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 130_900);
        assert_eq!(character.encumberance(), 6);
        for _i in 0..30 {
            character.inventory.push(Box::new(kev_shirt()));
        }
        assert_eq!(character.inventory.calculate_total_weight(), 160_900);
        assert_eq!(character.encumberance(), 8);
        // effective attributes never go below 0, even when the malus exceeds them
        assert_eq!(
            character.effective_attribute(Attribute::Reflexes),
            (basic_ref - 8).max(0)
        );
        assert_eq!(character.effective_attribute(Attribute::Move), 2);
    }
}
