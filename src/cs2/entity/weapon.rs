use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use crate::cs2::{CS2, entity::player::Player};

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, EnumIter, AsRefStr, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Weapon {
    #[default]
    Unknown,

    Knife,

    // Pistols
    Cz75A,
    Deagle,
    DualBerettas,
    FiveSeven,
    Glock,
    P2000,
    P250,
    Revolver,
    Tec9,
    Usp,

    // SMGs
    Bizon,
    Mac10,
    Mp5Sd,
    Mp7,
    Mp9,
    P90,
    Ump45,

    // LMGs
    M249,
    Negev,

    // Shotguns
    Mag7,
    Nova,
    Sawedoff,
    Xm1014,

    // Rifles
    Ak47,
    Aug,
    Famas,
    Galilar,
    M4A4,
    M4A1,
    Sg556,

    // Snipers
    Awp,
    G3SG1,
    Scar20,
    Ssg08,

    // Utility
    Taser,

    // Grenades
    Flashbang,
    HeGrenade,
    Smoke,
    Molotov,
    Decoy,
    Incendiary,

    // Bomb
    C4,
}

impl Weapon {
    pub fn from_handle(handle: i32, cs2: &CS2) -> Option<Self> {
        if handle == 0 {
            return None;
        }
        let index = handle as usize & 0xFFF;
        let entity = Player::get_client_entity(cs2, index)?;
        Some(Self::from_entity(entity, cs2))
    }

    pub fn from_entity(entity: usize, cs2: &CS2) -> Self {
        let weapon_index: u16 = cs2.process.read(
            entity
                + cs2.offsets.weapon.attribute_manager
                + cs2.offsets.weapon.item
                + cs2.offsets.econ_item_view.item_definition_index,
        );
        Self::from_index(weapon_index)
    }

    pub fn clip_ammo(entity: usize, cs2: &CS2) -> i32 {
        cs2.process.read(entity + cs2.offsets.weapon.clip_primary)
    }

    pub fn reserve_ammo(entity: usize, cs2: &CS2) -> i32 {
        cs2.process.read(entity + cs2.offsets.weapon.reserve_ammo)
    }

    pub fn from_index(index: u16) -> Self {
        use Weapon::*;

        match index {
            1 => Deagle,
            2 => DualBerettas,
            3 => FiveSeven,
            4 => Glock,
            7 => Ak47,
            8 => Aug,
            9 => Awp,
            10 => Famas,
            11 => G3SG1,
            13 => Galilar,
            14 => M249,
            16 => M4A4,
            17 => Mac10,
            19 => P90,
            23 => Mp5Sd,
            24 => Ump45,
            25 => Xm1014,
            26 => Bizon,
            27 => Mag7,
            28 => Negev,
            29 => Sawedoff,
            30 => Tec9,
            31 => Taser,
            32 => P2000,
            33 => Mp7,
            34 => Mp9,
            35 => Nova,
            36 => P250,
            38 => Scar20,
            39 => Sg556,
            40 => Ssg08,
            41 => Knife,
            42 => Knife,
            43 => Flashbang,
            44 => HeGrenade,
            45 => Smoke,
            46 => Molotov,
            47 => Decoy,
            48 => Incendiary,
            49 => C4,
            59 => Knife,
            60 => M4A1,
            61 => Usp,
            63 => Cz75A,
            64 => Revolver,
            80 => Knife,
            500 => Knife,
            505 => Knife,
            506 => Knife,
            507 => Knife,
            508 => Knife,
            509 => Knife,
            512 => Knife,
            514 => Knife,
            515 => Knife,
            516 => Knife,
            519 => Knife,
            520 => Knife,
            522 => Knife,
            523 => Knife,
            _ => Unknown,
        }
    }

