use embassy_time::{Duration, Instant};

use crate::input::BinarySource;

pub struct Debouncer<T> {
    last_value: bool,
    debounced_value: bool,
    source: T,

    /// Time that the input signal must be high before the output signal follows.
    rising_edge_duration: Duration,
    /// Time that the input signal must be low before the output signal follows.
    falling_edge_duration: Duration,
    time_of_last_edge: Instant,
}

impl<T> Debouncer<T>
where
    T: BinarySource,
{
    pub fn new(stream: T, rising_edge_duration: Duration, falling_edge_duration: Duration) -> Self {
        Self {
            last_value: stream.get(),
            debounced_value: stream.get(),
            source: stream,
            rising_edge_duration,
            falling_edge_duration,
            time_of_last_edge: Instant::now(),
        }
    }

    pub fn get(&self) -> bool {
        self.debounced_value
    }

    pub fn update(&mut self) {
        let current_value = self.source.get();

        let rising = !self.last_value && current_value;
        let falling = self.last_value && !current_value;
        if rising || falling {
            self.time_of_last_edge = Instant::now();
        }

        let debounce_duration = if current_value {
            self.rising_edge_duration
        } else {
            self.falling_edge_duration
        };
        if self.time_of_last_edge.elapsed() > debounce_duration {
            self.debounced_value = current_value;
        }

        self.last_value = current_value;
    }
}
