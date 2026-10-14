#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embedded_graphics::{Drawable, text::Text};
use ssd1306::mode::DisplayConfigAsync;

use panic_probe as _;

// Set up the device interrupts
embassy_rp::bind_interrupts!(struct Interrupts {
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

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
