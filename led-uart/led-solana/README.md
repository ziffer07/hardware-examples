# Solana LED state program

This is the on-chain component of the LED control example. It stores an
authorized LED state in a Solana account. The host-side
[`led-client`](../led-client/README.md) submits instructions to this program
and, after confirming the state change, forwards it to the Arduino over
serial.

The program is a `no_std` Rust library using Pinocchio. It does not connect
directly to the Arduino; the client is the bridge between Solana and the
hardware.

## Account format

The program expects one writable state account of exactly 41 bytes:

| Bytes | Meaning |
|---|---|
| `0..8` | Account discriminator: ASCII `LEDSTATE` |
| `8..40` | Authority public key |
| `40` | LED state: `0` off or `1` on |

The account must be owned by this program. The authority must sign each
instruction.

## Instructions

Instruction data is a compact byte array:

| Data | Operation |
|---|---|
| `[0]` | Initialize a zeroed state account; sets the authority and starts off |
| `[1, 0]` | Set the state to off |
| `[1, 1]` | Set the state to on |

Initialization requires the state account to be writable and owned by the
program, and the authority to sign. Setting the state also checks the
discriminator and that the signer matches the stored authority.

## Build and deploy to Devnet

Install the Solana CLI and its `cargo build-sbf` command, configure a funded
Devnet wallet, then build from this directory:

```sh
solana config set --url devnet
solana address
cargo build-sbf
```

The CLI wallet used for deployment must have enough Devnet SOL to deploy the
program. The compiled shared object is normally written to
`target/deploy/led_solana.so`. Deploy it with a program keypair:

```sh
solana program deploy target/deploy/led_solana.so
```

Record the program ID printed by the deploy command; pass that ID to
`led-client`. Keep the program keypair safe if you use a custom keypair or
need to manage program upgrades. If your CLI writes build artifacts to a
different directory, use the actual path it reports.

## Security and scope

This is an educational example. The program validates the account owner,
size, discriminator, signer, and requested state, but it does not implement
governance, upgrade controls, or production operational safeguards. Review
and test any changes before deploying them to a network with valuable assets.

## Related documentation

- [`led-client`](../led-client/README.md): submits transactions and controls serial output
- [`blink-code`](../blink-code/README.md): Arduino firmware
- [`led-uart`](../README.md): end-to-end architecture and setup
