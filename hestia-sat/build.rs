use std::env;
use std::path::Path;

// set this in your shell profile, e.g.
// export BBB_TOOLCHAIN_LIB=$HOME/opt/toolchains/bbb-toolchain-0.1
const TOOLCHAIN_ENV_VAR: &str = "BBB_TOOLCHAIN";

// location of libraries within the toolchain - this will be added to the linker path
const USR_LIB: &'static str = "arm-buildroot-linux-gnueabihf/sysroot/usr/lib";

// we check for the existence of this file to make sure deps are there
const TEST_FILE: &str = "libsqlite3.so";

fn main() {
    let toolchain_path = match env::var(TOOLCHAIN_ENV_VAR) {
        Ok(v) => v,
        Err(e) => panic!(
            "${} not set; install from https://github.com/Cube-OS/toolchains/: {}",
            TOOLCHAIN_ENV_VAR, e)
    };
    let lib_path = Path::new(&toolchain_path).join(USR_LIB);
    if !lib_path.exists() {
        panic!("Library path not found: {}", lib_path.display());
    }

    if !lib_path.join(TEST_FILE).exists() {
        panic!("Required library {} does not exist, check toolchain installation: {}",
               TEST_FILE, lib_path.join(TEST_FILE).display())
    }

    println!("cargo:rustc-link-search={}", lib_path.display());
}
