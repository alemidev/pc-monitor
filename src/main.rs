#![no_std]
#![no_main]

use panic_halt as _;

use embedded_hal::serial::Read;

use arduino_hal::{simple_pwm::*, port::{Pin, mode::{PwmOutput, PullUp, Input}}, hal::port::{PD3, PB1, PB2, PB3, PD2}};

// TODO can I make it a generic "Pin" and use a slice?
struct FourLedDisplay {
	led1: Pin<PwmOutput<Timer2Pwm>, PD3>,
	led2: Pin<PwmOutput<Timer1Pwm>, PB1>,
	led3: Pin<PwmOutput<Timer1Pwm>, PB2>,
	led4: Pin<PwmOutput<Timer2Pwm>, PB3>,
	button: Pin<Input<PullUp>, PD2>,
	counter: u8,
}

impl FourLedDisplay {
	fn new(
		led1: Pin<PwmOutput<Timer2Pwm>, PD3>,
		led2: Pin<PwmOutput<Timer1Pwm>, PB1>,
		led3: Pin<PwmOutput<Timer1Pwm>, PB2>,
		led4: Pin<PwmOutput<Timer2Pwm>, PB3>,
		button: Pin<Input<PullUp>, PD2>,
	) -> Self {
		FourLedDisplay{
			led1, led2, led3, led4, button,
			counter: 0,
		}
	}

	fn update(&mut self, value: u8) {
		match self.counter {
			0 => self.led1.set_duty(value),
			1 => self.led2.set_duty(value),
			2 => self.led3.set_duty(value),
			3 => self.led4.set_duty(value),
			_ => {},
		}
		self.counter = (self.counter + 1) % 4;
	}

	fn init(mut self) -> Self {
		self.led1.enable();
		self.led2.enable();
		self.led3.enable();
		self.led4.enable();
		self
	}

	fn should_reset(&self) -> bool { self.button.is_low() }

	fn reset(&mut self) {
		self.counter = 0;
		self.led1.set_duty(0);
		self.led2.set_duty(0);
		self.led3.set_duty(0);
		self.led4.set_duty(0);
	}
}

#[arduino_hal::entry]
fn main() -> ! {
	// init board peripherals
	let dp = arduino_hal::Peripherals::take().unwrap();
	let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale8);
	let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale8);
	let pins = arduino_hal::pins!(dp);
	let led1 = pins.d3.into_output().into_pwm(&timer2);
	let led2 = pins.d9.into_output().into_pwm(&timer1);
	let led3 = pins.d10.into_output().into_pwm(&timer1);
	let led4 = pins.d11.into_output().into_pwm(&timer2);
	let button = pins.d2.into_pull_up_input();
	let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

	// prepare display struct
	let mut display = FourLedDisplay::new(led1, led2, led3, led4, button).init();

	loop { // main loop
		if display.should_reset() {
			display.reset();
		} else {
			match serial.read() {
				Ok(value) => display.update(value),
				Err(_) => {},
			}
		}
	}
}
