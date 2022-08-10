#!/usr/bin/env python
from time import sleep
import struct
import serial

import psutil

def avg_usage_to_serial(dev:str):
	port = serial.Serial(dev, baudrate=57600)
	while True:
		# Map float [0:100] to int [0:255], square it to put more values in the lower end, where led is more sensible
		load = [ int(((x/100) **2) * 255) for x in psutil.cpu_percent(0.1, percpu=True) ] # mypy whines but percpu returns a list
		port.write(struct.pack("BBBB", *load))
		port.flush()

if __name__ == "__main__":
	import sys
	if len(sys.argv) < 2:
		print("[!] No device specified")
	else:
		avg_usage_to_serial(sys.argv[1])

