# Solana-to-Arduino LED client

This host-side Rust application connects the Solana program to the Arduino
Uno firmware. It sends a transaction to set the on-chain LED state, waits for
confirmation, checks the account state, and only then sends the matching
command over the Uno's serial connection.

The client uses Devnet at `https://api.devnet.solana.com`, loads the payer
wallet from `~/.config/solana/id.json`, and communicates with the firmware at
9600 baud.

## Requirements

- Rust and Cargo
- A deployed `led-solana` program on Devnet
- A Devnet wallet at `~/.config/solana/id.json` with enough SOL for
  transaction fees and account rent
- The updated firmware flashed to an Arduino Uno
- The Uno's serial device path, such as `/dev/ttyACM0` or `/dev/ttyUSB0`

Do not use a wallet with valuable funds for this example. On Linux, your user
may need permission to access the serial device. Add the user to the
appropriate serial-port group (commonly `dialout`) and reconnect or log in
again if access is denied.

## Run

From this directory, run:

```sh
cargo run -- <PROGRAM_ID> <SERIAL_PORT>
```

For example, replace the placeholders with your deployed program ID and Uno
serial port:

```sh
cargo run -- <PROGRAM_ID> /dev/ttyUSB0
```

The client opens that port itself; close `minicom` and any other serial
monitor first.

The client waits for the firmware's `?`/`R` readiness handshake. It then
prompts for:

- `1` to request LED on
- `0` to request LED off
- `q` to quit

For `1` or `0`, the client waits for Solana confirmation and verifies the
state account before sending the byte to the Arduino. The firmware echoes
the command after updating its output, and the client reports whether it
received that acknowledgment. A serial failure after transaction
confirmation does not undo the on-chain change.

## State account

The client creates the state account on first use and initializes it with
the payer as authority. It saves the generated account keypair to
`led-state-keypair.json` in the current working directory. Keep this file
between runs to reuse the same state account, and do not share or commit it.

If you delete the keypair file, the client generates a new account address.
The existing on-chain account remains on Devnet and its rent is not
recovered. The account's authority must match the wallet loaded by the
client.

## Related documentation

- [`led-solana`](../led-solana/README.md): the on-chain program
- [`blink-code`](../blink-code/README.md): building and flashing the Uno
- [`led-uart`](../README.md): end-to-end architecture and setup
