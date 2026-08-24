use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Bunnyhop {
    pub space_down: bool,
    pub was_in_air: bool,
    pub ground_ticks: u32,
}

impl CS2 {
    pub fn bunnyhop(&mut self, config: &Config, mouse: &mut Mouse) {
        if !config.misc.bunnyhop {
            if self.bhop.space_down {
                mouse.space_release();
                self.bhop.space_down = false;
                self.bhop.ground_ticks = 0;
            }
            return;
        }

        let hotkey_pressed = self.input.is_key_pressed(config.misc.bunnyhop_hotkey);
        if !hotkey_pressed {
            if self.bhop.space_down {
                mouse.space_release();
                self.bhop.space_down = false;
                self.bhop.ground_ticks = 0;
            }
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            if self.bhop.space_down {
                mouse.space_release();
                self.bhop.space_down = false;
                self.bhop.ground_ticks = 0;
            }
            return;
        };

        if local_player.health(self) <= 0 {
            if self.bhop.space_down {
                mouse.space_release();
                self.bhop.space_down = false;
                self.bhop.ground_ticks = 0;
            }
            return;
        }

        let in_air = local_player.is_in_air(self);

        if in_air {
            self.bhop.was_in_air = true;
            self.bhop.ground_ticks = 0;
            if self.bhop.space_down {
                mouse.space_release();
                self.bhop.space_down = false;
            }
        } else {
            if self.bhop.was_in_air {
                // Just landed from air! Send burst of scroll ticks to hit sub-tick window
                mouse.scroll_down_burst(2);
                mouse.space_press();
                self.bhop.space_down = true;
                self.bhop.was_in_air = false;
                self.bhop.ground_ticks = 0;
            } else {
                self.bhop.ground_ticks += 1;
                if self.bhop.ground_ticks <= 3 {
                    mouse.scroll_down();
                } else if self.bhop.ground_ticks >= 8 {
                    // Continuous ground reset
                    mouse.space_release();
                    mouse.scroll_down();
                    mouse.space_press();
                    self.bhop.ground_ticks = 0;
                }
            }
        }
    }
}
