use core::fmt::Write;

use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    text::{Baseline, Text},
};
use heapless::String;

const TEXT_4X6: MonoTextStyle<'static, BinaryColor> = MonoTextStyleBuilder::new()
    .font(&embedded_graphics::mono_font::ascii::FONT_4X6)
    .background_color(embedded_graphics::pixelcolor::BinaryColor::Off)
    .text_color(embedded_graphics::pixelcolor::BinaryColor::On)
    .build();

const BOLD_7X13: MonoTextStyle<'static, BinaryColor> = MonoTextStyleBuilder::new()
    .font(&embedded_graphics::mono_font::ascii::FONT_7X13_BOLD)
    .background_color(embedded_graphics::pixelcolor::BinaryColor::Off)
    .text_color(embedded_graphics::pixelcolor::BinaryColor::On)
    .build();

pub fn draw_status_screen<T>(
    screen: &mut T,
    cell_count: Option<u8>,
    battery_voltage: f32,
) -> Result<(), T::Error>
where
    T: DrawTarget<Color = BinaryColor>,
{
    // Text::with_baseline("Test", Point::new(1, 1), TEXT_4X6, Baseline::Top).draw(screen)?;

    // Line::new(Point::new(0, 7), Point::new(128, 7))
    //     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    //     .draw(screen)?;

    let mut cell_count_str: String<8> = String::new();
    match cell_count {
        Some(count) => write!(cell_count_str, "{}S", count),
        None => write!(cell_count_str, "--"),
    }
    .unwrap();
    Text::with_baseline(&cell_count_str, Point::new(1, 1), BOLD_7X13, Baseline::Top)
        .draw(screen)?;

    let mut battery_voltage_str: String<16> = String::new();
    write!(battery_voltage_str, "{:.1}V   ", battery_voltage).unwrap();
    Text::with_baseline(
        &battery_voltage_str,
        Point::new(1, 15),
        BOLD_7X13,
        Baseline::Top,
    )
    .draw(screen)?;

    Ok(())
}
