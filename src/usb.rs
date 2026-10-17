//! This module contains all the code related to USB setup and handling.

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pipe::{self, Pipe},
};
use static_cell::StaticCell;

/// The type we get from initializing the pi's USB peripheral for CDC-ACM mode
type UsbSerialClass = embassy_usb::class::cdc_acm::CdcAcmClass<
    'static,
    embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>,
>;
/// The type we get from the embassy_usb device builder
type UsbDevice =
    embassy_usb::UsbDevice<'static, embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>>;

const MAX_USB_PACKET_SIZE: u16 = 64;

static USB_PIPE: pipe::Pipe<CriticalSectionRawMutex, 128> = Pipe::new();

/// Initialize the usb peripheral for use in CDC-ACM mode.
// The contents of this function are copied from https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/usb_serial.rs
pub fn init_usb_serial(
    spawner: Spawner,
    peripheral: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>,
    interrupts: crate::Interrupts,
) {
    let usb_driver = embassy_rp::usb::Driver::new(peripheral, interrupts);
    let usb_config = {
        let mut config = embassy_usb::Config::new(0xdead, 0xbeef); // VID and PID don't really matter here
        config.manufacturer = Some("DPI");
        config.product = Some("trifolium v1.4");
        config.max_power = 100; // Use the default power of 100 milliamps
        config.self_powered = true;
        config
    };
    let mut usb_builder = {
        // We use static cells here because the lifetimes of the buffers must match the lifetime of
        // the USB peripheral, which is 'static.
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            usb_driver,
            usb_config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // No msos descriptors (whatever those are)
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };
    let class: UsbSerialClass = {
        use embassy_usb::class::cdc_acm::{CdcAcmClass, State};

        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut usb_builder, state, MAX_USB_PACKET_SIZE) // Max packet size of 64
    };
    let usb: UsbDevice = usb_builder.build();

    spawner.spawn(usb_task(usb)).unwrap();
    spawner.spawn(usb_update_task(class)).unwrap();
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice) {
    // Run the usb peripheral forever.
    usb.run().await;
}

#[embassy_executor::task]
async fn usb_update_task(mut usb: UsbSerialClass) {
    let mut buf: [u8; MAX_USB_PACKET_SIZE as usize] = [0; MAX_USB_PACKET_SIZE as usize];
    loop {
        let num_bytes = USB_PIPE.read(&mut buf).await;
        usb.write_packet(&buf[..num_bytes]).await.unwrap();
    }
}

pub async fn print(bytes: &[u8]) {
    USB_PIPE.write(bytes).await;
}

/// Implementation taken from https://github.com/MarkGhebrial/CS122A-lecture-homework-week5-r/blob/d6cb467f0743c9019d089348c2608e1246be14f1/src/vid.c#L177-L189
pub async fn print_u32(n: u32) {
    // Figure out the place value of the most significant digit
    let mut place_value = 1;
    while n / (place_value * 10) != 0 {
        place_value += 10;
    }

    // Print the digits, starting from the most significant one.
    while place_value != 0 {
        print(&[b'0' + (n / place_value % 10) as u8]).await;
        place_value /= 10;
    }
}
