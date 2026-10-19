#![no_std]
#![no_main]

mod battery;
mod configuration;
mod dshot;
mod menu;
mod usb;
mod util;

use embassy_executor::Spawner;
use embassy_rp::gpio::Level;
use embedded_graphics::draw_target::DrawTarget;
use ssd1306::mode::DisplayConfigAsync;

use panic_probe as _;

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

    usb::init_usb_serial(spawner, p.USB, Interrupts);
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

    let (mut dshot0, mut dshot1, mut dshot2, mut dshot3) = dshot::init_dshot(
        p.PIO0,
        p.PIN_0,
        p.PIN_1,
        p.PIN_2,
        p.PIN_3,
        embassy_dshot::DshotSpeed::DShot600,
    );

    usb::print(b"Arming dshot...").await;
    let arm_duration = embassy_time::Duration::from_millis(2000);
    let _arm_results = embassy_futures::join::join4(
        dshot0.arm_async(arm_duration),
        dshot1.arm_async(arm_duration),
        dshot2.arm_async(arm_duration),
        dshot3.arm_async(arm_duration),
    )
    .await; // TODO: Don't ignore the arm results
    usb::print(b" done\n").await;

    let rev = embassy_rp::gpio::Input::new(p.PIN_18, embassy_rp::gpio::Pull::Up);
    let trigger = embassy_rp::gpio::Input::new(p.PIN_21, embassy_rp::gpio::Pull::Up);

    spawner.spawn(blink(p.PIN_13.into())).unwrap();

    loop {
        let voltage = battery::read_battery_voltage().await;

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

        usb::print(b"Battery voltage (mV): ").await;
        usb::print_u32((voltage * 1000.0) as u32).await;
        usb::print(b"; Cell count: ").await;
        match battery::CELL_COUNT.try_get() {
            Some(count) => usb::print_u32(*count as u32).await,
            None => usb::print(b"--").await,
        }
        usb::print(b"\n").await;

        if rev.is_low() {
            usb::print(b"REVVING\n").await;
            // let mut throttle = 0;
            let throttle = 1000;
            dshot0.throttle_async(throttle).await.unwrap();
            dshot1.throttle_async(throttle).await.unwrap();
        } else {
            dshot0.throttle_idle_async().await.unwrap();
            dshot1.throttle_idle_async().await.unwrap();
        }
        embassy_time::Timer::after_millis(10).await;
    }
}

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
