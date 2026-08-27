use egui::{Align, Ui};

use crate::{
    config::{
        aim::WeaponConfig,
        bind::{BindMode, KeyChord, SettingBind, SettingId},
        write_config,
    },
    data::Data,
    message::{GameMessage, GameStatus},
    ui::{
        app::{App, AppState},
        color::Colors,
        gui::{
            aimbot::AimbotTab,
            helpers::{bool_setting_row, key_chord, open_url, text_settings_popup},
        },
        window_context::WindowContext,
    },
    update::UpdateStatus,
};

pub mod aimbot;
mod application;
mod config;
mod grenade;
mod helpers;
mod hud;
mod player;
mod r#unsafe;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Aimbot,
    Player,
    Hud,
    Grenades,
    Unsafe,
    Config,
    Application,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FeatureSettingsPopup {
    Player,
    OofArrows,
    SoundEsp,
    Hud,
    SniperCrosshair,
    GrenadeTrails,
    Bunnyhop,
    NoFlash,
    Smokes,
    FovChanger,
}

impl AppState {
    pub fn send_config(&self) {
        self.send_message(GameMessage::Config(Box::new(self.config.clone())));
        self.save();
    }

    pub fn send_message(&self, message: GameMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    pub fn bool_setting(&mut self, ui: &mut Ui, label: &str, id: SettingId) -> bool {
        self.bool_setting_hover(ui, label, None, id)
    }

    pub fn bool_setting_hover(
        &mut self,
        ui: &mut Ui,
        label: &str,
        hover_text: Option<&str>,
        id: SettingId,
    ) -> bool {
        let mut value = self.config.bool_value(&id);
        let has_bind = self.config.binds.iter().any(|binding| binding.target == id);
        let active = self
            .render_data
            .bound_values
            .get(&id)
            .copied()
            .unwrap_or(value);
        let response = bool_setting_row(ui, label, &mut value, has_bind, active);
        if let Some(text) = hover_text {
            response.response.clone().on_hover_text(text);
        }
        if response.open_bind {
            if self.bind_popup.is_none() {
                self.send_message(GameMessage::BindCapture(true));
            }
            self.bind_popup = Some(id.clone());
        }
        if response.changed {
            self.config.set_bool(&id, value);
        }
        response.changed
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn render_bind_popup(&mut self, ui: &mut Ui) {
        let Some(target) = self.bind_popup.clone() else {
            return;
        };
        let mut open = true;
        let mut changed = false;
        let mut close = false;
        egui::Window::new("Keybinds")
            .id(egui::Id::new("setting_bind_editor"))
            .collapsible(false)
            .resizable(false)
            .default_width(330.0)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.label(egui::RichText::new(format!("{target:?}")).weak());
                ui.label("Right-click a setting row to edit its binds.");
                ui.separator();

                let binding_index = self
                    .config
                    .binds
                    .iter()
                    .position(|binding| binding.target == target);
                let Some(binding_index) = binding_index else {
                    if ui.button("+ Add bind").clicked() {
                        self.config.binds.push(SettingBind {
                            target: target.clone(),
                            mode: BindMode::Toggle,
                            chords: vec![KeyChord::default()],
                        });
                        changed = true;
                    }
                    return;
                };

                let binding = &mut self.config.binds[binding_index];
                ui.horizontal(|ui| {
                    ui.label("Default mode");
                    changed |= ui
                        .selectable_value(&mut binding.mode, BindMode::Toggle, "Toggle")
                        .clicked();
                    changed |= ui
                        .selectable_value(&mut binding.mode, BindMode::Hold, "Hold")
                        .clicked();
                });
                ui.add_space(4.0);

                let mut remove = None;
                for (index, chord) in binding.chords.iter_mut().enumerate() {
                    egui::Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                changed |= ui.checkbox(&mut chord.enabled, "").changed();
                                changed |= key_chord(ui, ("setting_chord", &target, index), chord);
                                egui::ComboBox::from_id_salt(("chord_mode", &target, index))
                                    .selected_text(match chord.mode {
                                        Some(mode) => format!("{mode:?}"),
                                        None => format!("Default ({:?})", binding.mode),
                                    })
                                    .show_ui(ui, |ui| {
                                        changed |= ui
                                            .selectable_value(&mut chord.mode, None, "Default")
                                            .clicked();
                                        changed |= ui
                                            .selectable_value(
                                                &mut chord.mode,
                                                Some(BindMode::Toggle),
                                                "Toggle",
                                            )
                                            .clicked();
                                        changed |= ui
                                            .selectable_value(
                                                &mut chord.mode,
                                                Some(BindMode::Hold),
                                                "Hold",
                                            )
                                            .clicked();
                                    });
                                if ui.small_button("×").on_hover_text("Delete bind").clicked() {
                                    remove = Some(index);
                                }
                            });
                        });
                    ui.add_space(3.0);
                }
                if let Some(index) = remove {
                    binding.chords.remove(index);
                    changed = true;
                }

                let mut remove_binding = false;
                ui.horizontal(|ui| {
                    if ui.button("+ New chord").clicked() {
                        binding.chords.push(KeyChord::default());
                        changed = true;
                    }
                    if ui.button("Remove all").clicked() {
                        remove_binding = true;
                    }
                });
                if remove_binding {
                    self.config.binds.remove(binding_index);
                    changed = true;
                    close = true;
                }
            });

        if changed {
            self.send_config();
        }
        if !open || close {
            self.bind_popup = None;
            self.send_message(GameMessage::BindCapture(false));
        }
    }

    fn gui(&mut self, ui: &mut Ui) {
        ui.ctx().set_pixels_per_point(self.display_scale);
        egui::Panel::left("sidebar")
            .resizable(false)
            .show(ui, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "Config");
                ui.selectable_value(&mut self.current_tab, Tab::Application, "Application");

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    ui.label(concat!("v", env!("CARGO_PKG_VERSION")));

                    if ui.button("Report Issue").clicked() {
                        open_url("https://github.com/avitran0/deadlocked/issues");
                    }

                    ui.label(egui::RichText::new(format!("{}", self.game_status)).color(
                        match self.game_status {
                            GameStatus::Working => Colors::GREEN,
                            GameStatus::NotStarted => Colors::YELLOW,
                        },
                    ));

                    ui.label(format!("{:.1} ms", self.frame_avg_ms()));
                });
            });

        egui::CentralPanel::default().show(ui, |ui| match self.current_tab {
            Tab::Aimbot => self.aimbot_settings(ui),
            Tab::Player => self.player_settings(ui),
            Tab::Hud => self.hud_settings(ui),
            Tab::Grenades => self.grenade_settings(ui),
            Tab::Unsafe => self.unsafe_settings(ui),
            Tab::Config => self.config_settings(ui),
            Tab::Application => self.application_settings(ui),
        });

        self.render_text_popups(ui);
        self.render_bind_popup(ui);

        if self.update_popup {
            let mut close = false;
            egui::Window::new("Update Available")
                .id(egui::Id::new("update_popup"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    if let UpdateStatus::Available { version, url } = &self.update_status {
                        ui.label(
                            egui::RichText::new(format!("Update {version} available!"))
                                .color(Colors::YELLOW)
                                .size(18.0),
                        );
                        ui.separator();
                        ui.label("A new version of deadlocked is ready to download.");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Download").clicked() {
                                open_url(url);
                            }
                            if ui.button("Dismiss").clicked() {
                                close = true;
                            }
                        });
                    }
                });
            if close {
                self.update_popup = false;
            }
        }
    }

    fn weapon_config(&mut self) -> &mut WeaponConfig {
        if self.aimbot_tab == AimbotTab::Weapon {
            self.config
                .aim
                .weapons
                .get_mut(&self.aimbot_weapon)
                .unwrap()
        } else {
            &mut self.config.aim.global
        }
    }

    fn render_text_popups(&mut self, ui: &mut Ui) {
        let text = &mut self.config.hud.overlay_text;
        let mut changed = false;
        changed |= text_settings_popup(
            ui,
            "Status Text",
            &mut text.status_text,
            &mut self.text_popup,
            "status_text",
        );
        changed |= text_settings_popup(
            ui,
            "Player Name",
            &mut text.player_name,
            &mut self.text_popup,
            "player_name",
        );
        changed |= text_settings_popup(
            ui,
            "Player Tags",
            &mut text.player_tags,
            &mut self.text_popup,
            "player_tags",
        );
        changed |= text_settings_popup(
            ui,
            "Weapon Icon",
            &mut text.weapon_icon,
            &mut self.text_popup,
            "weapon_icon",
        );
        changed |= text_settings_popup(
            ui,
            "Ammo",
            &mut text.ammo_text,
            &mut self.text_popup,
            "ammo_text",
        );
        changed |= text_settings_popup(
            ui,
            "Weapon Name",
            &mut text.weapon_name,
            &mut self.text_popup,
            "weapon_name",
        );
        changed |= text_settings_popup(
            ui,
            "Bomb Timer",
            &mut text.bomb_timer,
            &mut self.text_popup,
            "bomb_timer",
        );
        changed |= text_settings_popup(
            ui,
            "Grenade Name",
            &mut text.grenade_name,
            &mut self.text_popup,
            "grenade_name",
        );
        changed |= text_settings_popup(
            ui,
            "Grenade Lineup",
            &mut text.grenade_lineup,
            &mut self.text_popup,
            "grenade_lineup",
        );
        changed |= text_settings_popup(
            ui,
            "Keybind List",
            &mut text.keybind_list,
            &mut self.text_popup,
            "keybind_list",
        );
        changed |= text_settings_popup(
            ui,
            "Spectator List",
            &mut text.spectator_list,
            &mut self.text_popup,
            "spectator_list",
        );
        if changed {
            self.send_config();
        }
    }
}

