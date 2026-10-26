// use core::fmt::Debug;

// use display_interface::AsyncWriteOnlyDataCommand;
// use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
// use ssd1306::{Ssd1306Async, mode::BufferedGraphicsModeAsync, size::DisplaySizeAsync};

// /// A binary color display that implements DrawTarget and that can be flushed.
// pub trait FlushableScreen: DrawTarget {
//     async fn flush(&mut self) -> Result<(), Self::Error>;
// }

// /// Implement FlushableScreen for the Ssd1306 driver
// impl<DI, SIZE> FlushableScreen for Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
// where
//     DI: AsyncWriteOnlyDataCommand,d
//     SIZE: DisplaySizeAsync,
// {
//     async fn flush(&mut self) -> Result<(), Self::Error> {
//         self.flush().await
//     }
// }


pub struct ButtonState {
    rev_trigger_pressed: bool,
    rev_trigger_rising: bool,
    rev_trigger_falling: bool,
    // etc...
}


pub enum UpdateResult {
    Noop,
    GoBack,
}

pub trait Screen {
    fn initialize<T>(&self, screen: &mut T) -> Result<(), T::Error>
    where
        T: FlushableScreen;

    fn update<T>(&self, screen: &mut T, buttons: &ButtonState) -> Result<UpdateResult, T::Error>
    where
        T: FlushableScreen;
}
