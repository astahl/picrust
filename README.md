# PiCrust

A basic system to tinker with ARM and the Raspberry Pi - 99.9% rust.

## Inspired by...

* [Building an Operating System for the Raspberry Pi](https://jsandler18.github.io) by Jake Sandler.
* [Writing a “bare metal” operating system for Raspberry Pi 4](https://www.rpi4os.com) by Adam Greenwood-Byrne.

Thank you!

## Prerequisites

You need a working Rust (edition 2021) development environment (rustc, rustup, cargo should all be working).
Install the rust pi's target triple:
```
rustup target add armv7a-none-eabi
```

To test the kernel on an emulated pi you can use QEMU, you will need the qemu-system-arm executable. You should also make sure you have the correct ARM machines for the RPi installed: 

```
qemu-system-arm -machine help   
```

should print (among others)

```
...
raspi2b              Raspberry Pi 2B (revision 1.1)
...
```

## What's where

* `src/main.rs` contains the entry point written in inline asm, as well as the hand-off to rust code.
* `src/peripherals` module that has various structs and fns to interface with the various Rapsberry Pi peripherals via memory mapped i/o (`mmio`) and the "property mailbox".
* `link.x`, `link64.x` a linker script that defines the structure of the executable blob, and whither the RPi VC firmware will load the various binary bits.
* `src/monitor.rs` RustMon. Like WozMon, but instead of being 254 bytes of decade-defining, finely tuned 6502 assembler, it's written in Rust.

## Using the Monitor

* Typing a hex address followed by Enter will print the 8 bytes starting at that memory location. For example, typing `8010` and enter will print the first 8 bytes of the inline assembly in `main.rs`. Neat!
* Typing `R ` followed by a hex address will branch execution to that memory location. Try `R 8010` and then try it again and again and...

## Future plans

* RustMon: 
  * [ ] Writing to memory `8000: BA DD F0 0F`
  * [ ] using ranges like `8000.8100` 
  * [ ] disassembly
* Framebuffer
  * [ ] Put a test image onto the framebuffer that indicates if more than one core is being started (if not, we might need to wake them up manually, or use the old_kernel=1 config)
  * [x] a simple text mode, using some character ROM dump, e.g. from the PET because it looks nice.
    * [x] how to put a binary file into the kernel image, linker perhaps?
  * [ ] text output of RustMon
* USB / HID to get at keyboard input, probably Interrupt handling, oh my.

## Building, Testing, Running

```
cargo build --release
```

will put the kernel binary into `target/armv7a-none-eabi/release/picrust`

```
cargo run32
```

will start qemu (make sure its on the path) with the kernel loaded into the VMs RAM.

To run the system on a real pi,

1. take a fresh sd card, 
2. write a raspbian 64 bit image to it, using the raspberry pi imager tool
3. Run the command `cargo img64`, which is an alias for `cargo objcopy --release --target aarch64_unknown_none -- -O binary kernel8.img`
4. replace the `kernel8.img` file on your sd card with the `kernel8.img` binary generated in the previous step. Remove any other .img files.
5. stick the card into a rpi 3/4 and see what happens. 
6. If it doesn't work, fiddle with the [`config.txt` file](https://www.raspberrypi.com/documentation/computers/config_txt.html) and retry. You might need to set  various parameters, for example `arm_64bit=1` and also the `video=` setting in the [`cmdline.txt` file](https://www.raspberrypi.com/documentation/computers/configuration.html#the-kernel-command-line).

As of now, only a simple test pattern is output via a 1280x720 framebuffer on the hdmi.
