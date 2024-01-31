# hestia-sat

Interface for the Hestia payload which runs on the WS-1 satellite on a BeagleBone Black (BBB) payload computer.

Written in Rust, conforming to the [Cube-OS interfaces](https://github.com/Cube-OS).

## Cross-compiling for BeagleBone

To build the code, just run `cargo build`.

It will fail until you have correctly set up the cross-compiler for the BBB.

### On Mac

* Add the target for your Rust environment with: 
  `rustup target add arm-unknown-linux-gnueabihf`
* Install the ARM Linux cross-compiler `arm-unknown-linux-gnueabihf` from
  [osx-arm-linux-toolchains](https://github.com/thinkski/osx-arm-linux-toolchains), and symlink
  the `bin/arm-unknown-linux-gnueabihf-gcc` command to your PATH as `arm-linux-gcc`. (It needs to match the
  linker configuration in `.cargo/config.toml`.)
* Install the BBB toolchain from [Cube-OS toolchains](https://github.com/Cube-OS/toolchains/), and add its absolute
  path to the BBB_TOOLCHAIN in your environment. (The relevant library path is added for the linker by `build.rs`,
  which will fail if something is wrong here.)

### On Linux

* Add the target for your Rust environment with:
  `rustup target add arm-unknown-linux-gnueabihf`
* Install the BBB toolchain from [Cube-OS toolchains](https://github.com/Cube-OS/toolchains/), and add its absolute
  path to the BBB_TOOLCHAIN in your environment. (The relevant library path is added for the linker by `build.rs`,
  which will fail if something is wrong here.)
* Either add `$BBB_TOOLCHAIN/usr/bin/` to your PATH, or symlink `$BBB_TOOLCHAIN/usr/bin/arm-linux-gcc` to a directory
  on your PATH, like `~/bin`. This will be used as the linker, as configured in `.cargo/config.toml`.

### With Docker

The CUAVA team have provided a Docker configuration which can be used for compiling for both the
satellite primary computer and the payload computer.

You can get it from [Cube-OS/cubeos-dev](https://github.com/Cube-OS/cubeos-dev).

To get it working on Mac, you need to:

* Edit the dockerfile to disable the 32-bit library installation
* Build the image: `docker build -t cubeos-dev .`
* Connect your SSH agent when starting the container like this:

```sh
docker run -it -v "$PWD":/usr/cubeos \
    -v /run/host-services/ssh-auth.sock:/run/host-services/ssh-auth.sock:ro \
    -e SSH_AUTH_SOCK="/run/host-services/ssh-auth.sock" \
    -w /usr/cubeos cubeos-dev bash
```

There is also a script `run-docker.sh` that does this.

## Running

### Environment variables

The Hestia binaries can be configured by setting the following environment variables.

| variable            | default | description                                                                      |
|---------------------|---------|----------------------------------------------------------------------------------|
| `UTS_LOG_PATH`      |         | Log file directory                                                               |
| `UTS_DOWNLOAD_PATH` |         | Path to output compressed logs for downloading                                   |
| `UTS_COMPRESS_LOGS` | `false` | Use gzip compression when writing logs                                           |
| `UTS_I2C_BUS`       | `1,2`   | List of active I2C bus numbers                                                   |
| `UTS_BOARD_VERSION` | `V2_2`  | Board version, used for switching some address settings [`V1_1`, `V2_0`, `V2_2`] |
| `UTS_LOG_INTERVAL`  | `5`     | Duration between logging output in seconds                                       |
| `UTS_PROGRAM_FILE`  |         | Location of program config file, e.g. `/home/debian/uts/uts-programs.toml`       |
| `UTS_HTTP_PORT`     | `5000`  | Port used for HTTP dashboard                                                     |
| `UTS_CORS_ENABLE`   | `false` | Enable CORS for remote API access                                                |
| `UTS_INSTALL_PATH`  |         | Installation directory, used for `uts-update`                                    |
| `UTS_SYSLOG`        | `false` | Send error logging to syslog instead of console                                  |
