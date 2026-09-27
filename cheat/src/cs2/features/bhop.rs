use std::time::{Duration, Instant};

use crate::{
    config::Config,
    constants::timing,
    cs2::{CS2, entity::player::Player, key_codes::KeyCode},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Bunnyhop {
    pub space_down: bool,
    pub was_in_air: bool,
    pub subtick_prejumped: bool,
    pub consecutive_hops: u32,
    pub hop_cooldown_until: Option<Instant>,
    next_ground_attempt: Option<Instant>,
}

impl Bunnyhop {
    fn reset(&mut self, mouse: &mut Mouse) {
        if std::mem::take(&mut self.space_down) {
            mouse.space_release();
        }
        self.was_in_air = false;
        self.subtick_prejumped = false;
        self.consecutive_hops = 0;
        self.hop_cooldown_until = None;
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
        self.consecutive_hops = self.consecutive_hops.saturating_add(1);
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
    fn predict_subtick_landing(&self, local_player: &Player) -> bool {
        let velocity = local_player.velocity(self);
        if velocity.z >= -60.0 {
            return false;
        }

        const TICK_INTERVAL: f32 = 1.0 / 64.0;
        const GRAVITY: f32 = 800.0;
        let fall_dist =
            velocity.z.abs() * TICK_INTERVAL + 0.5 * GRAVITY * TICK_INTERVAL * TICK_INTERVAL;

        if let Some(bvh) = &self.bvh {
            let origin = local_player.position(self);
            let future_pos = origin
                + glam::Vec3::new(velocity.x * TICK_INTERVAL, velocity.y * TICK_INTERVAL, 0.0);

            let mut min_clearance = f32::MAX;
            for &pt in &[origin, future_pos] {
                let start = pt + glam::Vec3::new(0.0, 0.0, 10.0);
                let end = pt - glam::Vec3::new(0.0, 0.0, 100.0);
                if let Some((dist, _, _)) = bvh.segment_intersections(start, end).first() {
                    let clearance = (*dist - 10.0).max(0.0);
                    if clearance < min_clearance {
                        min_clearance = clearance;
                    }
                }
            }

            if min_clearance < f32::MAX {
                return min_clearance <= fall_dist;
            }
        }

        false
    }

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

        let now = Instant::now();

        // If in legit cooldown, wait before allowing another hop.
        if let Some(cooldown) = self.bhop.hop_cooldown_until {
            if now < cooldown {
                if local_player.is_in_air(self) {
                    self.bhop.airborne(mouse);
                } else if self.bhop.space_down {
                    mouse.space_release();
                    self.bhop.space_down = false;
                }
                return;
            }
            self.bhop.hop_cooldown_until = None;
            self.bhop.consecutive_hops = 0;
        }

        let is_in_air = local_player.is_in_air(self);

        // Subtick mode landing prediction while in air:
        if is_in_air {
            if !self.bhop.subtick_prejumped {
                self.bhop.airborne(mouse);
            }

            if config.misc.bunnyhop_subtick
                && !self.bhop.subtick_prejumped
                && self.predict_subtick_landing(&local_player)
            {
                let allow_hop = if config.misc.bunnyhop_legit {
                    if self.bhop.consecutive_hops >= 4 {
                        self.bhop.hop_cooldown_until = Some(now + Duration::from_millis(300));
                        false
                    } else {
                        rand::random::<f32>() < 0.85
                    }
                } else {
                    true
                };

                if allow_hop {
                    let burst = if config.misc.bunnyhop_legit { 2 } else { 4 };
                    self.bhop.jump_attempt(
                        mouse,
                        now,
                        burst,
                        timing::BHOP_FULL_RETRY_INTERVAL,
                    );
                    self.bhop.subtick_prejumped = true;
                }
            }
            return;
        }

        // Player is on ground:
        let (landing_burst, ground_burst, retry_interval) = if config.misc.bunnyhop_legit {
            (2, 1, timing::BHOP_LEGIT_RETRY_INTERVAL)
        } else {
            (4, 2, timing::BHOP_FULL_RETRY_INTERVAL)
        };

        if self.bhop.subtick_prejumped {
            self.bhop.subtick_prejumped = false;
            self.bhop.was_in_air = false;
            self.bhop.next_ground_attempt = Some(now + retry_interval);
            return;
        }

        if self.bhop.was_in_air {
            let allow_hop = if config.misc.bunnyhop_legit {
                if self.bhop.consecutive_hops >= 4 {
                    self.bhop.hop_cooldown_until = Some(now + Duration::from_millis(300));
                    false
                } else {
                    rand::random::<f32>() < 0.85
                }
            } else {
                true
            };

            if allow_hop {
                self.bhop
                    .jump_attempt(mouse, now, landing_burst, retry_interval);
            } else {
                self.bhop.was_in_air = false;
                self.bhop.consecutive_hops = 0;
            }
        } else {
            let speed = local_player.velocity(self).length();
            if speed < 50.0 {
                self.bhop.consecutive_hops = 0;
            }
            if speed >= 50.0
                || local_player.is_jump_pressed(self)
                || self.input.is_key_pressed(KeyCode::Space)
            {
                self.bhop.grounded(mouse, now, ground_burst, retry_interval);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bhop_reset_clears_state_and_space() {
        let mut mouse = Mouse::dummy();
        let mut bhop = Bunnyhop {
            space_down: true,
            subtick_prejumped: true,
            consecutive_hops: 5,
            was_in_air: true,
            ..Default::default()
        };

        bhop.reset(&mut mouse);
        assert!(!bhop.space_down);
        assert!(!bhop.subtick_prejumped);
        assert!(!bhop.was_in_air);
        assert_eq!(bhop.consecutive_hops, 0);
    }

    #[test]
    fn bhop_jump_attempt_engages_space_and_counts_hops() {
        let mut mouse = Mouse::dummy();
        let mut bhop = Bunnyhop::default();
        let now = Instant::now();

        bhop.jump_attempt(&mut mouse, now, 2, Duration::from_millis(15));
        assert!(bhop.space_down);
        assert_eq!(bhop.consecutive_hops, 1);
        assert!(!bhop.was_in_air);

        bhop.jump_attempt(&mut mouse, now, 2, Duration::from_millis(15));
        assert!(bhop.space_down);
        assert_eq!(bhop.consecutive_hops, 2);
    }

    #[test]
    fn bhop_airborne_releases_space_when_not_subtick_prejumped() {
        let mut mouse = Mouse::dummy();
        let mut bhop = Bunnyhop {
            space_down: true,
            subtick_prejumped: false,
            ..Default::default()
        };

        bhop.airborne(&mut mouse);
        assert!(!bhop.space_down);
        assert!(bhop.was_in_air);
    }

    #[test]
    fn bhop_subtick_prejump_retention_flow() {
        let mut mouse = Mouse::dummy();
        let mut bhop = Bunnyhop::default();
        let now = Instant::now();

        // 1. In air, prejump triggers:
        bhop.jump_attempt(&mut mouse, now, 4, timing::BHOP_FULL_RETRY_INTERVAL);
        bhop.subtick_prejumped = true;
        assert!(bhop.space_down);
        assert!(bhop.subtick_prejumped);

        // 2. Subsequent airborne tick must NOT call airborne() if subtick_prejumped is true,
        // preserving space_down until landing.
        if !bhop.subtick_prejumped {
            bhop.airborne(&mut mouse);
        }
        assert!(bhop.space_down, "Space must stay pressed until landing tick!");

        // 3. Ground landing tick consumes prejump without calling duplicate jump_attempt:
        assert!(bhop.subtick_prejumped);
        bhop.subtick_prejumped = false;
        bhop.was_in_air = false;
        assert_eq!(bhop.consecutive_hops, 1);
        assert!(!bhop.subtick_prejumped);
    }

    #[test]
    fn in_jump_bitmask_works_for_custom_binds() {
        use crate::cs2::entity::player::IN_JUMP;
        // In-game buttons bitmask (m_pButtonStates[0]): bit 1 is set when ANY bound jump key is pressed (Space, x, etc.)
        let buttons_with_jump: u64 = IN_JUMP | (1 << 3); // jump + forward
        let buttons_without_jump: u64 = 1 << 3; // forward only

        assert_eq!(buttons_with_jump & IN_JUMP != 0, true);
        assert_eq!(buttons_without_jump & IN_JUMP != 0, false);
    }
}
