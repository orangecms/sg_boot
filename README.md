# Sophgo Boot Tool

This tool talks to CVITEK/Sophgo SoC mask ROMs, supporting:

- [x] [CV180xB](https://en.sophgo.com/sophon-u/product/introduce/cv180xb.html)
  * dual-core C906, with different clock speeds and cache sizes
- [x] [SG200x](https://en.sophgo.com/sophon-u/product/introduce/sg200x.html)
  * similar to CV180xB
  * option to switch the main core to be Arm

## Boards

- [Milk-V Duo](https://milkv.io/duo)
  * Duo: CV1800B
  * Duo S: SG2000
  * Duo 256M: SG2002
- Sipeed
  * [LicheeRV
    Nano (SG2002)](https://wiki.sipeed.com/hardware/en/lichee/RV_Nano/1_intro.html)
  * [NanoKVM Cube (SG2002)](https://wiki.sipeed.com/hardware/en/kvm/NanoKVM/introduction.html)

## Building

Have a Rust toolchain installed with Cargo.

```sh
cargo build --release
```

## Running

To run a given flat binary, e.g., `oreboot_sg200x.bin` on the main core:

```sh
cargo run --release -- run oreboot_sg200x.bin
```

For more options, see the help:

```sh
cargo run --release -- -h
```

## Development

This tool is written in Rust :crab: using well-known libraries from the Rust
community for connecting via serial, defining the CLI, and checksums, including:

- ![serialport-rs](https://avatars.githubusercontent.com/u/32803384?s=24&v=4)
  [serialport-rs](https://github.com/serialport/serialport-rs)
- ![clap](https://avatars.githubusercontent.com/u/39927937?s=24&v=4)
  [clap](https://docs.rs/clap)
