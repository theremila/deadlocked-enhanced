#![allow(unused)]
use egui::Color32;
use serde::{Deserialize, Serialize};

pub struct Colors;

impl Colors {
    pub const BACKDROP: Color32 = Color32::from_rgb(16, 16, 18);
    pub const BASE: Color32 = Color32::from_rgb(23, 23, 26);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(36, 36, 40);
    pub const SUBTEXT: Color32 = Color32::from_rgb(145, 145, 151);
    pub const TEXT: Color32 = Color32::from_rgb(232, 232, 235);
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(160, 240, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(100, 150, 240);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);
    pub const PINK: Color32 = Color32::from_rgb(222, 151, 178);

    pub const ACCENT_COLORS: [(&str, Color32); 8] = [
        ("Pink", Self::PINK),
        ("Red", Self::RED),
        ("Orange", Self::ORANGE),
        ("Yellow", Self::YELLOW),
        ("Green", Self::GREEN),
        ("Teal", Self::TEAL),
        ("Blue", Self::BLUE),
        ("Purple", Self::PURPLE),
    ];
}
