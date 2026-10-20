use embassy_dshot::{Command, DshotError, rp::BidirDshotPio};
use embassy_time::Instant;

use crate::{configuration::ARM_DURATION, util::clamp};

#[derive(PartialEq, Eq)]
enum ArmingState {
    ARMING,
    ARMED,
}
#[derive(PartialEq, Eq)]
enum ControlMode {
    STOPPED,
    THROTTLE,
}

pub struct Motor<'a, PIO, const SM: usize>
where
    PIO: embassy_rp::pio::Instance,
{
    dshot: BidirDshotPio<'a, PIO, SM>,

    /// The moment when we began arming the esc
    arming_start: embassy_time::Instant,
    arming_state: ArmingState,
    control_mode: ControlMode,

    current_throttle: u16,
    current_rpm: u32,
}

impl<'a, PIO, const SM: usize> Motor<'a, PIO, SM>
where
    PIO: embassy_rp::pio::Instance,
{
    /// Construct a new Motor. This begins the dshot arming process
    pub async fn new(
        mut dshot: BidirDshotPio<'a, PIO, SM>,
        reversed: bool, // TODO: Change this parameter to MotorConfiguration
    ) -> Self {
        let direction_command = if reversed {
            embassy_dshot::Command::SpinDirectionReversed
        } else {
            embassy_dshot::Command::SpinDirectionNormal
        };
        dshot
            .send_command_repeated_async(direction_command, 6)
            .await
            .unwrap(); // TODO: Don't unwrap

        Self {
            dshot,
            arming_state: ArmingState::ARMING,
            arming_start: Instant::now(),
            control_mode: ControlMode::STOPPED,
            current_throttle: 0,
            current_rpm: 0,
        }
    }

    // TODO: Function to change the spin direction of the motor on the fly, after the motor has
    // already been initialized
    // pub async fn set_reversed(&mut self, reversed: bool) {
    //     se
    // }

    /// Set the desired motor throttle. Throttle values will be clamped to the range 0..=2000 and,
    /// if the ESC is armed, will be sent to the ESC on the next call to `.update()`.
    pub fn set_throttle(&mut self, throttle: u16) {
        self.current_throttle = clamp(throttle, 0, 2000);
        self.control_mode = ControlMode::THROTTLE;
    }

    /// Stop the motor. This is not equivalent to sending zero throttle. Zero throttle causes the
    /// motor to twitch.
    pub fn stop(&mut self) {
        self.control_mode = ControlMode::STOPPED;
    }

    pub fn is_stopped(&self) -> bool {
        self.control_mode == ControlMode::STOPPED
    }

    /// Has the motor been armed?
    pub fn is_armed(&self) -> bool {
        self.arming_state != ArmingState::ARMING
    }

    pub fn get_rpm(&self) -> u32 {
        self.current_rpm
    }

    /// This is the function that actually sends dshot commands. It must be called at regular,
    /// frequent intervals. The exact interval required depends on the ESC firmware, but should be
    /// less than a few milliseconds.
    pub async fn update(&mut self) -> Result<(), DshotError> {
        // Update the arming state after we've been sending MotorStop commands for ARM_DURATION
        // TODO: Transition arming state back to ARMING if too much time elapses between `.update()` calls
        if !self.is_armed() && self.arming_start.elapsed() >= ARM_DURATION {
            self.arming_state = ArmingState::ARMED;
        }

        if self.is_armed() && self.control_mode == ControlMode::THROTTLE {
            // Tell the motor to spin up
            let telem = self
                .dshot
                .throttle_with_telemetry(self.current_throttle)
                .await?;
            self.current_rpm = telem.rpm(5); // TODO: How many motor poles?
        } else {
            // Tell the motor to stop
            self.dshot.send_command_async(Command::MotorStop).await?;
            self.current_rpm = 0;
        }

        Ok(())
    }
}
