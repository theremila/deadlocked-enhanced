use crate::{
    config::{Config, r#unsafe::BunnyhopMode},
    cs2::{CS2, entity::player::Player},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Bunnyhop {
    pub space_down: bool,
    pub was_in_air: bool,
    pub ground_ticks: u32,
}

impl Bunnyhop {
    fn reset(&mut self, mouse: &mut Mouse) {
        if std::mem::take(&mut self.space_down) {
            mouse.space_release();
        }
        self.was_in_air = false;
        self.ground_ticks = 0;
    }

    fn airborne(&mut self, mouse: &mut Mouse) {
        self.was_in_air = true;
        self.ground_ticks = 0;
        if std::mem::take(&mut self.space_down) {
            mouse.space_release();
        }
    }

    fn landed(&mut self, mouse: &mut Mouse, scroll_ticks: usize) {
        mouse.scroll_down_burst(scroll_ticks);
        mouse.space_press();
        self.space_down = true;
        self.was_in_air = false;
        self.ground_ticks = 0;
    }

    fn grounded(
        &mut self,
        mouse: &mut Mouse,
        initial_scroll_ticks: u32,
        reset_tick: u32,
        scroll_burst: usize,
    ) {
        self.ground_ticks = self.ground_ticks.saturating_add(1);
        if self.ground_ticks <= initial_scroll_ticks {
            mouse.scroll_down_burst(scroll_burst);
        } else if self.ground_ticks >= reset_tick {
            if self.space_down {
                mouse.space_release();
            }
            mouse.scroll_down_burst(scroll_burst);
            mouse.space_press();
            self.space_down = true;
            self.ground_ticks = 0;
        }
    }
}

impl CS2 {
    pub fn bunnyhop(&mut self, config: &Config, mouse: &mut Mouse) {
        if !config.misc.bunnyhop || !self.input.is_key_pressed(config.misc.bunnyhop_hotkey) {
            self.bhop.reset(mouse);
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.bhop.reset(mouse);
            return;
        };

        if local_player.health(self) <= 0 {
            self.bhop.reset(mouse);
            return;
        }

        if local_player.is_in_air(self) {
            self.bhop.airborne(mouse);
            return;
        }

        let (landing_burst, initial_ticks, reset_tick, ground_burst) =
            match config.misc.bunnyhop_mode {
                BunnyhopMode::Full => (4, 2, 6, 2),
                BunnyhopMode::Legit => (2, 3, 8, 1),
            };

        if self.bhop.was_in_air {
            self.bhop.landed(mouse, landing_burst);
        } else {
            self.bhop
                .grounded(mouse, initial_ticks, reset_tick, ground_burst);
        }
    }
}
