#![no_std]
#![no_main]

use packet::PacketId;
use panic_halt as _;

use embedded_hal::serial::Read;
use embedded_graphics::{
	mono_font::{ascii::FONT_4X6, MonoTextStyle},
	pixelcolor::BinaryColor,
	prelude::*,
	text::Text, primitives::{PrimitiveStyleBuilder, Rectangle, PrimitiveStyle},
};

use arduino_hal::{simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm, Timer2Pwm}, I2c};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode};

mod packet;
mod utils;

use crate::packet::PacketBuilder;
use crate::utils::FourLedDisplay;

type Display = Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

struct DisplayStyle<'a> {
	border_style: PrimitiveStyle<BinaryColor>,
	text_style: MonoTextStyle<'a, BinaryColor>,
	bar_style: PrimitiveStyle<BinaryColor>,
	background_style: PrimitiveStyle<BinaryColor>,
}

#[arduino_hal::entry]
fn main() -> ! {
	// init board peripherals
	let dp = arduino_hal::Peripherals::take().unwrap();
	let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Direct);
	let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Direct);
	let pins = arduino_hal::pins!(dp);
	let mut led_load = pins.d6.into_output();
	let mut led_rx = pins.d5.into_output(); // green
	let mut led_tx = pins.d4.into_output(); // red
	let button = pins.d2.into_pull_up_input();
	let mut cpu_leds = FourLedDisplay::new(
		pins.d3.into_output().into_pwm(&timer2),
		pins.d9.into_output().into_pwm(&timer1),
		pins.d10.into_output().into_pwm(&timer1),
		pins.d11.into_output().into_pwm(&timer2),
	);
	let i2c = arduino_hal::i2c::I2c::new(
		dp.TWI, pins.a4.into_pull_up_input(), pins.a5.into_pull_up_input(), 100000
	);
	let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

	led_load.set_high();
	led_tx.set_low();
	led_rx.set_low();

	let style = DisplayStyle {
		border_style: PrimitiveStyleBuilder::new()
			.stroke_width(1)
			.stroke_color(BinaryColor::On)
			.build(),
		text_style: MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
		bar_style: PrimitiveStyleBuilder::new()
			.stroke_width(1)
			.stroke_color(BinaryColor::On)
			.fill_color(BinaryColor::On)
			.build(),
		background_style: PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::Off)
			.fill_color(BinaryColor::Off)
			.build(),
	};
	
	// prepare display
	let interface = I2CDisplayInterface::new(i2c);
	let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();

	display.init().unwrap();
	cpu_leds.set(1, 255);

	display.clear();
	display.flush().unwrap();
	cpu_leds.set(2, 255);

	draw_ui(&mut display, &style);
	cpu_leds.set(3, 255);

	display.flush().unwrap();
	cpu_leds.set(4, 255);

	led_load.set_low();

	let mut pkt_builder = PacketBuilder::new();

	// TODO put these in a struct

	arduino_hal::delay_ms(25);
	cpu_leds.set_all(0);
	loop { // main loop
		if button.is_low() { // If reset button is pressed, don't process serial bus
			cpu_leds.set_all(0);
			led_load.set_high();
			continue;
		}

		match serial.read() { // if there's a byte available
			Ok(value) => {
				led_load.set_high();
				if let Some(pkt) = pkt_builder.update(value) { // update packet builder
					match pkt.id { // if a packet is ready, match against its id
						PacketId::Reset => {
							cpu_leds.set_all(0);
						},
						PacketId::SetLedsPacket => {
							if let Some(payload) = pkt.payload && payload.len() == 4 {
								cpu_leds.set_many(payload[0], payload[1], payload[2], payload[3]);
							}
						},
						PacketId::NetStatePacket => {
							if let Some(payload) = pkt.payload && payload.len() == 2 {
								if payload[0] == 0 { led_tx.set_low() } else { led_tx.set_high() };
								if payload[1] == 0 { led_rx.set_low() } else { led_rx.set_high() };
							}
						},
						PacketId::ScreenDrawPacket => {
							if let Some(payload) = pkt.payload && payload.len() == 6 {
								draw_cpu_bar(&mut display, 1, payload[0], &style);
								draw_cpu_bar(&mut display, 2, payload[1], &style);
								draw_cpu_bar(&mut display, 3, payload[2], &style);
								draw_cpu_bar(&mut display, 4, payload[3], &style);
								draw_network_bar(&mut display, NetDirection::TX, payload[4], &style);
								draw_network_bar(&mut display, NetDirection::RX, payload[5], &style);
								display.flush().unwrap();
							}
						},
						_ => {}, // TODO log it?
					}
				}
			},
			Err(_) => {
				led_load.set_low();
			},
		}
	}
}

fn byte_to_height(val: u8, max: u8) -> u32 {
	let t = (val >> 2) as u32; // TODO this is a cheap ass solution with awful precision!!!
	return if t > max as u32 { max as u32 } else { t };
	// return ((val as f32 / 255.0) * max as f32) as u32; // TODO get rid of floating point operations!
}

fn draw_ui(display: &mut Display, style: &DisplayStyle) {
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

fn draw_cpu_bar(display: &mut Display, index: u8, value: u8, style: &DisplayStyle) {
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

enum NetDirection {
	TX, RX
}

fn draw_network_bar(display: &mut Display, direction: NetDirection, value: u8, style: &DisplayStyle) {
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

fn _display_grid(display: &mut Display, cpu_leds: &mut FourLedDisplay) {
	let mut flip : bool = false;
	for y in 0..64 {
		for x in 0..128 {
			cpu_leds.set(3, if flip { 0 } else { 255 });
			display.set_pixel(x, y, flip);
			flip = !flip;
		}
		flip = !flip;
	}
	display.flush().unwrap();
}
