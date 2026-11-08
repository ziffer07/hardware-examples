# LED blink on an Arduino Uno with Rust

This example targets an Arduino Uno with an ATmega328P. The repository's
top-level [README](../README.md) links to the Arduino Uno and ATmega328P
datasheets.

## Prerequisites

Install Rust with `rustup`, then install the nightly toolchain:

```sh
rustup toolchain install nightly
```

This project builds the tier 3 AVR target from source with Cargo's nightly
`build-std` feature rather than installing a prebuilt target with
`rustup target add`. On Ubuntu, the AVR tools used by the build and upload
workflow can be installed with:

```sh
sudo apt-get install binutils gcc-avr avr-libc avrdude
rustup component add llvm-tools
```

`avrdude` uploads firmware to the board; it does not compile the Rust program.
Tool requirements can vary with the Rust toolchain and upload method.

## Project setup

Create a binary crate and select nightly for it:

```sh
cargo new <project-name> --bin
cd <project-name>
rustup override set nightly
mkdir .cargo
```

Create `.cargo/config.toml`:

```toml
[build]
target = "avr-none"

[unstable]
build-std = ["core"]

[target.avr-none]
rustflags = ["-C", "target-cpu=atmega328p"]
```

The target setting selects AVR, `build-std` builds the `core` library for the
target, and `target-cpu` selects the ATmega328P instruction set.

Build an optimized firmware image with:

```sh
cargo build --release
```

The ELF output is under `target/avr-none/release/`, named after the crate (for
example, `target/avr-none/release/led_project.elf`). Release builds are
optimized and are generally preferable on a microcontroller with limited
flash and RAM.

## Bare-metal entry point and panic handler

A `no_std` program does not link Rust's standard library, and `no_main` disables
Cargo's usual Rust `main` wrapper. The program supplies its own entry-point
symbol. In the 2024 edition, `no_mangle` is an unsafe attribute:

```rust
#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}
```

`no_mangle` keeps the exported symbol named `main`; it does not place the
function at a hardware address. The AVR startup and linker setup must use the
entry-point convention expected by the target.

`no_std` programs also need a panic handler. This one stops in an infinite
loop:

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
```

Set panic behavior in `Cargo.toml` to avoid unwinding, which is not supported
for this bare-metal setup:

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

## Blink the built-in LED

On an Arduino Uno, the built-in LED is connected to digital pin 13, which is
PB5 on the ATmega328P. `DDRB` controls the direction of Port B pins; setting
bit 5 makes PB5 an output. `PORTB` controls the output level; setting bit 5
turns the LED on.

The value `32` is `2^5`, so it sets bit 5. This example uses simple assignments
rather than a bitwise XOR operation:

```rust
#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::read_volatile};

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        *DDRB = 32;
    }

    loop {
        unsafe {
            *PORTB = 32;
            delay(100_000);
            *PORTB = 0;
            delay(100_000);
        }
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        unsafe {
            read_volatile(DDRB as *const u8);
        }
    }
}
```

These assignments replace the entire register value: `*DDRB = 32` clears the
direction bits for the other Port B pins, and `*PORTB = 0` drives all Port B
output bits low. That is suitable only when those other pins are not being used.
If you need to preserve their state, set and clear just PB5 with a bit mask.

The register pointers refer to memory-mapped hardware registers, so accessing
them is `unsafe`. For reliable memory-mapped I/O, writes should also be
volatile (for example, with `core::ptr::write_volatile`); a normal raw-pointer
write is not guaranteed to be volatile. The delay here is an uncalibrated busy
loop, not a precise time delay. Use a hardware timer when accurate timing is
needed.

Rust does not make unsafe hardware access safe by itself, nor is it inherently
faster than C. The benefit is that unsafe operations can be kept in small,
explicit sections and exposed through safe interfaces.
