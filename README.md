# PiCrust

A basic system to tinker with ARM and the Raspberry Pi - 99% rust.

## Prerequisites

You need a working Rust (edition 2021) development environment (rustc, rustup, cargo should all be working).
Install the rust pi's target triple:
```
> rustup target add armv7a-none-eabi
```

To test the kernel on an emulated pi you can use QEMU, you will need the qemu-system-arm executable. You should also make sure you have the correct ARM machines for the RPi installed: 

```
> qemu-system-arm -machine help   
...
raspi2b              Raspberry Pi 2B (revision 1.1)
...
```

## What's where

`src/main.rs` contains the entry point written in inline asm, as well as the hand-off to rust code.
`src/peripherals` module that has various structs and fns to interface with the various Rapsberry Pi peripherals via memory mapped i/o (`mmio`) and the "property mailbox".
`link.x` a linker script that defines the structure of the executable blob, and whither the RPi VC firmware will load the various binary bits. 
`src/monitor.rs` RuzMon. Like WozMon, but instead of being 254 bytes of decade-defining, finely tuned 6502 assembler, it's written in Rust.

## Using the Monitor

* Typing a hex address followed by Enter will print the 8 bytes starting at that memory location. For example, typing `8010` and enter will print the first 8 bytes of the inline assembly in `main.rs`. Neat!
* Typing `R ` followed by a hex address will branch execution to that memory location. Try `R 8010` and then try it again and again and...

## Future plans

* RuzMon: 
  * [] Writing to memory `8000: BA DD F0 0F`
  * [] using ranges like `8000.8100` 
  * [] disassembly

* Framebuffer
  * [] text output of RuzMon

* USB / HID to get at keyboard input

## Building, Testing, Running

```
cargo build --release
```

will put the kernel binary into `target/armv7a-none-eabi/release/picrust`

```
cargo run --release
```

will start qemu (make sure its on the path) with the kernel loaded into the VMs RAM.

To run the system on a real pi, it might be possible -- but as of yet untested -- to simply

1. take a fresh sd card, 
2. write a raspbian 32 bit image to it, using the raspberry pi imager tool
3. replace the `kernel.img` file in `boot/` with the `picrust` binary.
4. stick the card into a rpi 2/3/4 and see what happens. 

As of now, only the serial interface via UART0 pins _should_ be working. Framebuffer output is currently not in use.
