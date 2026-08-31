use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use utils::{Channel, Mutex};

use crate::{
    config::Config,
    constants::timing,
    cs2::CS2,
    data::{Data, RuntimeData},
    message::{GameMessage, GameStatus, UiMessage},
    os::mouse::Mouse,
};

pub struct GameManager {
    channel: Channel<UiMessage, GameMessage>,
    runtime_data: Arc<Mutex<RuntimeData>>,
    shared_config: Arc<Mutex<Arc<Config>>>,
    config: Config,
    mouse: Mouse,
    cs2: CS2,
    last_runtime_publish: Instant,
    last_frame_time_report: Instant,
    loop_iterations_since_report: u64,
    slowest_loop_since_report: Duration,
}

impl GameManager {
    pub fn new(
        channel: Channel<UiMessage, GameMessage>,
        runtime_data: Arc<Mutex<RuntimeData>>,
        shared_config: Arc<Mutex<Arc<Config>>>,
    ) -> Self {
        let mouse = match Mouse::open() {
            Ok(mouse) => mouse,
            Err(err) => {
                utils::error!("error creating uinput device: {err}");
                utils::error!("uinput kernel module is not loaded, or user is not in input group.");
                std::process::exit(1);
            }
        };

        Self {
            channel,
            runtime_data,
            shared_config,
            config: Config::default(),
            mouse,
            cs2: CS2::new(),
            last_runtime_publish: Instant::now(),
            last_frame_time_report: Instant::now(),
            loop_iterations_since_report: 0,
            slowest_loop_since_report: Duration::ZERO,
        }
    }

    fn send_message(&self, message: UiMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    pub fn run(&mut self) {
        self.send_message(UiMessage::Status(GameStatus::NotStarted));
        let mut previous_status = GameStatus::NotStarted;
        loop {
            let start = Instant::now();
            while let Ok(message) = self.channel.try_receive() {
                match message {
                    GameMessage::Config(config) => {
                        self.config = *config;
                        self.cs2.rebaseline_binds(&self.config);
                        *self.shared_config.lock() = Arc::new(self.config.clone());
                    }
                    GameMessage::BindCapture(capturing) => {
                        self.cs2.set_bind_capture(capturing);
                    }
                }
            }

            let mut is_valid = self.cs2.is_valid();
            if !is_valid {
                if previous_status == GameStatus::Working {
                    self.send_message(UiMessage::Status(GameStatus::NotStarted));
                    previous_status = GameStatus::NotStarted;
                }
                self.cs2.setup();
                is_valid = self.cs2.is_valid();
            }

            if is_valid {
                if previous_status == GameStatus::NotStarted {
                    self.send_message(UiMessage::Status(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.cs2.run(&self.config, &mut self.mouse);
                let now = Instant::now();
                if now.duration_since(self.last_runtime_publish) >= timing::RUNTIME_PUBLISH_INTERVAL
                {
                    self.cs2.runtime_data(&mut self.runtime_data.lock());
                    self.last_runtime_publish = now;
                }
            } else {
                *self.runtime_data.lock() = RuntimeData::default();
            }

            if is_valid {
                let elapsed = start.elapsed();
                self.loop_iterations_since_report += 1;
                self.slowest_loop_since_report = self.slowest_loop_since_report.max(elapsed);
                let report_at = Instant::now();
                let report_window = report_at.duration_since(self.last_frame_time_report);
                if report_window >= timing::FRAME_TIME_REPORT_INTERVAL {
                    if self.slowest_loop_since_report >= timing::SLOW_LOOP_WARNING {
                        utils::debug!(
                            "slowest hot-loop pass took {} ms (warning threshold {} ms)",
                            self.slowest_loop_since_report.as_millis(),
                            timing::SLOW_LOOP_WARNING.as_millis()
                        );
                    }
                    let average_period =
                        report_window.div_f64(self.loop_iterations_since_report as f64);
                    self.send_message(UiMessage::FrameTime(average_period));
                    self.last_frame_time_report = report_at;
                    self.loop_iterations_since_report = 0;
                    self.slowest_loop_since_report = Duration::ZERO;
                }
                std::hint::spin_loop();
            } else {
                sleep(timing::INVALID_PROCESS_RETRY_INTERVAL);
            }
        }
    }
}

pub struct EspManager {
    data: Arc<Mutex<Data>>,
    pending_data: Data,
    shared_config: Arc<Mutex<Arc<Config>>>,
    config: Arc<Config>,
    cs2: CS2,
}

impl EspManager {
    pub fn new(data: Arc<Mutex<Data>>, shared_config: Arc<Mutex<Arc<Config>>>) -> Self {
        Self {
            data,
            pending_data: Data::default(),
            shared_config,
            config: Arc::new(Config::default()),
            cs2: CS2::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let start = Instant::now();
            let config = self.shared_config.lock().clone();
            if !Arc::ptr_eq(&config, &self.config) {
                self.config = config;
                self.cs2.set_data_config(&self.config);
            }

            if !self.cs2.is_valid() {
                self.cs2.setup();
            }

            if self.cs2.is_valid() {
                self.cs2.refresh_data_entities();
                self.cs2.data(&mut self.pending_data);
                std::mem::swap(&mut *self.data.lock(), &mut self.pending_data);

                let elapsed = start.elapsed();
                if elapsed < timing::ESP_FRAME_INTERVAL {
                    sleep(timing::ESP_FRAME_INTERVAL - elapsed);
                }
            } else {
                self.pending_data = Data::default();
                std::mem::swap(&mut *self.data.lock(), &mut self.pending_data);
                sleep(timing::INVALID_PROCESS_RETRY_INTERVAL);
            }
        }
    }
}
