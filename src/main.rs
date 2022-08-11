#![no_std]
#![no_main]

use packet::PacketId;
use panic_halt as _;

use embedded_hal::serial::Read;

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm, Timer2Pwm};
use sh1106::{prelude::GraphicsMode, Builder};

mod packet;
mod utils;

use crate::packet::PacketBuilder;
use crate::utils::FourLedDisplay;

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
	led_rx.set_high();
	led_tx.set_high();
	
	// prepare display
	let mut display: GraphicsMode<_> = Builder::new().with_size(sh1106::prelude::DisplaySize::Display128x64).connect_i2c(i2c).into();
	cpu_leds.set(1, 255);

	display.init().unwrap();
	cpu_leds.set(2, 255);

	let mut flip : u8 = 0;
	for x in 0..128 {
		for y in 0..64 {
			cpu_leds.set(3, if flip == 0 { 0 } else { 255 });
			display.set_pixel(x, y, flip);
			flip = !flip;
		}
	}
	cpu_leds.set(3, 255);

	display.flush().unwrap();
	cpu_leds.set(4, 255);

	led_load.set_low();
	led_tx.set_low();
	led_rx.set_low();

	let mut pkt_builder = PacketBuilder::new();

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

