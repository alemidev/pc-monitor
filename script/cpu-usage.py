#!/usr/bin/env python
import sys
import struct
from time import sleep

import serial
import psutil

def cpu_load_serial_driver(device:str, retry_interval:float=5.0):
	while True:
		try:
			port = serial.Serial(device, baudrate=57600)
			avg_usage_to_serial(port)
		except serial.SerialException as e:
			print(f"[!] Could not connect to device: {str(e)}", file=sys.stderr)
		sleep(retry_interval)

def avg_usage_to_serial(port:serial.Serial):
	port.write(struct.pack("BB", 0, 0))
	port.flush()
	while True:
		# Map float [0:100] to int [0:255], square it to put more values in the lower end, where led is more sensible
		load = [ int(((x/100) **2) * 255) for x in psutil.cpu_percent(0.05, percpu=True) ] # mypy whines but percpu returns a list
		try:
			port.write(struct.pack("BBBBBB", 1, 4, *load))
			port.flush()
		except serial.SerialException as e:
			print(f"[!] Failed writing payload to device: {str(e)}", file=sys.stderr)
			break


if __name__ == "__main__":
	if len(sys.argv) < 2:
		print("[!] No device specified")
		exit(-1)
	
	cpu_load_serial_driver(sys.argv[1])

