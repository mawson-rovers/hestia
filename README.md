# Hestia

Hestia is a cubesat circuit board developed by Mawson Rovers in support of
[UTS Resilient Space Computing initiative][uts]. It is designed to provide
temperature control and monitoring for a demonstration heatsink payload which
will be launched aboard the [Waratah Seed satellite][ws] on a SpaceX rocket
in early 2024.

[ws]: https://www.waratahseed.space
[uts]: https://web-tools.uts.edu.au/projects/detail.cfm?ProjectId=PRO22-15177

Hestia was the Greek goddess of home and hearth, and similar to that, our Hestia
board provides a home and warmth to the UTS heatsink.

## Project structure

This project has four primary modules:

* **hestia-pcb** - the circuit board design for the cubesat, as a KiCAD 7 project
* **hestia-fw** - the firmware for the MSP430 microcontroller on the Hestia board,
    responsible for controlling the heater and measuring temperature, written in C
* **hestia-sat** - the payload control software to be run on a BeagleBone Black
    on board the Waratah Seed satellite, written in Rust
* **hestia-dash** - a web dashboard for monitoring and operating the board, used
    primarily for testing and written in Python

All the software projects require customised build environments, and more details
can be found in their respective READMEs.

The project also has a **scripts/** folder which is used for automated builds on
GitHub, and **ws-1/** folder for interface information from our host Waratah Seed
satellite.

## Project team

The project is led by [Dr Nick Bennett][nick], Senior Lecturer at UTS, with
software and electronics development managed by [Matt Ryall][matt] from Mawson
Rovers. [John Dowdell][jd] and [Scott Fraser][scott] developed the Hestia
components for the mission.

[nick]: https://profiles.uts.edu.au/Nicholas.Bennett
[matt]: https://github.com/mryall-mawson
[scott]: https://github.com/Scouttman
[jd]: https://github.com/JDMawson

The project was funded by [SmartSat CRC](https://smartsatcrc.com), a
federally-funded Collaborative Research Centre, and the [University of
Technology, Sydney (UTS)](https://www.uts.edu.au).

## Licensing

Mawson Rovers has made the cubesat payload designs, firmware and software available under [an MIT License](LICENSE).

Please feel free to learn from this code and reuse it on future space missions under the terms of the license. However, we will not be taking external contributions to this code.
