# Amber - Microcontroller emulation tool
Have you ever got tired for testing your MCU code on real devices?
Having to upload it through a cheap flash programmer with unreliable connection, and looked at the blinking light in confusion, not knowing what's going on?
Have you messed up the voltages one time and burned your MCU accidentally?

Well, I have done all of that, and I've made this emulation tool so that no more MCUs would suffer from my inability to read the specification manual.
This tool is designed to emulate not only the MCU itself, but also the surrounding components, so that you can test your code in something closer to the real world circuit boards.
Of course, this is still only a testing tool, and nothing beats testing on the real hardware, but I found it quite useful for checking the logic of the code.

This is still very much Work In Progress, so not much is supported yet.
Currently the only supported MCU is Atmega2560 with AVR architecture.
Out of external components there is only a LED light and a UART tranciever, which allows communication with the terminal.

## Features
- Emulation of multile MCUs communicating with each other. The emulation uses multithreaded architecture, so it can efficiently emulate multiple devices running at the same time.
- Real time performance, sometimes even faster than real time! The MCU routines have been heavily optimized, and thanks to Rust's performance that really makes a big difference.
- Interaction with the running emulation, through virtual UART connection.
- VCD output, even for internal MCU components.
- Input files are literally compiled raw memory dumps, the ones you usually would upload to the MCU.
  This has actually been my issue with other MCU emulators, since they force you to use their internal compilers, while I use a custom compilation toolchain, so I would prefer to use compiled code as input, not C code.
