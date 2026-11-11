## TI OMAP4 X-Loader USB Bootstrapper Tool
This repository contains a rust crate for Windows that allows pushing a stage2 loader to X-Boot, and jump to that loader to execute desired core. Known working on Pandaboard non ES, further testing is required to observe behaviour on other devices.

It should be nottes that this is not yet working on HS (high-security) fused key devices, and alternative methods will have to be discovered to run unsigned code on devices using their own secure X-Loader.

I do not take responsibility for what you do with this program.

To build, set up a stable Rust toolchain and Visual Studio 2022 or later (needed for the Rust MSVC build tools), and run:

    cargo build debug
It will produce an EXE which you may run and pass your stage2 loader to in your terminal of choice.
