#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embedded_graphics::{Drawable, text::Text};
use ssd1306::mode::DisplayConfigAsync;

use panic_probe as _;
use static_cell::StaticCell;

// Set up the device interrupts
embassy_rp::bind_interrupts!(struct Interrupts {
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    /**** INITIALIZE USB SERIAL *****/
    // Copied from https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/usb_serial.rs
    let usb_driver = embassy_rp::usb::Driver::new(p.USB, Interrupts);
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
    let class = {
        use embassy_usb::class::cdc_acm::{CdcAcmClass, State};

        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut usb_builder, state, 64)
    };
    let usb = usb_builder.build();
    spawner.spawn(usb_task(usb)).unwrap();
    spawner.spawn(print_task(class)).unwrap();

    /**** INITIALIZE DISPLAY ****/
    let i2c =
        embassy_rp::i2c::I2c::new_async(p.I2C1, p.PIN_15, p.PIN_14, Interrupts, Default::default());
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = ssd1306::Ssd1306Async::new(
        interface,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();
    display.init().await.unwrap();

    // Set the font size to 6 by 10 pixels
    let text_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(embedded_graphics::pixelcolor::BinaryColor::On)
        .build();

    Text::with_baseline(
        "Hello world!",
        embedded_graphics::geometry::Point::zero(),
        text_style,
        embedded_graphics::text::Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "Hello Rust!",
        embedded_graphics::geometry::Point::new(0, 16),
        text_style,
        embedded_graphics::text::Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().await.unwrap();

    // Make the LED blink
    spawner.spawn(blink(p.PIN_13.into())).unwrap();
}

#[embassy_executor::task]
async fn usb_task(
    mut usb: embassy_usb::UsbDevice<
        'static,
        embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>,
    >,
) {
    usb.run().await;
}

// struct USB {
//     inner: UsbClass,
// }
// impl embedded_io_async::ErrorType for USB {
//     type Error = embassy_usb::driver::EndpointError;
// }
// impl embedded_io_async::Write for USB {
//     async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
//         self.inner.write_packet(buf).await.map(|_| buf.len())
//     }

//     async fn flush(&mut self) -> Result<(), Self::Error> {
//         // noop
//         Ok(())
//     }
// }
type UsbClass = embassy_usb::class::cdc_acm::CdcAcmClass<
    'static,
    embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>,
>;

/// Print "Hello world" to usb every 250 ms
#[embassy_executor::task]
async fn print_task(mut class: UsbClass) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(250));
    loop {
        class.write_packet(b"Hello world\n").await.unwrap();

        ticker.next().await;
    }
}

// #[embassy_executor::task]
// async fn display(spi: embassy_rp::Peri<'static, embassy_rp::spi::Spi<'static, >>)

#[embassy_executor::task]
async fn blink(pin: embassy_rp::Peri<'static, embassy_rp::gpio::AnyPin>) {
    let mut pin = embassy_rp::gpio::Output::new(pin, Level::High);

    loop {
        pin.set_high();
        embassy_time::Timer::after_millis(100).await;
        pin.set_low();
        embassy_time::Timer::after_millis(100).await;
    }
}
