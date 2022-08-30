use arduino_hal::{simple_pwm::*, port::{Pin, mode::PwmOutput}, hal::port::{PD3, PB1, PB2, PB3}};

// TODO can I make it a generic "Pin" and use a slice?
pub struct FourLedDisplay {
	led1: Pin<PwmOutput<Timer2Pwm>, PB3>,
	led2: Pin<PwmOutput<Timer1Pwm>, PB2>,
	led3: Pin<PwmOutput<Timer1Pwm>, PB1>,
	led4: Pin<PwmOutput<Timer2Pwm>, PD3>,
}

impl FourLedDisplay {
	pub fn new(
		mut led1: Pin<PwmOutput<Timer2Pwm>, PB3>,
		mut led2: Pin<PwmOutput<Timer1Pwm>, PB2>,
		mut led3: Pin<PwmOutput<Timer1Pwm>, PB1>,
		mut led4: Pin<PwmOutput<Timer2Pwm>, PD3>,
	) -> Self {
		led1.enable();
		led2.enable();
		led3.enable();
		led4.enable();
		FourLedDisplay{
			led1, led2, led3, led4,
		}
	}

	pub fn set(&mut self, index:u8, value:u8) -> &mut Self {
		match index {
			1 => self.led1.set_duty(value),
			2 => self.led2.set_duty(value),
			3 => self.led3.set_duty(value),
			4 => self.led4.set_duty(value),
			_ => {},
		}
		self
	}

	pub fn set_all(&mut self, value:u8) -> &mut Self {
		self.led1.set_duty(value);
		self.led2.set_duty(value);
		self.led3.set_duty(value);
		self.led4.set_duty(value);
		self
	}

	pub fn set_many(&mut self, first:u8, second:u8, third:u8, fourth:u8) -> &mut Self {
		self.led1.set_duty(first);
		self.led2.set_duty(second);
		self.led3.set_duty(third);
		self.led4.set_duty(fourth);
		self
	}
}
