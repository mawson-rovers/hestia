# hestia-fw

Firmware for MSP430F2618 microcontroller on the Hestia board.

The firmware is responsible for:

* Reading temperature sensors on the board
* Control of the heater
* I2C interface so the payload computer can retrieve the temperatures and control the heater

Note: to develop and test the firmware with the board, **you will need an MSP430 programmer**, like the
MSP-FET.

### Building in Code Composer Studio

The easiest way to build the project is in Code Composer Studio. It has been tested with
CCS 11.x and 12.x.

* After checking out the code, open up the .project file in the source code.
* Ensure you have selected the correct device, MSP430F2618, and try to build the firmware.
* If it all compiles okay, run the flash or debug tools with your MSP programmer connected
to update the firmware on the board.

### Building in CLion

CLion is a nicer IDE to work in than CCS, but requires more setup.

First, edit `toolchains/toolchain-msp430-gcc-ti.cmake` to set the tool locations to match
your machine. You'll need to find and set correct paths for:

* `TOOLCHAINS_PATH` - the `toolchains/` path inside this module
* `MSP430_TI_COMPILER_FOLDER` - where the TI GCC compiler lives
* `FLASHER_PATH` - the path to TI MSPFlasher
* `MSP430_TI_HEADERS_FOLDER` - where the TI headers are, e.g. msp430f2618.h

The TI GCC compiler and MSPFlasher may have been installed when you installed Code Composer
Studio, but if not, you will need to download them from the TI website and install manually.

When you open the CLion project, you'll be prompted to edit the CMake configuration.
You need to add the following toolchain setting under "CMake Options":

`-DCMAKE_TOOLCHAIN_FILE=../toolchains/toolchain-msp430-gcc-ti.cmake`

You can also do this later in the Settings inside CLion, if you forget to do it at the
initial prompt.

Once CLion refreshes the CMake build settings, you should have a set of build targets
configured, including:

* `hestia-msp430f2618.elf` - build the binary
* `flash-hestia-msp430f2618` - flash the board using MSPFlasher 

Running these should correctly build the project and flash it to the board if a
programmer is connected.
