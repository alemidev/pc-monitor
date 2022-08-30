#!/usr/bin/env python
import asyncio
import sys
import struct
import logging
from time import sleep

import serial
import psutil

BAR_MAX_HEIGHT = 255

class State:
	run: bool
	device: str
	baudrate: int
	loop: asyncio.AbstractEventLoop
	packets: asyncio.Queue
	_port: serial.Serial
	_task: asyncio.Task

	def __init__(self, device:str="/dev/ttyUSB0", baudrate=57600):
		self.run = True
		self.baudrate = baudrate
		self.device = device
		self.packets = asyncio.Queue()
		self.port = None

	async def run_port_manager(self):
		while self.run:
			logging.debug("[*] Connecting to device '%s'", self.device)
			try:
				self.port = await self._loop.run_in_executor(None, serial.Serial, self.device, self.baudrate)
				await self._loop.run_in_executor(None, self.port.write, struct.pack("BB", 0, 0))
				while self.run:
					pkt = await self.packets.get()
					logging.debug("[>] Dispatching packet [ %s ]", str(pkt))
					await self._loop.run_in_executor(None, self.port.write, pkt)
			except serial.SerialException as e:
				logging.error("[!] Error operating with device: %s", str(e))
				await asyncio.sleep(30)
			except Exception as e:
				logging.exception("unhandled exception")
				self.run = False
			finally:
				if self.port:
					self.port.close()

	async def display_polling(self):
		rx = 0
		tx = 0
		while self.run:
			cpu_report = await self._loop.run_in_executor(None, psutil.cpu_percent, 0.1, True)
			net = psutil.net_io_counters(pernic=True)
			d_rx = sum(v.bytes_recv for k, v in net.items() if k != "lo")
			d_tx = sum(v.bytes_sent for k, v in net.items() if k != "lo")
			p_rx = min(int(d_rx - rx), 255)
			p_tx = min(int(d_tx - tx), 255)
			w_rx = min(int((d_rx - rx) / 4096), 255)
			w_tx = min(int((d_tx - tx) / 4096), 255)
			await self.packets.put(struct.pack("BBBBBB", 1, 4, *( int(((x/100) **2) * BAR_MAX_HEIGHT) for x in cpu_report )))
			await self.packets.put(struct.pack("BBBB", 2, 2, min(int((d_tx - tx) / 1024), BAR_MAX_HEIGHT), min(int((d_rx - rx) / 1024), 255)))
			await self.packets.put(struct.pack("BBBBBBBBBB", 3, 8, *( int((x/100) * BAR_MAX_HEIGHT) for x in cpu_report ), p_tx, p_rx, w_tx, w_rx))
			rx = d_rx
			tx = d_tx

	# async def cpu_load_leds(self):
	# 	return
	# 	while self.run:
	# 		cpu_report = await self._loop.run_in_executor(None, psutil.cpu_percent, 0.05, True)
	# 		load = [ int(((x/100) **2) * 255) for x in cpu_report ] # mypy whines but percpu returns a list
	# 		logging.info("CPU [%d|%d|%d|%d]", *load)
	# 		await self.packets.put(struct.pack("BBBBBB", 1, 4, *load))

	# async def net_traffic_leds(self):
	# 	return
	# 	rx = 0
	# 	tx = 0
	# 	while self.run:
	# 		net = psutil.net_io_counters(pernic=True)
	# 		d_rx = sum(v.bytes_recv for k, v in net.items() if k != "lo")
	# 		d_tx = sum(v.bytes_sent for k, v in net.items() if k != "lo")
	# 		logging.info("NET [TX %d | %d RX]", d_tx - tx, d_rx - rx)
	# 		await self.packets.put(struct.pack("BBBB", 2, 2, min(d_tx - tx, 255), min(d_rx - rx, 255)))
	# 		rx = d_rx
	# 		tx = d_tx
	# 		await asyncio.sleep(0.01)

	async def run_tasks(self):
		self._loop = asyncio.get_event_loop()
		port_manager = self._loop.create_task(self.run_port_manager())
		# cpu_leds = self._loop.create_task(self.cpu_load_leds())
		# net_leds = self._loop.create_task(self.net_traffic_leds())
		display = self._loop.create_task(self.display_polling())
		# await asyncio.gather(port_manager, cpu_leds, net_leds, display)
		await asyncio.gather(port_manager, display)

if __name__ == "__main__":
	if len(sys.argv) < 2:
		logging.error("[!] No device specified")
		exit(-1)

	logging.basicConfig(level=logging.WARNING)

	state = State(sys.argv[1], baudrate=115200)

	asyncio.run(state.run_tasks())

	
