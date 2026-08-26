use std::{
    fs::File,
    io::Write,
    os::fd::AsRawFd,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use glam::Vec2;
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr, libc::c_ulong};

#[derive(Clone, Copy)]
struct Timeval {
    seconds: u64,
    microseconds: u64,
}

#[derive(Clone, Copy)]
struct InputEvent {
    time: Timeval,
    event_type: u16,
    code: u16,
    value: i32,
}

impl InputEvent {
    fn bytes(&self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes[0..8].copy_from_slice(&self.time.seconds.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.time.microseconds.to_le_bytes());
        bytes[16..18].copy_from_slice(&self.event_type.to_le_bytes());
        bytes[18..20].copy_from_slice(&self.code.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.value.to_le_bytes());

        bytes
    }
}

#[repr(C)]
struct DeviceSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

const DEVICE_SETUP: DeviceSetup = DeviceSetup {
    id: InputId {
        // usb
        bustype: 0x03,
        // texas instruments
        vendor: 0x0451,
        // ti-84 silver
        // yes, this is a calculator, sending mouse inputs
        product: 0xe008,
        version: 1,
    },
    // "TI-84 Plus Silver Calculator"
    name: [
        84, 73, 45, 56, 52, 32, 80, 108, 117, 115, 32, 83, 105, 108, 118, 101, 114, 32, 67, 97,
        108, 99, 117, 108, 97, 116, 111, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0,
    ],
    ff_effects_max: 0,
};

const UINPUT_IOCTL_BASE: c_ulong = b'U' as c_ulong;
ioctl_none!(ui_dev_create, UINPUT_IOCTL_BASE, 1);
ioctl_none!(ui_dev_destroy, UINPUT_IOCTL_BASE, 2);
ioctl_write_int!(ui_set_evbit, UINPUT_IOCTL_BASE, 100);
ioctl_write_int!(ui_set_keybit, UINPUT_IOCTL_BASE, 101);
ioctl_write_int!(ui_set_relbit, UINPUT_IOCTL_BASE, 102);
ioctl_write_ptr!(ui_dev_setup, UINPUT_IOCTL_BASE, 3, DeviceSetup);

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const SYN_REPORT: u16 = 0x00;
const AXIS_X: u16 = 0x00;
const AXIS_Y: u16 = 0x01;
const AXIS_WHEEL: u16 = 0x08;
const BTN_LEFT: u16 = 0x110;
const KEY_SPACE: u16 = 57;
const KEY_W: u16 = 17;
const KEY_A: u16 = 30;
const KEY_S: u16 = 31;
const KEY_D: u16 = 32;

pub struct Mouse {
    file: File,
    fractional: Vec2,
}

static CREATED: AtomicBool = AtomicBool::new(false);
impl Mouse {
    pub fn open() -> Result<Self, String> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err("mouse already initialized".into());
        }
        let file = File::options()
            .write(true)
            .open("/dev/uinput")
            .map_err(|e| e.to_string())?;
        let fd = file.as_raw_fd();

        unsafe {
            // enable event types
            ui_set_evbit(fd, EV_SYN as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_KEY as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_REL as u64).map_err(|e| e.to_string())?;

            ui_set_relbit(fd, AXIS_X as u64).map_err(|e| e.to_string())?;
            ui_set_relbit(fd, AXIS_Y as u64).map_err(|e| e.to_string())?;
            ui_set_relbit(fd, AXIS_WHEEL as u64).map_err(|e| e.to_string())?;

            ui_set_keybit(fd, BTN_LEFT as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_SPACE as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_W as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_A as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_S as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, KEY_D as u64).map_err(|e| e.to_string())?;

            ui_dev_setup(fd, &DEVICE_SETUP).map_err(|e| e.to_string())?;
            ui_dev_create(fd).map_err(|e| e.to_string())?;
        }

        Ok(Self {
            file,
            fractional: Vec2::ZERO,
        })
    }

    pub fn move_rel(&mut self, coords: Vec2) {
        for axis in 0..2 {
            if coords[axis] != 0.0 && coords[axis].signum() != self.fractional[axis].signum() {
                self.fractional[axis] = 0.0;
            }
        }

        let total = coords + self.fractional;
        let int_x = total.x.trunc() as i32;
        let int_y = total.y.trunc() as i32;
        self.fractional.x = total.x - int_x as f32;
        self.fractional.y = total.y - int_y as f32;

        if int_x == 0 && int_y == 0 {
            return;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let x = InputEvent {
            time,
            event_type: EV_REL,
            code: AXIS_X,
            value: int_x,
        };

        let y = InputEvent {
            time,
            event_type: EV_REL,
            code: AXIS_Y,
            value: int_y,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        if int_x != 0 {
            self.file.write_all(&x.bytes()).unwrap();
        }
        if int_y != 0 {
            self.file.write_all(&y.bytes()).unwrap();
        }
        self.file.write_all(&syn.bytes()).unwrap();
    }

    pub fn scroll_down_burst(&mut self, count: usize) {
        if count == 0 {
            return;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let wheel = InputEvent {
            time,
            event_type: EV_REL,
            code: AXIS_WHEEL,
            value: -(count.min(i32::MAX as usize) as i32),
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        let _ = self.file.write_all(&wheel.bytes());
        let _ = self.file.write_all(&syn.bytes());
    }

    pub fn left_press(&mut self) {
        self.key(BTN_LEFT, 1);
    }

    pub fn left_release(&mut self) {
        self.key(BTN_LEFT, 0);
    }

    pub fn space_press(&mut self) {
        self.key(KEY_SPACE, 1);
    }

    pub fn space_release(&mut self) {
        self.key(KEY_SPACE, 0);
    }

    pub fn counter_strafe(&mut self, forward_vel: f32, side_vel: f32) {
        if forward_vel > 20.0 {
            self.key(KEY_S, 1);
            self.key(KEY_S, 0);
        } else if forward_vel < -20.0 {
            self.key(KEY_W, 1);
            self.key(KEY_W, 0);
        }

        if side_vel > 20.0 {
            self.key(KEY_D, 1);
            self.key(KEY_D, 0);
        } else if side_vel < -20.0 {
            self.key(KEY_A, 1);
            self.key(KEY_A, 0);
        }
    }

    fn key(&mut self, code: u16, pressed: i32) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let press = InputEvent {
            time,
            event_type: EV_KEY,
            code,
            value: pressed,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        self.file.write_all(&press.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();
    }
}

impl Drop for Mouse {
    fn drop(&mut self) {
        let _ = unsafe { ui_dev_destroy(self.file.as_raw_fd()) };
        CREATED.store(false, Ordering::Relaxed);
    }
}

pub fn check_uinput() -> bool {
    let path = Path::new("/dev/uinput");
    if !path.exists() {
        utils::error!("the uinput kernel module is not loaded.");
        utils::error!("this module needs to be loaded for mouse input to work.");
        utils::error!("please carefully read the readme before using.");
        return false;
    }
    if File::options().write(true).open(path).is_err() {
        utils::error!("user has no write permissions for /dev/uinput.");
        utils::error!("did you run the setup script?");
        utils::error!("please carefully read the readme before using.");
        return false;
    }
    true
}
