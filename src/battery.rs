use crate::{configuration::VOLTAGE_DIVIDER_RATIO, util::map};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, once_lock::OnceLock,
};

/// The number of cells in the battery. Automatically detected at startup.
pub static CELL_COUNT: OnceLock<u8> = OnceLock::new();

static MY_ADC: OnceLock<MyAdc> = OnceLock::new();
struct MyAdc {
    /// The
    adc: Mutex<CriticalSectionRawMutex, embassy_rp::adc::Adc<'static, embassy_rp::adc::Async>>,
    batt_channel: Mutex<CriticalSectionRawMutex, embassy_rp::adc::Channel<'static>>,
}

pub fn init_adc(
    spawner: Spawner,
    adc: embassy_rp::Peri<'static, embassy_rp::peripherals::ADC>,
    batt_voltage_divider_pin: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_28>,
) {
    let adc = Mutex::new(embassy_rp::adc::Adc::new(
        adc,
        crate::Interrupts,
        embassy_rp::adc::Config::default(),
    ));

    let channel = Mutex::new(embassy_rp::adc::Channel::new_pin(
        batt_voltage_divider_pin,
        embassy_rp::gpio::Pull::None,
    ));

    MY_ADC
        .init(MyAdc {
            adc,
            batt_channel: channel,
        })
        .map_err(|_| ())
        .expect("init_adc may only be called once");

    // Initialize CELL_COUNT
    spawner.spawn(cell_count_task()).unwrap();
}

#[embassy_executor::task]
async fn cell_count_task() {
    // Let the battery voltage stabilize before we sample it
    embassy_time::Timer::after_millis(100).await;

    let voltage = read_battery_voltage().await;
    CELL_COUNT.init(calc_cell_count(voltage)).unwrap();
}

async fn read_battery_adc_channel() -> Result<u16, ()> {
    let my_adc = MY_ADC.try_get().ok_or(())?;
    let mut adc = my_adc.adc.lock().await;
    let mut channel = my_adc.batt_channel.lock().await;

    let sample = adc.read(&mut channel).await.unwrap();

    Ok(sample)
}

/// Read the voltage of the battery.
pub async fn read_battery_voltage() -> f32 {
    let adc_voltage = map(
        read_battery_adc_channel().await.unwrap() as f32,
        0.0,
        2048.0,
        0.0,
        3.3,
    );
    adc_voltage / VOLTAGE_DIVIDER_RATIO
}

/// Given a voltage, calculate the number of LiPo battery cells. Will overcount if cells are charged
/// beyond 4.3 volts.
fn calc_cell_count(mut voltage: f32) -> u8 {
    let mut cell_count: u8 = 0;
    while voltage > 0.0 {
        cell_count += 1;
        voltage -= 4.3; // A fully charged LiPo cell is 4.3 volts
    }
    cell_count
}
