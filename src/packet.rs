const PACKET_BUFFER : usize = 32;

#[derive(Copy, Clone)]
pub enum PacketId {
	Invalid = 0xFF,
	Reset = 0x00,
	SetLedsPacket = 0x01,
	NetStatePacket = 0x02,
}

impl From<u8> for PacketId {
	fn from(x: u8) -> Self {
		match x {
			0 => PacketId::Reset,
			1 => PacketId::SetLedsPacket,
			2 => PacketId::NetStatePacket,
			_ => PacketId::Invalid,
		}
	}
}

pub struct Packet<'a> {
	pub id: PacketId,
	pub payload: Option<&'a[u8]>,
}

enum PacketBuilderStep {
	ID,
	SIZE,
	PAYLOAD,
}

pub struct PacketBuilder {
	step: PacketBuilderStep,
	id: PacketId,
	size: u8,
	index: usize,
	buffer: [u8; PACKET_BUFFER],
}

impl PacketBuilder {
	pub fn new() -> Self {
		PacketBuilder {
			step: PacketBuilderStep::ID,
			id: PacketId::Invalid,
			size: 0,
			index: 0,
			buffer: [0;PACKET_BUFFER],
		}
	}

	pub fn update(&mut self, byte:u8) -> Option<Packet> {
		let mut ret = None;
		match self.step {
			PacketBuilderStep::ID => {
				let id = PacketId::from(byte);
				match id {
					PacketId::SetLedsPacket | PacketId::Reset | PacketId::NetStatePacket => {
						self.id = id;
						self.step = PacketBuilderStep::SIZE;
					},
					_ => {
						// TODO log it somehow?
					},
				}
			},
			PacketBuilderStep::SIZE => {
				if byte as usize > PACKET_BUFFER {
					self.step = PacketBuilderStep::ID;
					// TODO log it somehow?
				} else if byte == 0 {
					// packet without payload
					ret = Some(Packet{id: self.id, payload: None}); // jank zero size slice
					self.step = PacketBuilderStep::ID;
				} else {
					self.size = byte;
					self.index = 0;
					self.step = PacketBuilderStep::PAYLOAD;
				}
			},
			PacketBuilderStep::PAYLOAD => {
				self.buffer[self.index] = byte;
				self.index += 1;
				if self.index >= self.size as usize {
					ret = Some(Packet{id: self.id, payload: Some(&self.buffer[0..self.size as usize])});
					self.step = PacketBuilderStep::ID;
				}
			},
		}
		ret
	}
}


