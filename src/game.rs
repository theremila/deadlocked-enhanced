use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use utils::{Channel, Mutex};

use crate::{
    config::Config,
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
                self.cs2.runtime_data(&mut self.runtime_data.lock());
            } else {
                *self.runtime_data.lock() = RuntimeData::default();
            }

            if is_valid {
                let elapsed = start.elapsed();
                if elapsed < self.loop_duration() {
                    sleep(self.loop_duration() - elapsed);
                } else {
                    utils::debug!(
                        "game loop took {} ms (max {} ms)",
                        elapsed.as_millis(),
                        self.loop_duration().as_millis()
                    );
                }
                self.send_message(UiMessage::FrameTime(elapsed));
            } else {
                sleep(Duration::from_secs(5));
            }
        }
    }

    fn loop_duration(&self) -> Duration {
        Duration::from_millis(2)
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
    const FRAME_DURATION: Duration = Duration::from_millis(8);

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
                if elapsed < Self::FRAME_DURATION {
                    sleep(Self::FRAME_DURATION - elapsed);
                }
            } else {
                self.pending_data = Data::default();
                std::mem::swap(&mut *self.data.lock(), &mut self.pending_data);
                sleep(Duration::from_secs(5));
            }
        }
    }
}
