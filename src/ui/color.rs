#![allow(unused)]
use egui::Color32;
use serde::{Deserialize, Serialize};

pub struct Colors;

impl Colors {
    pub const BACKDROP: Color32 = Color32::from_rgb(14, 14, 20);
    pub const BASE: Color32 = Color32::from_rgb(18, 18, 26);
    pub const CARD_BG: Color32 = Color32::from_rgb(22, 22, 32);
    pub const CARD_HEADER: Color32 = Color32::from_rgb(28, 28, 40);
    pub const BORDER: Color32 = Color32::from_rgb(38, 38, 54);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(52, 52, 74);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(30, 30, 44);
    pub const HOVER: Color32 = Color32::from_rgb(40, 40, 58);
    pub const SUBTEXT: Color32 = Color32::from_rgb(150, 150, 175);
    pub const MUTED: Color32 = Color32::from_rgb(105, 105, 130);
    pub const TEXT: Color32 = Color32::from_rgb(245, 245, 255);
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(160, 240, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(100, 150, 240);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);

    pub const ACCENT_COLORS: [(&str, Color32); 7] = [
        ("Red", Self::RED),
        ("Orange", Self::ORANGE),
        ("Yellow", Self::YELLOW),
        ("Green", Self::GREEN),
        ("Teal", Self::TEAL),
        ("Blue", Self::BLUE),
        ("Purple", Self::PURPLE),
    ];
}
