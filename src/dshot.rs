pub type Dshot<const SM: usize> =
    embassy_dshot::rp::BidirDshotPio<'static, embassy_rp::peripherals::PIO0, SM>;

pub fn init_dshot(
    pio0: embassy_rp::Peri<'static, embassy_rp::peripherals::PIO0>,
    pin0: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_0>,
    pin1: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_1>,
    pin2: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_2>,
    pin3: embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_3>,
    speed: embassy_dshot::DshotSpeed,
) -> (Dshot<0>, Dshot<1>, Dshot<2>, Dshot<3>) {
    // Initialize the PIO peripheral and load the dshot program
    let mut pio = embassy_rp::pio::Pio::new(pio0, crate::Interrupts);
    let program = embassy_dshot::rp::BidirDshotProgram::new(&mut pio.common);

    let dshot0 =
        embassy_dshot::rp::BidirDshotPio::new(pio.sm0, &mut pio.common, pin0, &program, speed);
    let dshot1 =
        embassy_dshot::rp::BidirDshotPio::new(pio.sm1, &mut pio.common, pin1, &program, speed);
    let dshot2 =
        embassy_dshot::rp::BidirDshotPio::new(pio.sm2, &mut pio.common, pin2, &program, speed);
    let dshot3 =
        embassy_dshot::rp::BidirDshotPio::new(pio.sm3, &mut pio.common, pin3, &program, speed);

    (dshot0, dshot1, dshot2, dshot3)
}
