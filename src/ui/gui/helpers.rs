use std::hash::Hash;

use egui::{Color32, DragValue, Event, Sense, Ui};
use strum::IntoEnumIterator as _;

use crate::config::{bind::KeyChord, text::TextCategory};
use crate::cs2::{bones::Bones, key_codes::KeyCode};

pub fn bone_selector(ui: &mut Ui, bones: &mut Vec<Bones>) -> bool {
    let mut changed = false;

    for bone in Bones::iter() {
        let index = bones.iter().position(|selected| *selected == bone);
        if ui
            .selectable_label(index.is_some(), format!("{bone:?}"))
            .clicked()
        {
            if let Some(index) = index {
                bones.remove(index);
            } else {
                bones.push(bone);
            }
            changed = true;
        }
    }

    changed
}

pub fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(5.0)
        .inner_margin(egui::Margin::same(9))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).strong().size(14.0));
            let rect = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [rect.left_top(), egui::pos2(rect.right(), rect.top())],
                egui::Stroke::new(1.0, ui.visuals().selection.bg_fill),
            );
            ui.add_space(4.0);
            add_body(ui);
        });
}

pub fn scroll(ui: &mut Ui, id: &str, add_content: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .id_salt(id)
        .show(ui, add_content);
}

pub struct BoolSettingResponse {
    pub changed: bool,
    pub open_bind: bool,
    pub response: egui::Response,
}

/// Flat setting row shared by every bindable boolean setting.
pub fn bool_setting_row(
    ui: &mut Ui,
    label: &str,
    value: &mut bool,
    has_bind: bool,
    active: bool,
) -> BoolSettingResponse {
    let height = 26.0;
    let desired = egui::vec2(ui.available_width().max(120.0), height);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
    let visuals = ui.style().interact(&response);
    let text_color = if ui.is_enabled() {
        visuals.text_color()
    } else {
        ui.visuals().weak_text_color()
    };

    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(3),
            ui.visuals().widgets.hovered.weak_bg_fill,
        );
    }
    ui.painter().text(
        egui::pos2(rect.left() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::TextStyle::Body.resolve(ui.style()),
        text_color,
    );

    let switch_rect = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 20.0, rect.center().y),
        egui::vec2(28.0, 14.0),
    );
    let switch_color = if *value {
        ui.visuals().selection.bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    ui.painter()
        .rect_filled(switch_rect, egui::CornerRadius::same(7), switch_color);
    let knob_x = if *value {
        switch_rect.right() - 7.0
    } else {
        switch_rect.left() + 7.0
    };
    ui.painter().circle_filled(
        egui::pos2(knob_x, switch_rect.center().y),
        5.0,
        Color32::WHITE,
    );

    if has_bind {
        let color = if active {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().weak_text_color()
        };
        ui.painter().text(
            egui::pos2(switch_rect.left() - 9.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "B",
            egui::TextStyle::Small.resolve(ui.style()),
            color,
        );
    }

    let changed = response.clicked_by(egui::PointerButton::Primary) && ui.is_enabled();
    if changed {
        *value = !*value;
        response.mark_changed();
    }

    BoolSettingResponse {
        changed,
        open_bind: response.clicked_by(egui::PointerButton::Secondary),
        response,
    }
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

pub fn drag_hover(ui: &mut Ui, label: &str, hover_text: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let changed = ui.add(drag).on_hover_text(hover_text).changed();
        ui.label(label).on_hover_text(hover_text);
        changed
    })
    .inner
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

pub fn key_chord(ui: &mut Ui, id: impl std::fmt::Debug + Hash, chord: &mut KeyChord) -> bool {
    let listening_id = ui.make_persistent_id(egui::Id::new(id));
    let was_listening = ui
        .ctx()
        .memory(|memory| memory.data.get_temp::<bool>(listening_id).unwrap_or(false));
    let text = if was_listening {
        "press a chord…".to_owned()
    } else if chord.keys.is_empty() {
        "unbound".to_owned()
    } else {
        chord
            .keys
            .iter()
            .map(|key| format!("{key:?}"))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let response = ui.button(text);
    let mut listening = if response.clicked() {
        !was_listening
    } else {
        was_listening
    };
    let mut changed = false;

    if was_listening && !response.clicked() {
        let captured = ui.input(|input| {
            input.events.iter().find_map(|event| match event {
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if *key != egui::Key::F35 => {
                    let key = KeyCode::from_egui(*key)?;
                    Some((key, *modifiers))
                }
                Event::PointerButton {
                    button,
                    pressed: true,
                    ..
                } => Some((KeyCode::from_egui_mouse(*button), input.modifiers)),
                _ => None,
            })
        });

        if let Some((key, modifiers)) = captured {
            if key == KeyCode::Escape {
                listening = false;
            } else if matches!(key, KeyCode::Delete | KeyCode::Backspace) {
                chord.keys.clear();
                listening = false;
                changed = true;
            } else {
                let mut keys = Vec::with_capacity(4);
                if modifiers.ctrl {
                    keys.push(KeyCode::LeftControl);
                }
                if modifiers.shift {
                    keys.push(KeyCode::LeftShift);
                }
                if modifiers.alt {
                    keys.push(KeyCode::LeftAlt);
                }
                keys.push(key);
                chord.keys = keys;
                chord.canonicalize();
                listening = false;
                changed = true;
            }
        }
    }

    ui.ctx()
        .memory_mut(|memory| memory.data.insert_temp(listening_id, listening));
    changed
}

pub fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
