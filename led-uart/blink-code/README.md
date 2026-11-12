# Control an Arduino Uno LED over UART with Rust

This bare-metal Rust example waits for a one-byte command over the Arduino
Uno's serial connection:

- Send `1` to turn the built-in LED on.
- Send `0` to turn it off.
- Send `?` to check that the firmware is ready; it responds with `R`.
- After applying `1` or `0`, the firmware echoes the command as an
  acknowledgment. Any other byte is ignored.

The program targets the Uno's ATmega328P microcontroller, runs without the
Rust standard library, and accesses the chip's GPIO and USART registers
directly. The companion Solana client waits for a confirmed transaction and
matching on-chain state before sending an LED command to the board.

## What you need

- An Arduino Uno (ATmega328P) connected by USB
- Rust installed with `rustup`, plus the nightly toolchain
- AVR build tools and `avrdude` for uploading
- A serial terminal such as `minicom`

On Ubuntu or Debian, install the AVR tools and `minicom` with:

```sh
sudo apt-get update
sudo apt-get install binutils gcc-avr avr-libc avrdude minicom
rustup toolchain install nightly
rustup component add llvm-tools
```

This project includes `.cargo/config.toml`, which selects the `avr-none`
target, builds Rust's `core` library for it, and selects the ATmega328P CPU.
Because this AVR target is not distributed as a prebuilt Rust target, Cargo
builds the needed core library from source using nightly Rust.

## Build the firmware

From the repository root, enter this example's directory and build:

```sh
cd led-uart/blink-code
cargo +nightly build --release
```

The generated firmware is:

```text
target/avr-none/release/blink-code.elf
```

`--release` builds an optimized firmware image. The `.elf` file contains
machine code and metadata that `avrdude` can upload to the board.

## Upload it to the Uno

Find the serial device for the board. It is often `/dev/ttyACM0` or
`/dev/ttyUSB0` on Linux. Use the port that appears when you connect the Uno:

```sh
avrdude -p atmega328p -c arduino -P /dev/ttyACM0 -b 115200 -D \
  -U flash:w:target/avr-none/release/blink-code.elf:e
```

Replace `/dev/ttyACM0` with the correct port for your system. Close `minicom`
or any other program using that port before uploading. The `115200` baud rate
above is for the Uno bootloader upload connection; it is separate from the
firmware's 9600-baud UART setting used to receive commands.

## Run the Solana client

Close `minicom` or any other program using the serial port, then run the
companion client from the repository root:

```sh
cargo run --manifest-path led-uart/led-client/Cargo.toml -- \
  <PROGRAM_ID> /dev/ttyACM0
```

Replace `<PROGRAM_ID>` with the deployed Solana program ID, and use the serial
device for your Uno (often `/dev/ttyACM0` or `/dev/ttyUSB0`). The client opens
the port at 9600 baud, waits for a firmware-ready response, and then prompts
for `1`, `0`, or `q`. It sends the command to the Arduino only after Solana
confirms the transaction and the state account matches the requested value.
The firmware acknowledges commands after updating the LED output. A missing
acknowledgment is reported separately because the on-chain state may already
have changed.

The client uses the configured Solana wallet at
`~/.config/solana/id.json` and connects to Devnet. Make sure that wallet has
enough Devnet SOL for transactions. Do not use a wallet with valuable funds
for this example.

## How the code works

### 1. Bare-metal program setup

At the top of `src/main.rs`:

```rust
#![no_std]
#![no_main]
```

`no_std` tells Rust not to link the standard library, which expects operating
system services that this microcontroller does not have. `no_main` disables
the usual Rust program entry point; this firmware provides its own exported
`main` function instead.

The program also supplies a panic handler:

```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

If a panic occurs, the handler stops in an infinite loop. The return type `!`
means the function never returns. `Cargo.toml` sets panic behavior to `abort`
for release builds, avoiding unwinding support that this bare-metal target
does not use.

### 2. Memory-mapped registers and volatile access

The program defines pointers to the ATmega328P's data direction, output, and
USART registers:

```rust
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

