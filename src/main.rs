#![no_std]
#![no_main]

mod battery;
mod configuration;
mod debounce;
mod dshot;
mod input;
mod menu;
mod motor;
mod usb;
mod util;

use embassy_executor::Spawner;
use ssd1306::{
    Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, DisplayConfigAsync},
    prelude::I2CInterface,
    size::DisplaySize128x64,
};

use panic_probe as _;

use crate::{debounce::Debouncer, input::BinarySource, motor::Motor};

// Set up the device interrupts
embassy_rp::bind_interrupts!(struct Interrupts {
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Use the external oscillator and configure the PLLs for 133MHz
    let hal_config = embassy_rp::config::Config::new(
        embassy_rp::clocks::ClockConfig::system_freq(133_000_000).unwrap(),
    );
    let p = embassy_rp::init(hal_config);

    battery::init_adc(spawner, p.ADC, p.PIN_28);

    usb::init_usb_serial(spawner, p.USB);
    usb::print(b"Initialized USB").await;

    /**** INITIALIZE DISPLAY ****/
    let mut i2c_config = embassy_rp::i2c::Config::default();
    i2c_config.frequency = 1_000_000; // 1MHz
    let i2c = embassy_rp::i2c::I2c::new_async(p.I2C1, p.PIN_15, p.PIN_14, Interrupts, i2c_config);
    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = ssd1306::Ssd1306Async::new(
        interface,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();
    display.init().await.unwrap();
    display.clear_buffer();
    display.flush().await.unwrap();

    spawner.spawn(display_task(display)).unwrap();

    let (dshot0, dshot1, dshot2, dshot3) = dshot::init_dshot(
        p.PIO0,
        p.PIN_0,
        p.PIN_1,
        p.PIN_2,
        p.PIN_3,
        configuration::DSHOT_SPEED,
    );
    let (mut motor0, mut motor1, mut motor2, mut motor3) = embassy_futures::join::join4(
        Motor::new(dshot0, configuration::MOTOR0_CONFIG.reversed),
        Motor::new(dshot1, configuration::MOTOR1_CONFIG.reversed),
        Motor::new(dshot2, configuration::MOTOR2_CONFIG.reversed),
        Motor::new(dshot3, configuration::MOTOR3_CONFIG.reversed),
    )
    .await;

    let mut rev = Debouncer::new(
        embassy_rp::gpio::Input::new(p.PIN_18, embassy_rp::gpio::Pull::Up).set_inverted(true),
        embassy_time::Duration::from_ticks(0),
        embassy_time::Duration::from_micros(2000),
    );
    let mut trigger = Debouncer::new(
        embassy_rp::gpio::Input::new(p.PIN_21, embassy_rp::gpio::Pull::Up).set_inverted(true),
        embassy_time::Duration::from_ticks(0),
        embassy_time::Duration::from_micros(2000),
    );
    let mut menu_button = Debouncer::new(
        embassy_rp::gpio::Input::new(p.PIN_19, embassy_rp::gpio::Pull::Up).set_inverted(true),
        embassy_time::Duration::from_micros(1000),
        embassy_time::Duration::from_micros(1000),
    );

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_micros(1000));
    loop {
        rev.update();
        trigger.update();
        menu_button.update();

        let mut throttle = 0;
        if rev.get() {
            throttle = 1000;
        }

        if throttle == 0 {
            motor0.stop();
            motor2.stop();
        } else {
            motor0.set_throttle(throttle);
            motor2.set_throttle(throttle);
        }

        // Send the dshot commands
        let dshot_results = embassy_futures::join::join4(
            motor0.update(),
            motor1.update(),
            motor2.update(),
            motor3.update(),
        )
        .await;
        if let Err(_e) = dshot_results.0 {
            usb::print(b"Dshot error on motor 0\n").await;
        }
        if let Err(_e) = dshot_results.1 {
            usb::print(b"Dshot error on motor 1\n").await;
        }
        if let Err(_e) = dshot_results.2 {
            usb::print(b"Dshot error on motor 2\n").await;
        }
        if let Err(_e) = dshot_results.3 {
            usb::print(b"Dshot error on motor 3\n").await;
        }

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn display_task(
    mut display: Ssd1306Async<
        I2CInterface<
            embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C1, embassy_rp::i2c::Async>,
        >,
        DisplaySize128x64,
        BufferedGraphicsModeAsync<DisplaySize128x64>,
    >,
) {
    // Update the display at roughly 30fps
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(33));

    loop {
        usb::print(b"updating display\n").await;

        let voltage = battery::read_battery_voltage().await;
        usb::print(b"done reading battery voltage\n").await;

        // display
        //     .clear(embedded_graphics::pixelcolor::BinaryColor::Off)
        //     .unwrap();

        menu::draw_status_screen(
            &mut display,
            battery::CELL_COUNT.try_get().map(|n| *n),
            voltage,
        )
        .unwrap();

        display.flush().await.unwrap();

        ticker.next().await;
    }
}
