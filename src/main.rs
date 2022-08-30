#![no_std]
#![no_main]

use packet::PacketId;
use panic_halt as _;

use embedded_hal::serial::Read;
use embedded_graphics::{
	mono_font::{ascii::FONT_4X6, MonoTextStyle},
	pixelcolor::BinaryColor,
	primitives::{PrimitiveStyleBuilder, PrimitiveStyle},
};

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm, Timer2Pwm};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

mod packet;
mod utils;
mod display;

use crate::packet::PacketBuilder;
use crate::utils::FourLedDisplay;
use display::{draw_ui, draw_cpu_bar, draw_network_bar, NetDirection, Spinner};

pub struct DisplayStyle<'a> {
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
		pins.d11.into_output().into_pwm(&timer2),
		pins.d10.into_output().into_pwm(&timer1),
		pins.d9.into_output().into_pwm(&timer1),
		pins.d3.into_output().into_pwm(&timer2),
	);
	let i2c = arduino_hal::i2c::I2c::new(
		dp.TWI, pins.a4.into_pull_up_input(), pins.a5.into_pull_up_input(), 800000
	);
	let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

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
	let mut s = Spinner::new(91, 10);

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
							if let Some(payload) = pkt.payload && payload.len() == 8 {
								s.draw(&mut display);
								draw_cpu_bar(&mut display, 1, payload[0], &style);
								draw_cpu_bar(&mut display, 2, payload[1], &style);
								draw_cpu_bar(&mut display, 3, payload[2], &style);
								draw_cpu_bar(&mut display, 4, payload[3], &style);
								draw_network_bar(&mut display, NetDirection::TX, payload[4], payload[6], &style);
								draw_network_bar(&mut display, NetDirection::RX, payload[5], payload[7], &style);
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