const UCSR0A: *mut u8 = 0xC0 as *mut u8;
const UCSR0B: *mut u8 = 0xC1 as *mut u8;
const UCSR0C: *mut u8 = 0xC2 as *mut u8;
const UBRR0L: *mut u8 = 0xC4 as *mut u8;
const UBRR0H: *mut u8 = 0xC5 as *mut u8;
const UDR0: *mut u8 = 0xC6 as *mut u8;
```

These addresses refer to hardware [registers](https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf#G1196529), (Section 30) rather than ordinary variables.
The code uses `read_volatile` and `write_volatile` so the compiler performs
each hardware read or write instead of optimizing it away. Dereferencing
these raw pointers is `unsafe`: using an incorrect address or value can
affect the hardware unexpectedly.

### 3. Select the built-in LED pin

On the Uno, the built-in LED is connected to ATmega328P pin PB5 (digital pin
13). A bit mask selects that pin:

```rust
const LED_PIN: u8 = 1 << 5; // 0b0010_0000
```

Shifting `1` left by five positions sets bit 5, whose decimal value is `32`.
The code configures PB5 as an output while preserving the other Port B pin
settings:

```rust
write_volatile(DDRB, read_volatile(DDRB) | LED_PIN);
```

`DDRB` controls whether Port B pins are inputs or outputs. Setting bit 5 makes
PB5 an output; the bitwise OR leaves the other bits unchanged.

### 4. Set up USART reception and transmission

The USART is the microcontroller's serial hardware. The setup function
configures it for 9600 baud and enables both its receiver and transmitter:

```rust
// 9600 baud at 16 MHz, using the normal-speed setting
write_volatile(UBRR0H, 0);
write_volatile(UBRR0L, 103);

// Enable the USART receiver and transmitter
write_volatile(UCSR0B, (1 << 4) | (1 << 3));

// Asynchronous mode, 8 data bits, no parity, 1 stop bit
write_volatile(UCSR0C, 0x06);
```

For normal-speed asynchronous USART, the baud-rate divisor is calculated as:

```text
UBRR = (CPU frequency / (16 × baud rate)) - 1
     = (16,000,000 / (16 × 9,600)) - 1
     ≈ 103
```

The value is split across `UBRR0H` (high byte) and `UBRR0L` (low byte).
Here the high byte is zero and the low byte is 103. Bit 4 of `UCSR0B` is
`RXEN0`, which enables receiving, and bit 3 is `TXEN0`, which enables
transmitting. Writing `0x06` to `UCSR0C` selects asynchronous mode with 8
data bits, no parity, and one stop bit (8N1).

### 5. Poll for and read a received byte

The `RXC0` flag is bit 7 of `UCSR0A`. It is set when a byte has arrived and is
ready to read:

```rust
fn usart_available() -> bool {
    unsafe { read_volatile(UCSR0A) & (1 << 7) != 0 }
}

fn usart_read() -> u8 {
    unsafe { read_volatile(UDR0) }
}
```

`usart_available` masks off every status bit except bit 7 and checks whether
that bit is nonzero. Once it is set, reading `UDR0` retrieves the received
byte. The main loop only reads `UDR0` after checking that a byte is available.

This is polling: the processor repeatedly checks the status register rather
than waiting for a USART interrupt.

### 6. Interpret commands, change the LED output, and acknowledge

The main function initializes the LED pin and USART, then repeatedly waits for
incoming data:

```rust
loop {
    if usart_available() {
        let command = usart_read();

        if command == b'?' {
            usart_write(b'R');
        } else {
            unsafe {
                if command == b'1' {
                    write_volatile(PORTB, read_volatile(PORTB) | LED_PIN);
                    usart_write(command);
                } else if command == b'0' {
                    write_volatile(PORTB, read_volatile(PORTB) & !LED_PIN);
                    usart_write(command);
                }
            }
        }
    }
}
```

`b'1'` and `b'0'` are the ASCII byte values sent by the terminal. For `1`, the
code sets bit 5 in `PORTB`, turning the LED on. For `0`, it clears just bit 5
with `& !LED_PIN`, leaving the other Port B outputs unchanged. Other received
bytes (including a newline sent after a command) do not change the LED. The
`?`/`R` handshake lets the client detect that the firmware is running after
opening the serial port (which may reset an Uno). The echoed LED command lets
the client detect that firmware processed it, but it does not independently
measure the physical LED.