    pub fn to_icon(&self) -> &'static str {
        match self {
            Weapon::Unknown => "",
            Weapon::Knife => "\u{e052}",
            Weapon::Cz75A => "\u{e02b}",
            Weapon::Deagle => "\u{e04a}",
            Weapon::DualBerettas => "\u{e056}",
            Weapon::FiveSeven => "\u{e03a}",
            Weapon::Glock => "\u{e023}",
            Weapon::P2000 => "\u{e05a}",
            Weapon::P250 => "\u{e04f}",
            Weapon::Revolver => "\u{e003}",
            Weapon::Tec9 => "\u{e00e}",
            Weapon::Usp => "\u{e037}",
            Weapon::Bizon => "\u{e00c}",
            Weapon::Mac10 => "\u{e049}",
            Weapon::Mp5Sd => "\u{e039}",
            Weapon::Mp7 => "\u{e047}",
            Weapon::Mp9 => "\u{e040}",
            Weapon::P90 => "\u{e035}",
            Weapon::Ump45 => "\u{e01f}",
            Weapon::M249 => "\u{e055}",
            Weapon::Negev => "\u{e019}",
            Weapon::Mag7 => "\u{e032}",
            Weapon::Nova => "\u{e02d}",
            Weapon::Sawedoff => "\u{e031}",
            Weapon::Xm1014 => "\u{e01d}",
            Weapon::Ak47 => "\u{e008}",
            Weapon::Aug => "\u{e060}",
            Weapon::Famas => "\u{e025}",
            Weapon::Galilar => "\u{e000}",
            Weapon::M4A4 => "\u{e042}",
            Weapon::M4A1 => "\u{e015}",
            Weapon::Sg556 => "\u{e00d}",
            Weapon::Awp => "\u{e007}",
            Weapon::G3SG1 => "\u{e04b}",
            Weapon::Scar20 => "\u{e009}",
            Weapon::Ssg08 => "\u{e03e}",
            Weapon::Taser => "\u{e021}",
            Weapon::Flashbang => "\u{e05e}",
            Weapon::HeGrenade => "\u{e00b}",
            Weapon::Smoke => "\u{e02e}",
            Weapon::Molotov => "\u{e024}",
            Weapon::Decoy => "\u{e028}",
            Weapon::Incendiary => "\u{e01a}",
            Weapon::C4 => "\u{e01e}",
        }
    }

    pub fn base_damage(&self) -> f32 {
        use Weapon::*;
        match self {
            Awp => 115.0,
            Ssg08 => 88.0,
            G3SG1 | Scar20 => 80.0,
            Ak47 => 36.0,
            M4A4 => 33.0,
            M4A1 => 38.0,
            Aug => 28.0,
            Sg556 => 30.0,
            Galilar => 30.0,
            Famas => 30.0,
            Deagle => 53.0,
            Revolver => 86.0,
            Usp | P2000 => 35.0,
            Glock => 30.0,
            P250 => 38.0,
            FiveSeven | Tec9 | Cz75A => 32.0,
            DualBerettas => 38.0,
            Mp9 | Mac10 => 26.0,
            Mp7 | Mp5Sd => 29.0,
            Ump45 => 35.0,
            P90 => 26.0,
            Bizon => 27.0,
            Negev => 35.0,
            M249 => 32.0,
            Mag7 | Nova | Sawedoff | Xm1014 => 180.0,
            Taser => 1000.0,
            _ => 30.0,
        }
    }

    pub fn armor_ratio(&self) -> f32 {
        use Weapon::*;
        match self {
            Awp => 1.95,
            Ssg08 => 1.70,
            G3SG1 | Scar20 => 1.65,
            Ak47 => 1.55,
            M4A4 | M4A1 => 1.40,
            Aug | Sg556 => 1.80,
            Galilar | Famas => 1.40,
            Deagle | Revolver => 1.70,
            FiveSeven | Tec9 | Cz75A => 1.55,
            P250 => 1.28,
            Usp | P2000 => 1.01,
            Glock => 0.94,
            DualBerettas => 1.15,
            Negev | M249 => 1.42,
            Ump45 => 1.30,
            P90 => 1.38,
            _ => 1.0,
        }
    }

    pub fn penetration_power(&self) -> f32 {
        use Weapon::*;
        match self {
            Awp | Ssg08 | G3SG1 | Scar20 => 2.5,
            Ak47 | M4A4 | M4A1 | Aug | Sg556 | Galilar | Famas | Deagle | Revolver | Negev | M249 => 2.0,
            _ => 1.0,
        }
    }

    pub fn headshot_multiplier(&self) -> f32 {
        if *self == Weapon::Taser {
            1.0
        } else {
            4.0
        }
    }

    pub fn base_inaccuracy(&self, is_scoped: bool) -> f32 {
        use Weapon::*;
        match self {
            Awp => if is_scoped { 0.0015 } else { 0.08 },
            Ssg08 => if is_scoped { 0.002 } else { 0.04 },
            G3SG1 | Scar20 => if is_scoped { 0.002 } else { 0.06 },
            Deagle | Revolver => 0.006,
            Ak47 | M4A4 | M4A1 | Aug | Sg556 => 0.0045,
            Usp | P2000 | P250 | FiveSeven | Glock => 0.007,
            Mp9 | Mac10 | Mp7 | Mp5Sd | Ump45 | P90 | Bizon => 0.010,
            Negev | M249 => 0.012,
            Mag7 | Nova | Sawedoff | Xm1014 => 0.030,
            _ => 0.008,
        }
    }
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Weapon::Unknown => "Unknown",
            Weapon::Knife => "Knife",
            Weapon::Cz75A => "CZ75-Auto",
            Weapon::Deagle => "Desert Eagle",
            Weapon::DualBerettas => "Dual Berettas",
            Weapon::FiveSeven => "Five-SeveN",
            Weapon::Glock => "Glock-18",
            Weapon::P2000 => "P2000",
            Weapon::P250 => "P250",
            Weapon::Revolver => "R8 Revolver",
            Weapon::Tec9 => "Tec-9",
            Weapon::Usp => "USP-S",
            Weapon::Bizon => "PP-Bizon",
            Weapon::Mac10 => "MAC-10",
            Weapon::Mp5Sd => "MP5-SD",
            Weapon::Mp7 => "MP7",
            Weapon::Mp9 => "MP9",
            Weapon::P90 => "P90",
            Weapon::Ump45 => "UMP-45",
            Weapon::M249 => "M249",
            Weapon::Negev => "Negev",
            Weapon::Mag7 => "MAG-7",
            Weapon::Nova => "Nova",
            Weapon::Sawedoff => "Sawed-Off",
            Weapon::Xm1014 => "XM1014",
            Weapon::Ak47 => "AK-47",
            Weapon::Aug => "AUG",
            Weapon::Famas => "FAMAS",
            Weapon::Galilar => "Galil AR",
            Weapon::M4A4 => "M4A4",
            Weapon::M4A1 => "M4A1-S",
            Weapon::Sg556 => "SG 553",
            Weapon::Awp => "AWP",
            Weapon::G3SG1 => "G3SG1",
            Weapon::Scar20 => "SCAR-20",
            Weapon::Ssg08 => "SSG 08",
            Weapon::Taser => "Zeus x27",
            Weapon::Flashbang => "Flashbang",
            Weapon::HeGrenade => "HE Grenade",
            Weapon::Smoke => "Smoke Grenade",
            Weapon::Molotov => "Molotov Cocktail",
            Weapon::Decoy => "Decoy Grenade",
            Weapon::Incendiary => "Incendiary Grenade",
            Weapon::C4 => "C4 Explosive",
        };
        write!(f, "{}", s)
    }
}