impl App {
    pub fn render(&mut self) {
        let gui = self.gui.as_mut().unwrap();
        let overlay = self.overlay.as_mut().unwrap();
        let state = &mut self.state;

        state.render_data.clone_from(&state.data.lock());
        state.render_data.apply_runtime(&state.runtime_data.lock());

        if let Err(err) = gui.make_current() {
            utils::error!("could not make gui window current: {err}");
            return;
        }
        gui.run(|ui| state.gui(ui));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            utils::error!("could not swap gui window buffers: {err}");
            return;
        }

        overlay.window().set_cursor_hittest(false).unwrap();
        Self::update_overlay_window(overlay, &state.render_data);
        if let Err(err) = overlay.make_current() {
            utils::error!("could not make overlay window current: {err}");
            return;
        }

        // Rendering consumes the same effective bool values as the game thread while
        // preserving the user's saved/base config.
        let bound_overrides = state
            .render_data
            .bound_values
            .iter()
            .map(|(target, value)| (target.clone(), state.config.bool_value(target), *value))
            .collect::<Vec<_>>();
        for (target, _, value) in &bound_overrides {
            state.config.set_bool(target, *value);
        }
        let render_data = std::mem::take(&mut state.render_data);
        overlay.run(|ui| state.overlay(ui, &render_data));
        state.render_data = render_data;
        for (target, saved_value, _) in &bound_overrides {
            state.config.set_bool(target, *saved_value);
        }
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            utils::error!("could not swap overlay window buffers: {err}");
        }
    }

    fn update_overlay_window(overlay: &WindowContext, data: &Data) {
        use winit::dpi::PhysicalPosition;
        let position =
            PhysicalPosition::new(data.window_position.x as i32, data.window_position.y as i32);
        if !match overlay.window().outer_position() {
            Ok(pos) => pos == position,
            Err(_) => false,
        } {
            overlay.window().set_outer_position(position);
        }

        let size = winit::dpi::PhysicalSize::new(
            data.window_size.x.max(1.0) as u32,
            data.window_size.y.max(1.0) as u32,
        );
        if overlay.window().inner_size() != size {
            let _ = overlay.window().request_inner_size(size);
        }
    }
}
