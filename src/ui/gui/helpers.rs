use std::hash::Hash;

use egui::{
    Color32, CornerRadius, DragValue, Event, Frame, Margin, Sense, Stroke, Ui, Widget,
};
use strum::IntoEnumIterator as _;

use crate::config::text::TextCategory;
use crate::cs2::{bones::Bones, key_codes::KeyCode};
use crate::ui::color::Colors;

/// Renders a modern CS cheat framed Groupbox / Card with header and clean inner padding.
pub fn groupbox(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(Colors::CARD_BG)
        .stroke(Stroke::new(1.0, Colors::BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let accent = ui.style().visuals.selection.bg_fill;
                // Accent indicator bar
                let (response, painter) = ui.allocate_painter(egui::vec2(3.0, 14.0), Sense::hover());
                painter.rect_filled(response.rect, CornerRadius::same(2), accent);

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(13.5)
                        .color(Colors::TEXT),
                );
            });
            ui.add_space(3.0);
            ui.separator();
            ui.add_space(4.0);

            add_body(ui);
        });
    ui.add_space(6.0);
}

pub fn bone_selector(ui: &mut Ui, bones: &mut Vec<Bones>) -> bool {
    let mut changed = false;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        for bone in Bones::iter() {
            let index = bones.iter().position(|selected| *selected == bone);
            let is_selected = index.is_some();
            let text = format!("{bone:?}");

            let accent = ui.style().visuals.selection.bg_fill;
            let (bg_color, text_color, border_color) = if is_selected {
                (accent, Colors::TEXT, accent)
            } else {
                (Colors::HIGHLIGHT, Colors::SUBTEXT, Colors::BORDER)
            };

            let response = ui.add(
                egui::Button::new(
                    egui::RichText::new(text)
                        .size(11.5)
                        .color(text_color),
                )
                .fill(bg_color)
                .stroke(Stroke::new(1.0, border_color))
                .corner_radius(CornerRadius::same(4))
                .min_size(egui::vec2(0.0, 20.0)),
            );

            if response.clicked() {
                if let Some(index) = index {
                    bones.remove(index);
                } else {
                    bones.push(bone);
                }
                changed = true;
            }
        }
    });

    changed
}

pub fn scroll(ui: &mut Ui, id: &str, add_content: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .id_salt(id)
        .show(ui, add_content);
}

pub fn checkbox(ui: &mut Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label).changed()
}

pub fn checkbox_hover(ui: &mut Ui, label: &str, hover_text: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label)
        .on_hover_text(hover_text)
        .changed()
}

pub fn drag(ui: &mut Ui, label: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(drag);
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub fn combo_box<T: std::fmt::Debug + strum::IntoEnumIterator + PartialEq>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut T,
) -> bool {
    let mut changed = false;
    egui::ComboBox::new(id, label)
        .selected_text(format!("{:?}", *value))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                let text = format!("{:?}", mode);
                if ui.selectable_value(value, mode, text).clicked() {
                    changed = true;
                }
            }
        });
    changed
}

pub fn color_picker(ui: &mut Ui, label: &str, color: &mut Color32) -> bool {
    let [mut r, mut g, mut b, mut a] = color.to_srgba_unmultiplied();
    let res = ui
        .horizontal(|ui| {
            let (response, painter) =
                ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
            painter.rect_filled(
                response.rect,
                ui.style().visuals.widgets.inactive.corner_radius,
                *color,
            );
            let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
            res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
            res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
            res = res.union(ui.add(DragValue::new(&mut a).prefix("a: ")));
            ui.label(label);
            res
        })
        .inner;

    let changed = res.changed();
    if changed {
        *color = Color32::from_rgba_premultiplied(r, g, b, a);
    }

    changed
}

pub fn text_settings_button(ui: &mut Ui, open_popup: &mut Option<String>, id: &str) {
    if ui.button("⚙").on_hover_text("Text settings").clicked() {
        *open_popup = Some(id.to_string());
    }
}

pub fn text_settings_popup(
    ui: &mut Ui,
    label: &str,
    category: &mut TextCategory,
    open_popup: &mut Option<String>,
    popup_id: &str,
) -> bool {
    let is_open = open_popup.as_deref() == Some(popup_id);
    if !is_open {
        return false;
    }

    let mut open = true;
    let mut changed = false;
    egui::Window::new(label)
        .id(egui::Id::new(popup_id))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            changed |= ui
                .horizontal(|ui| {
                    ui.label("Font Size");
                    ui.add(
                        egui::DragValue::new(&mut category.font_size)
                            .range(1.0..=99.0)
                            .speed(0.2)
                            .max_decimals(1),
                    )
                    .changed()
                })
                .inner;

            changed |= color_picker(ui, "Color", &mut category.color);

            ui.separator();

            changed |= combo_box(
                ui,
                &format!("{popup_id}_pos"),
                "Position",
                &mut category.position,
            );
            changed |= combo_box(
                ui,
                &format!("{popup_id}_align"),
                "Align",
                &mut category.align,
            );
        });

    if !open {
        *open_popup = None;
    }

    changed
}

pub fn keybind(ui: &mut Ui, id: &str, label: &str, keycode: &mut KeyCode) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(Keybind::new(keycode, id));
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub struct Keybind<'gui> {
    keycode: &'gui mut KeyCode,
    id: egui::Id,
}

impl<'gui> Keybind<'gui> {
    pub fn new(keycode: &'gui mut KeyCode, id: impl std::fmt::Debug + Hash) -> Self {
        Self {
            keycode,
            id: egui::Id::new(id),
        }
    }
}

impl<'gui> Widget for Keybind<'gui> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let listening_id = ui.make_persistent_id(self.id);

        let mut listening = {
            let ctx = ui.ctx();
            ctx.memory(|mem| mem.data.get_temp::<bool>(listening_id).unwrap_or(false))
        };

        let text = if listening {
            "[ ... ]".to_string()
        } else {
            format!("[ {:?} ]", self.keycode)
        };

        let accent = ui.style().visuals.selection.bg_fill;
        let (bg, text_color, border) = if listening {
            (Colors::HIGHLIGHT, accent, accent)
        } else {
            (Colors::HIGHLIGHT, Colors::TEXT, Colors::BORDER)
        };

        let mut response = ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .size(12.0)
                    .monospace()
                    .color(text_color),
            )
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .corner_radius(CornerRadius::same(4)),
        );

        if response.clicked() {
            listening = !listening;
        }

        if response.secondary_clicked() {
            listening = false;
        }

        if listening {
            let input = ui.input(|i| {
                for event in &i.events {
                    if let Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if *key == egui::Key::F35 {
                            return KeyCode::from_egui_modifiers(*modifiers);
                        } else {
                            return KeyCode::from_egui(*key);
                        }
                    }

                    if let Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } = event
                    {
                        return Some(KeyCode::from_egui_mouse(*button));
                    }
                }
                None
            });

            if let Some(input) = input {
                if input == KeyCode::Escape {
                    *self.keycode = KeyCode::None;
                    response.mark_changed();
                } else {
                    *self.keycode = input;
                    response.mark_changed();
                }
                listening = false;
            }
        }

        let ctx = ui.ctx();
        ctx.memory_mut(|mem| mem.data.insert_temp(listening_id, listening));

        response
    }
}

pub fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
