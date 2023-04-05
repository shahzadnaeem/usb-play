use rusb::{devices, Device, DeviceDescriptor, DeviceHandle, Error, GlobalContext};
use std::time::Duration;

pub const LOGITECH: u16 = 0x046d; // Vendor
pub const G213: u16 = 0xc336; // Device

const ENDPOINT: u8 = 0x82; // Read Interrupt

const REQ_TYPE: u8 = 0x21;
const REQ: u8 = 0x09;
const VALUE: u16 = 0x0211;
const INDEX: u16 = 0x0001;
const CMD_LEN: usize = 20;
const TIMEOUT_MS: u64 = 200;

pub trait G213DeviceDescriptor {
    fn vendor_id(&self) -> u16;
    fn product_id(&self) -> u16;
}

impl G213DeviceDescriptor for DeviceDescriptor {
    fn vendor_id(&self) -> u16 {
        self.vendor_id()
    }

    fn product_id(&self) -> u16 {
        self.product_id()
    }
}

pub fn is_g213_keyboard(descriptor: &dyn G213DeviceDescriptor) -> bool {
    descriptor.vendor_id() == LOGITECH && descriptor.product_id() == G213
}

fn send_to_keyboard(
    handle: &DeviceHandle<GlobalContext>,
    bytes: &mut [u8],
) -> Result<usize, Error> {
    handle.write_control(
        REQ_TYPE,
        REQ,
        VALUE,
        INDEX,
        bytes,
        Duration::from_millis(TIMEOUT_MS),
    )?;

    handle.read_interrupt(ENDPOINT, bytes, Duration::from_millis(TIMEOUT_MS))
}

fn send_whole_keyboard_colour(handle: &DeviceHandle<GlobalContext>, colour: u32) {
    let command = format!("11ff0c3a0001{:06x}0200000000000000000000", colour);

    let mut bytes = [0u8; CMD_LEN];

    hex::decode_to_slice(command, &mut bytes).unwrap();

    let _bytes_sent = send_to_keyboard(handle, &mut bytes).unwrap();

    // println!("{} bytes sent", _bytes_sent);
}

fn send_breathe(handle: &DeviceHandle<GlobalContext>, speed: u16, colour: u32) {
    let command = format!("11ff0c3a0002{:06x}{:04x}006400000000000000", colour, speed);

    let mut bytes = [0u8; CMD_LEN];

    hex::decode_to_slice(command, &mut bytes).unwrap();

    let _bytes_sent = send_to_keyboard(handle, &mut bytes).unwrap();

    // println!("{} bytes sent", _bytes_sent);
}

fn send_cycle(handle: &DeviceHandle<GlobalContext>, speed: u16) {
    let command = format!("11ff0c3a0003ffffff0000{:04x}64000000000000", speed);

    let mut bytes = [0u8; CMD_LEN];

    hex::decode_to_slice(command, &mut bytes).unwrap();

    let _bytes_sent = send_to_keyboard(handle, &mut bytes).unwrap();

    // println!("{} bytes sent", _bytes_sent);
}

pub fn find_g213_keyboard() -> Option<Device<GlobalContext>> {
    devices().unwrap().iter().find(|device| {
        let desc = device.device_descriptor().unwrap();
        is_g213_keyboard(&desc)
    })
}

fn send_command_wrapper(
    device: Device<GlobalContext>,
    cmd_fn: impl Fn(&DeviceHandle<GlobalContext>),
) {
    let mut handle = device.open().expect("Unable to open device!");

    let mut kernel_driver_detached = false;

    if handle.kernel_driver_active(INDEX as u8).unwrap() {
        handle
            .detach_kernel_driver(INDEX as u8)
            .expect("Unable to detatch kernel USB driver");

        kernel_driver_detached = true;
    }

    cmd_fn(&handle);

    if kernel_driver_detached {
        handle
            .attach_kernel_driver(INDEX as u8)
            .expect("Unable to attach kernel USB driver");
    }
}

pub fn set_whole_keyboard_colour(device: Device<GlobalContext>, color: u32) {
    send_command_wrapper(device, |h| {
        send_whole_keyboard_colour(h, color);
    });
}

pub fn set_breathe(device: Device<GlobalContext>, speed: u16, color: u32) {
    send_command_wrapper(device, |h| {
        send_breathe(h, speed, color);
    });
}

pub fn set_cycle(device: Device<GlobalContext>, speed: u16) {
    send_command_wrapper(device, |h| {
        send_cycle(h, speed);
    });
}

#[cfg(test)]
mod g213_keyboard_tests {
    // use rusb::{ffi::libusb_device_descriptor, DeviceDescriptor};

    use super::*;

    // NOTE: A lot of work to test a one line function...

    struct GoodG213DeviceDescriptor {}
    struct NonLogitechDeviceDescriptor {}
    struct NonG213DeviceDescriptor {}

    impl G213DeviceDescriptor for GoodG213DeviceDescriptor {
        fn vendor_id(&self) -> u16 {
            0x046d
        }

        fn product_id(&self) -> u16 {
            0xc336
        }
    }

    impl G213DeviceDescriptor for NonLogitechDeviceDescriptor {
        fn vendor_id(&self) -> u16 {
            0x0400
        }

        fn product_id(&self) -> u16 {
            0xc336
        }
    }

    impl G213DeviceDescriptor for NonG213DeviceDescriptor {
        fn vendor_id(&self) -> u16 {
            0x046d
        }

        fn product_id(&self) -> u16 {
            0x1234
        }
    }

    #[test]
    fn a_g213_keyboard() {
        let descriptor = GoodG213DeviceDescriptor {};

        assert_eq!(is_g213_keyboard(&descriptor), true);
    }

    #[test]
    fn not_a_logitech_device() {
        let descriptor = NonLogitechDeviceDescriptor {};

        assert_eq!(is_g213_keyboard(&descriptor), false);
    }

    #[test]
    fn not_a_g213_keyboard() {
        let descriptor = NonG213DeviceDescriptor {};

        assert_eq!(is_g213_keyboard(&descriptor), false);
    }
}
