use embassy_dshot::DshotSpeed;
use embassy_time::Duration;

pub const ADC_RESOLUTION: usize = 4096;

pub const VOLTAGE_DIVIDER_RATIO: f32 = 1.0 / 11.0;
/// Per-cell threshold for the low battery warning.
pub const LOW_VOLTAGE_THRESHOLD: f32 = 3.5;
/// Per-cell threshold for the critical battery warning.
pub const CRITICAL_VOLTAGE_THRESHOLD: f32 = 3.2;

/***** MOTOR CONFIGURATION *****/
pub const DSHOT_SPEED: DshotSpeed = DshotSpeed::DShot600;
/// How long to send MotorStop commands before the ESC is armed.
pub const ARM_DURATION: Duration = Duration::from_millis(1500);

pub struct MotorConfiguration {
    pub reversed: bool,
}
pub const MOTOR0_CONFIG: MotorConfiguration = MotorConfiguration { reversed: false };
pub const MOTOR1_CONFIG: MotorConfiguration = MotorConfiguration { reversed: false };
pub const MOTOR2_CONFIG: MotorConfiguration = MotorConfiguration { reversed: false };
pub const MOTOR3_CONFIG: MotorConfiguration = MotorConfiguration { reversed: false };
