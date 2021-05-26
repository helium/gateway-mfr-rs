[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

[actions-badge]: https://github.com/helium/gateway-mfr-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/helium/gateway-mfr-rs/actions/workflows/rust.yml
[discord-badge]: https://img.shields.io/discord/500028886025895936.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/helium

## gateway-mfr-rs

The gateway_mfr application provisions an attached ECC508/ECC608 for use as part of a Helium hotspot.

It does this by configuring and locking the ECC configuration fields and then generating the miner key in slot 0.

The public part of the miner key needs to be captured from the output of this application and supplied as part of the data required to get into the Helium Onboarding Server if gateway add and assert location transactions are to be paid for on behalf of the user.

This applications should be used as part of a manufacturing image that does NOT include the Helium miner software and is solely used for testing and provisioning the built hotspot before setting up the production miner image.

## Usage

1. Build the application into the manufacturing QA/provisioning image. This will
   involve installing rust on the host system and cross compiling for running
   the application on the target hardware. Install
   [cross](https://github.com/rust-embedded/cross) make cross compiling to
   targets easier. 

   For example to compile for Raspbery-Pi's aarch64 architecture:

   ```shell
   cross build --target aarch64-unknown-linux-musl --release
   ```

   The resulting cross compiled binary will be located in `./target/aarch64-unknown-linux-musl/release/gateway_mfr`

2. As part of the provisioning/QA steps start and provision the ECC:

    ```shell
    gateway_mfr provision
    ```

    This will configure the ECC, generate the miner key and output it to stdout.
    Capture this output and collect it and other required information for use by
    the Onboarding Server.

    If you need the extract the onboarding/miner key at a later stage you can
    run:

    ```shell
    gateway_mfr key 0
    ```

3. To verify that the ECC is configured correctly you can run a final test cycle as part of the QA steps:

    ```shell
    gateway_mfr test
    ```

    This will output a json table with all executed ECC tests and their results. This includes a top level `result` key with `pass` or `fail` as the value. 

The ECC is now configured for production use. The production image, including
the Helium miner can be installed and started. If configured correctly the miner
software will use the configured key in slot 0 as the miner key and use the ECC
for secure transaction signing.

