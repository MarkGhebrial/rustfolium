use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::draw_target::DrawTarget;
use ssd1306::{Ssd1306Async, mode::BufferedGraphicsModeAsync, size::DisplaySizeAsync};

/// A binary color display that implements DrawTarget and that can be flushed.
pub trait FlushableDrawTarget: DrawTarget {
    async fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Implement FlushableScreen for the Ssd1306 driver
impl<DI, SIZE> FlushableDrawTarget for Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
{
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await
    }
}
