// use embedded_graphics::{geometry::Point, text::{Baseline, Text}};

// use crate::screen::{flushable_screen::FlushableDrawTarget, screen_trait::{Screen, UpdateResult}};

// struct MainScreen {}

// impl Screen for MainScreen {
//     fn initialize<T>(&self, screen: &mut T) -> Result<(), T::Error>
//     where
//         T: FlushableDrawTarget,
//     {
//         let mut cell_count_str: String<8> = String::new();
//         match cell_count {
//             Some(count) => write!(cell_count_str, "{}S", count),
//             None => write!(cell_count_str, "--"),
//         }
//         .unwrap();
//         Text::with_baseline(&cell_count_str, Point::new(1, 1), BOLD_7X13, Baseline::Top)
//             .draw(screen)?;

//         let mut battery_voltage_str: String<16> = String::new();
//         write!(battery_voltage_str, "{:.1}V   ", battery_voltage).unwrap();
//         Text::with_baseline(
//             &battery_voltage_str,
//             Point::new(1, 15),
//             BOLD_7X13,
//             Baseline::Top,
//         )
//         .draw(screen)
//     }

//     fn update<T>(
//         &self,
//         screen: &mut T,
//         buttons: &super::screen_trait::ButtonState,
//     ) -> Result<UpdateResult, T::Error>
//     where
//         T: FlushableDrawTarget,
//     {
//         todo!()
//     }
// }
