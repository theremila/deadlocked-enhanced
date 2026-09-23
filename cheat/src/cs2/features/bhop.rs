use std::time::{Duration, Instant};

use crate::{
    config::{Config, r#unsafe::BunnyhopMode},
    constants::timing,
    cs2::{CS2, entity::player::Player},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Bunnyhop {
    pub space_down: bool,
    pub was_in_air: bool,
    next_ground_attempt: Option<Instant>,
}

impl Bunnyhop {
    fn reset(&mut self, mouse: &mut Mouse) {
        if std::mem::take(&mut self.space_down) {
            mouse.space_release();
        }
        self.was_in_air = false;
        self.next_ground_attempt = None;
    }

    fn airborne(&mut self, mouse: &mut Mouse) {
        self.was_in_air = true;
        self.next_ground_attempt = None;
        if std::mem::take(&mut self.space_down) {
            mouse.space_release();
        }
    }

    fn jump_attempt(
        &mut self,
        mouse: &mut Mouse,
        now: Instant,
        scroll_ticks: usize,
        retry_interval: Duration,
    ) {
        if self.space_down {
            mouse.space_release();
        }
        mouse.scroll_down_burst(scroll_ticks);
        mouse.space_press();
        self.space_down = true;
        self.was_in_air = false;
        self.next_ground_attempt = Some(now + retry_interval);
    }

    fn grounded(
        &mut self,
        mouse: &mut Mouse,
        now: Instant,
        scroll_burst: usize,
        retry_interval: Duration,
    ) {
        if self
            .next_ground_attempt
            .is_none_or(|deadline| now >= deadline)
        {
            self.jump_attempt(mouse, now, scroll_burst, retry_interval);
        }
    }
}

impl CS2 {
    pub fn bunnyhop(&mut self, config: &Config, mouse: &mut Mouse) {
        if !config.misc.bunnyhop {
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

        let (landing_burst, ground_burst, retry_interval) = match config.misc.bunnyhop_mode {
            BunnyhopMode::Full => (4, 2, timing::BHOP_FULL_RETRY_INTERVAL),
            BunnyhopMode::Legit => (2, 1, timing::BHOP_LEGIT_RETRY_INTERVAL),
        };
        let now = Instant::now();

        if self.bhop.was_in_air {
            self.bhop
                .jump_attempt(mouse, now, landing_burst, retry_interval);
        } else {
            self.bhop.grounded(mouse, now, ground_burst, retry_interval);
        }
    }
}
