#![no_std]
#![no_main]

mod battery;
mod configuration;
mod dshot;
mod menu;
mod usb;
mod util;

use embassy_executor::Spawner;
use embedded_graphics::draw_target::DrawTarget;
use ssd1306::{
    Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, DisplayConfigAsync},
    prelude::I2CInterface,
    size::DisplaySize128x64,
};

use panic_probe as _;

use crate::configuration::{MOTOR0_CONFIG, MOTOR1_CONFIG, MOTOR2_CONFIG, MOTOR3_CONFIG};

// Set up the device interrupts
embassy_rp::bind_interrupts!(struct Interrupts {
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    battery::init_adc(spawner, p.ADC, p.PIN_28);

    usb::init_usb_serial(spawner, p.USB);
    usb::print(b"Initialized USB").await;

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
    display.clear_buffer();
    display.flush().await.unwrap();

    spawner.spawn(display_task(display)).unwrap();

    let (mut dshot0, mut dshot1, mut dshot2, mut dshot3) = dshot::init_dshot(
        p.PIO0,
        p.PIN_0,
        p.PIN_1,
        p.PIN_2,
        p.PIN_3,
        embassy_dshot::DshotSpeed::DShot600,
    );

    usb::print(b"Arming dshot...").await;
    let arm_duration = embassy_time::Duration::from_millis(500); // This can probably be 300ms
    let _arm_results = embassy_futures::join::join4(
        dshot0.arm_async(arm_duration),
        dshot1.arm_async(arm_duration),
        dshot2.arm_async(arm_duration),
        dshot3.arm_async(arm_duration),
    )
    .await;

    if let Err(_e) = _arm_results.0 {
        usb::print(b"Encountered error arming dshot 0\n").await;
    }
    if let Err(_e) = _arm_results.1 {
        usb::print(b"Encountered error arming dshot 1\n").await;
    }
    if let Err(_e) = _arm_results.2 {
        usb::print(b"Encountered error arming dshot 2\n").await;
    }
    if let Err(_e) = _arm_results.3 {
        usb::print(b"Encountered error arming dshot 3\n").await;
    }
    usb::print(b" done\n").await;

    dshot0
        .send_command_repeated_async(
            if MOTOR0_CONFIG.reversed {
                embassy_dshot::Command::SpinDirectionReversed
            } else {
                embassy_dshot::Command::SpinDirectionNormal
            },
            6,
        )
        .await
        .unwrap();
    dshot1
        .send_command_repeated_async(
            if MOTOR1_CONFIG.reversed {
                embassy_dshot::Command::SpinDirectionReversed
            } else {
                embassy_dshot::Command::SpinDirectionNormal
            },
            6,
        )
        .await
        .unwrap();
    dshot2
        .send_command_repeated_async(
            if MOTOR2_CONFIG.reversed {
                embassy_dshot::Command::SpinDirectionReversed
            } else {
                embassy_dshot::Command::SpinDirectionNormal
            },
            6,
        )
        .await
        .unwrap();
    dshot3
        .send_command_repeated_async(
            if MOTOR3_CONFIG.reversed {
                embassy_dshot::Command::SpinDirectionReversed
            } else {
                embassy_dshot::Command::SpinDirectionNormal
            },
            6,
        )
        .await
        .unwrap();

    let rev = embassy_rp::gpio::Input::new(p.PIN_18, embassy_rp::gpio::Pull::Up);
    let trigger = embassy_rp::gpio::Input::new(p.PIN_21, embassy_rp::gpio::Pull::Up);

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_micros(100));
    loop {
        // let voltage = battery::read_battery_voltage().await;

        // display
        //     .clear(embedded_graphics::pixelcolor::BinaryColor::Off)
        //     .unwrap();
        // menu::draw_status_screen(
        //     &mut display,
        //     battery::CELL_COUNT.try_get().map(|n| *n),
        //     voltage,
        // )
        // .unwrap();
        // display.flush().await.unwrap();

        // usb::print(b"Battery voltage (mV): ").await;
        // usb::print_u32((voltage * 1000.0) as u32).await;
        // usb::print(b"; Cell count: ").await;
        // match battery::CELL_COUNT.try_get() {
        //     Some(count) => usb::print_u32(*count as u32).await,
        //     None => usb::print(b"--").await,
        // }
        // usb::print(b"\n").await;

        let mut throttle = 0;
        if rev.is_low() {
            throttle = 1000;
        }

        if throttle == 0 {
            let (idle_result0, idle_result2) = embassy_futures::join::join(
                dshot0.send_command_async(embassy_dshot::Command::MotorStop),
                dshot2.send_command_async(embassy_dshot::Command::MotorStop),
            )
            .await;
            idle_result0.unwrap();
            idle_result2.unwrap();

            // dshot0.send_command(embassy_dshot::Command::MotorStop).unwrap();
            // dshot2.send_command(embassy_dshot::Command::MotorStop).unwrap();
        } else {
            let (telem0, telem2) = embassy_futures::join::join(
                dshot0.throttle_with_telemetry(throttle),
                dshot2.throttle_with_telemetry(throttle),
            )
            .await;
            usb::print(b"")
            usb::print_u32(telem0.unwrap().erpm).await;
            usb::print(b"\n").await;
            // let telem0 = .await.unwrap();
            // let telem2 = dshot2.throttle_with_telemetry(throttle).await.unwrap();
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
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(30));

    loop {
        usb::print(b"updating display\n").await;

        let voltage = battery::read_battery_voltage().await;
        usb::print(b"done reading battery voltage").await;

        display
            .clear(embedded_graphics::pixelcolor::BinaryColor::Off)
            .unwrap();

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
