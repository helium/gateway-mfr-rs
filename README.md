[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[actions-badge]: https://github.com/helium/gateway-mfr-rs/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/helium/gateway-mfr-rs/actions/workflows/ci.yml
[discord-badge]: https://img.shields.io/discord/500028886025895936.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/helium

## gateway-mfr-rs

The gateway_mfr application provisions an security part (like the ECC508/ECC608)
for use as part of a Helium hotspot, and provides utilities for testing and
benchmarking the addressed part.

In the ECC case, it does provisioning by configuring and locking the ECC
configuration fields and then generating the miner key in the slot identified in
the device URL (default slot 0).

Other security parts may be provisioned in different ways or may have been
locked down before hotspot integration.

The public part of the miner key needs to be captured from the output of this
application and supplied as part of the data required to get into the Helium
Onboarding Server if gateway add and assert location transactions are to be paid
for on behalf of the user.

This applications should be used as part of a manufacturing image that does NOT
include the Helium miner software and is solely used for testing and
provisioning the built hotspot before setting up the production miner image.

## Addressing

The security device to provision or test is addressed using a `--device` option.
In the ECC case, for exmaple this URL could be `ecc://i2c-1:96?slot=0` to
address the `/dev/i2c-1` linux device, using the bus address`96` and slot `0` on
the ECC device. This is also the default URL for the application, and must be
provided for ECC parts with a different bus address or slot.

If you are passing an additional command such as those decribed in the [usage section](#usage) below those commands need to come after the device address. For example:

```
gateway_mfr --device ecc://i2c-1:96?slot=0 key
```

Each security part will have it's own URL scheme and host/path arguments to
address the specific system and entry used for key material and provisioning.

## Usage

1. Using the application can be done in two ways;

   - Download a pre-built binary from the
     [releases](https://github.com/helium/gateway-mfr-rs/releases/latest) page.
     Note that the `unknown` target systems are all `ecc608` based targets. To
     find the right target binary for a given platform, look at the [supported
     targets](https://github.com/helium/gateway-rs#supported-targets) for the
     maker name and associated target.

   - Build the application. This will involve [installing
     rust](https://www.rust-lang.org/learn/get-started) on the host system and
     cross compiling for running the application on the target hardware.
     [Install cross](https://github.com/rust-embedded/cross) make cross
     compiling to targets easier. Also install
     [cargo-make](https://github.com/sagiegurari/cargo-make) to set up for
     packaging and specific profile support.

     For example to compile for Raspbery-Pi's aarch64 architecture:

     ```shell
     cargo make --profile aarch64-unknown-linux-musl --release
     ```

     The resulting cross compiled binary will be located in `./target/ aarch64-unknown-linux-musl/release/gateway_mfr`

     **NOTE**: For some profiles the resulting target will not be in the profile
     name but under the target system triplet that was used to build the target.
     For example, the `x86_64-tpm-debian-gnu` uses the `x86_64-unkown-linux-gnu`
     target but a custom Docker file to build using Debian since that is where
     `tpm` is supported.

2. As part of the provisioning/QA steps start and provision the security part:

   ```shell
   gateway_mfr provision
   ```

   This will configure the security part, generate the miner key and output it
   to stdout. Capture this output and collect it and other required information
   for use by the Onboarding Server.

   If you need the extract the onboarding/miner key at a later stage you can
   run:

   ```shell
   gateway_mfr key
   ```

   **NOTE**: Do **not** include this application in the final image as it is not
   used as part of normal hotspot operations.

3. To verify that the security part is configured correctly you can run a final
   test cycle as part of the QA steps:

   ```shell
   gateway_mfr test
   ```

   This will output a json table with all executed tests for the security part
   and their results. This includes a top level `result` key with `pass` or
   `fail` as the value.

   Tests are specific for each security part and are intended to test that the
   security part is locked, and that signing and ecdh opterations function

4. To benchmark a security part as part of integration:

   ```shell
   gateway_mfr bench
   ```

   This will run a number of signing iterations (default 100) and report the
   average signing time and the number of signing operations per second.

   Helium Hotspots using a full miner will need 6-7 or better signing operations
   per second while light/dataonly hotspots should be able to operate with
   around 3-5 operations per second (this number needs to be confirmed).

The security part is now configured for production use. The production image,
including the Helium miner can be installed and started. If configured correctly
the miner software will use the configured key in slot 0 as the miner key and
use the security part for secured transaction signing.

The full suite of options can be found by running the help command:

```shell
gateway_mfr help
```

This will give you an output like the following where you can find all of the options listed:

```
gateway_mfr 0.3.2
Gateway Manufacturing 

USAGE:
    gateway_mfr [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --device <device>    The security device to use [default: ecc://i2c-1]

SUBCOMMANDS:
    bench        Run a benchmark test
    config       Gets the zone, slot or key config for a given ecc slot
    help         Prints this message or the help of the given subcommand(s)
    info         Get ecc chip information
    key          Prints public key information for a given slot
    provision    Configures the ECC for gateway/miner use. This includes configuring slot and key configs for †he
                 given slot, locking the data and config zone and generating an ecc compact key in the configured
                 slot
    test         Read the slot configuration for a given slot
```
