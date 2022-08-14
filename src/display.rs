use embedded_graphics::{
	prelude::*,
	text::Text, primitives::Rectangle,
};

use arduino_hal::I2c;
use ssd1306::{prelude::*, Ssd1306, mode::BufferedGraphicsMode};

use crate::DisplayStyle;

type Display = Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

fn byte_to_height(val: u8, max: u8) -> u32 {
	let t = (val >> 2) as u32; // TODO this is a cheap ass solution with awful precision!!!
	return if t > max as u32 { max as u32 } else { t };
	// return ((val as f32 / 255.0) * max as f32) as u32; // TODO get rid of floating point operations!
}

pub fn draw_ui(display: &mut Display, style: &DisplayStyle) {
	Rectangle::new(Point::new(0, 0), Size::new(128, 64))
		.into_styled(style.border_style)
		.draw(display)
		.unwrap();

	Text::new("CPU1", Point::new(2, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU2", Point::new(22, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU3", Point::new(42, 6), style.text_style).draw(display).unwrap();
	Text::new("CPU4", Point::new(62, 6), style.text_style).draw(display).unwrap();
	Text::new("TX", Point::new(104, 6), style.text_style).draw(display).unwrap();
	Text::new("RX", Point::new(116, 6), style.text_style).draw(display).unwrap();
}

pub fn draw_cpu_bar(display: &mut Display, index: u8, value: u8, style: &DisplayStyle) {
	let x = 2 + ((index - 1) * 20);
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

pub fn draw_network_bar(display: &mut Display, direction: NetDirection, value: u8, style: &DisplayStyle) {
	let x = match direction { NetDirection::TX => 104, NetDirection::RX => 116 };
	let height = byte_to_height(value, 54);
	Rectangle::new(Point::new(x as i32, 8), Size::new(10, 54 - (height-1) as u32))
		.into_styled(style.background_style)
		.draw(display)
		.unwrap();
	Rectangle::new(Point::new(x as i32, (62 - height) as i32), Size::new(10, 1 + height as u32))
		.into_styled(style.bar_style)
		.draw(display)
		.unwrap();
}

fn _draw_all(display: &mut Display, cpu1: u8, cpu2: u8, cpu3: u8, cpu4: u8, tx: u8, rx: u8, style: &DisplayStyle) {
	draw_ui(display, style);

	draw_cpu_bar(display, 1, cpu1, style);
	draw_cpu_bar(display, 2, cpu2, style);
	draw_cpu_bar(display, 3, cpu3, style);
	draw_cpu_bar(display, 4, cpu4, style);

	draw_network_bar(display, NetDirection::TX, tx, style);
	draw_network_bar(display, NetDirection::RX, rx, style);

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