use embedded_graphics::{
	prelude::*,
	text::Text, primitives::Rectangle, mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_4X6}, pixelcolor::BinaryColor,
};

use arduino_hal::I2c;
use ssd1306::{prelude::*, Ssd1306, mode::BufferedGraphicsMode};

use crate::DisplayStyle;

type Display = Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

pub struct Spinner {
	flip: bool,
	x: i32,
	y: i32,
	style: MonoTextStyle<'static, BinaryColor>,
}

impl Spinner {
	pub fn new(x: i32, y: i32) -> Self {
		let style = MonoTextStyleBuilder::new().font(&FONT_4X6).text_color(BinaryColor::On).background_color(BinaryColor::Off).build();
		Spinner { flip: true, x, y, style }
	}

	pub fn draw(&mut self, display: &mut Display) {
		Text::new(if self.flip { "-" } else { "|" }, Point::new(self.x, self.y), self.style)
			.draw(display)
			.unwrap();
		self.flip = !self.flip;
	}
}

fn byte_to_height(val: u8, max: u8) -> u32 {
	let t = (val >> 2) as u32; // TODO this is a cheap ass solution with awful precision!!!
	return if t > max as u32 { max as u32 } else { t };
	// return ((val as f32 / 255.0) * max as f32) as u32; // TODO get rid of floating point operations!
}

pub fn _draw_number_as_box(display: &mut Display, value: u32, width: u32, height: u32, base_x: u32, base_y: u32) {
	let mut i = 0;
	for x in 0..width {
		for y in 0..height {
			display.set_pixel(base_x + x, base_y + y, ((value << i) & 1) == 1);
			i = (i + i) % 32;
		}
	}

}

pub fn draw_ui(display: &mut Display, style: &DisplayStyle) {
	Rectangle::new(Point::new(0, 0), Size::new(85, 64))
		.into_styled(style.border_style)
		.draw(display)
		.unwrap();

	Rectangle::new(Point::new(87, 0), Size::new(11, 16))
		.into_styled(style.border_style)
		.draw(display)
		.unwrap();

	// since my specific display is 2 displays of different colors joined, there's a small gap
	// between pixels 19 and 20. This makes the 2 extra blank pixels look bad, so I'm removing 3.
	// On normal screens this will look worse and you should put them back.
	Rectangle::new(Point::new(87, 16), Size::new(11, 48))
		.into_styled(style.border_style)
		.draw(display)
		.unwrap();

	Rectangle::new(Point::new(100, 0), Size::new(28, 64))
		.into_styled(style.border_style)
		.draw(display)
		.unwrap();


	Text::new("CPU1", Point::new(5, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU2", Point::new(25, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU3", Point::new(45, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU4", Point::new(65, 6), style.text_style).draw(display).unwrap();
	Text::new("TX", Point::new(104, 6), style.text_style).draw(display).unwrap();
	Text::new("RX", Point::new(116, 6), style.text_style).draw(display).unwrap();
}

pub fn draw_cpu_bar(display: &mut Display, index: u8, value: u8, style: &DisplayStyle) {
	let x = 5 + ((index - 1) * 20);
	let height = byte_to_height(value, 54);
	Rectangle::new(Point::new(x as i32, 8), Size::new(15, 54 - (height-1) as u32))
		.into_styled(style.background_style)
		.draw(display)
		.unwrap();
	Rectangle::new(Point::new(x as i32, (62 - height) as i32), Size::new(15, 1 + height as u32))
		.into_styled(style.bar_style)
		.draw(display)
		.unwrap();
}

pub enum NetDirection {
	TX, RX
}

pub fn draw_network_bar(display: &mut Display, direction: NetDirection, value_fine: u8, value_wide: u8, style: &DisplayStyle) {
	let x = match direction { NetDirection::TX => 103, NetDirection::RX => 115 };
	let height_fine = byte_to_height(value_fine, 54);
	let height_wide = byte_to_height(value_wide, 54);
	Rectangle::new(Point::new(x as i32, 8), Size::new(2, 54 - (height_fine-1) as u32))
		.into_styled(style.background_style)
		.draw(display)
		.unwrap();
	Rectangle::new(Point::new(x as i32, (62 - height_fine) as i32), Size::new(2, 1 + height_fine as u32))
		.into_styled(style.bar_style)
		.draw(display)
		.unwrap();
	Rectangle::new(Point::new(3 + x as i32, 8), Size::new(7, 54 - (height_wide-1) as u32))
		.into_styled(style.background_style)
		.draw(display)
		.unwrap();
	Rectangle::new(Point::new(3 + x as i32, (62 - height_wide) as i32), Size::new(7, 1 + height_wide as u32))
		.into_styled(style.bar_style)
		.draw(display)
		.unwrap();
}

fn _draw_all(display: &mut Display, cpu1: u8, cpu2: u8, cpu3: u8, cpu4: u8, tx: u8, rx: u8, tx_wide:u8, rx_wide: u8, style: &DisplayStyle) {
	draw_ui(display, style);

	draw_cpu_bar(display, 1, cpu1, style);
	draw_cpu_bar(display, 2, cpu2, style);
	draw_cpu_bar(display, 3, cpu3, style);
	draw_cpu_bar(display, 4, cpu4, style);

	draw_network_bar(display, NetDirection::TX, tx, tx_wide, style);
	draw_network_bar(display, NetDirection::RX, rx, rx_wide, style);

	display.flush().unwrap();
}

fn _display_grid(display: &mut Display) {
	let mut flip : bool = false;
	for y in 0..64 {
		for x in 0..128 {
			display.set_pixel(x, y, flip);
			flip = !flip;
		}
		flip = !flip;
	}
	display.flush().unwrap();
}
