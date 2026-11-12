# Solana-controlled Arduino LED

This directory contains a small end-to-end example that connects a Solana
program to an Arduino Uno LED. The components work together at runtime, but
they are separate programs built and deployed to different environments:

1. **`led-solana`** runs on Solana and stores the authorized LED state.
2. **`led-client`** runs on a computer. It sends a transaction, waits for
   confirmation, verifies the on-chain state, then forwards the command over
   the serial port.
3. **`blink-code`** runs on the Arduino Uno. It receives the serial command
   and switches the built-in LED.

The Arduino does not connect to Solana directly. The host client is the
bridge. As a result, the hardware changes only when the client is running
and able to communicate with both Devnet and the serial device.

## End-to-end flow

```text
You enter 1 or 0
        |
        v
led-client sends transaction to Devnet
        |
        v
Solana program validates signer and updates LED state account
        |
        v
Client waits for confirmation and checks the account state
        |
        v
Client sends ASCII '1' or '0' to Arduino over UART
        |
        v
Firmware changes the built-in LED and echoes the command
```

The serial link uses **9600 baud, 8N1**. On startup, the client sends `?` and
waits for `R` from the firmware. After a successful on-chain update and
readback, the client sends `1` or `0`; the firmware echoes that byte after
changing the GPIO output. The echo confirms the firmware processed the
command, not that the LED's physical output was independently measured.

If the transaction confirms but serial communication fails, the on-chain
state remains updated; the client reports the hardware error separately.

## Setup and run

### 1. Build and deploy the Solana program

Follow [`led-solana/README.md`](./led-solana/README.md) to build and deploy
the program to Devnet. Configure the Solana CLI to use a funded Devnet
wallet, and save the deployed program ID.

### 2. Build and flash the firmware

Follow [`blink-code/README.md`](./blink-code/README.md) to build the AVR
firmware and flash it to an Arduino Uno. Note the serial device path for the
board, commonly `/dev/ttyACM0` or `/dev/ttyUSB0` on Linux.

### 3. Run the client

The client uses the wallet at `~/.config/solana/id.json` and connects to
Devnet. That wallet needs Devnet SOL for fees and, on first use, account
creation rent. It must be the same wallet throughout this example because it
is stored as the state account's authority.

Close `minicom` and any other program using the Arduino serial port. From
`led-uart/led-client`, run:

```sh
cargo run -- <PROGRAM_ID> <SERIAL_PORT>
```

For example:

```sh
cargo run -- <PROGRAM_ID> /dev/ttyUSB0
```

Replace `<PROGRAM_ID>` with the actual deployed program ID, and use the
actual serial device path for your board. Enter `1` to turn the LED on, `0`
to turn it off, or `q` to quit.

On the first client run, it creates and initializes an on-chain state
account. The generated keypair is saved as
`led-state-keypair.json` in the current working directory; run the client
from `led-uart/led-client` and keep that file to reuse the same account. Do
not share or commit the keypair.

## Component guides

- [`blink-code/README.md`](./blink-code/README.md): AVR firmware build, flash, and UART behavior
- [`led-client/README.md`](./led-client/README.md): client configuration, state account, and serial behavior
- [`led-solana/README.md`](./led-solana/README.md): account layout, instructions, build, and deployment
