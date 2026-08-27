use std::hash::Hash;

use egui::{CollapsingHeader, Color32, DragValue, Event, Sense, Ui};
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
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
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

/// Native checkbox with a bind indicator and a right-click bind action.
pub fn bool_setting_row(
    ui: &mut Ui,
    label: &str,
    value: &mut bool,
    has_bind: bool,
    active: bool,
) -> BoolSettingResponse {
    let response = ui
        .horizontal(|ui| {
            let response = ui.checkbox(value, label);
            if has_bind {
                ui.label(egui::RichText::new("B").small().color(if active {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().weak_text_color()
                }))
                .on_hover_text("Bound — right-click the checkbox to edit");
            }
            response
        })
        .inner;

    BoolSettingResponse {
        changed: response.changed(),
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
