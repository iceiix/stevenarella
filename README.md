# Steven (Rust)
[![Build Status](https://travis-ci.org/iceiix/steven.svg?branch=updates)](https://travis-ci.org/iceiix/steven)

A Minecraft client coded in Rust. Ported from [steven-go](https://github.com/Thinkofname/steven-go).
Don't expect it to go anywhere, just doing this for fun.

## Images

![Steven on Hypixel](https://i.imgur.com/PM5fLuu.png)
![Steven](https://i.imgur.com/RRspOQF.png)


In action: http://gfycat.com/NeedyElaborateGypsymoth

## Chat

I generally am on the `irc.spi.gt` irc network in the `#think` channel.
Feel free to pop in to say hi, [Webchat can be found here](https://irc.spi.gt/iris/?channels=think)

## Downloads

Windows users can download pre-compiled builds from here: https://ci.appveyor.com/project/iceiix/steven
(Select your platform, Click the artifacts tab and download Steven.zip)

The Visual Studio 2015 Redistributable is required to run these builds.

## Building

Currently requires SDL2, and **beta or nightly** Rust to build.

Compile and run:
```bash
cargo run --release
```
Just compile:
```bash
cargo build --release
```

If you get an error such as:

```
  = note: ld: library not found for -lSDL2                                                                                                                                                                                                   
          clang: error: linker command failed with exit code 1 (use -v to see invocation)     
```

then you need to install the prerequisites, see below:

### Prerequisites

Compiling `steven` requires Rust beta and SDL2. Instructions for setting up SDL2 is platform-specific and will be covered as such outside of this section. 

An easy way to manage multiple Rust toolchains is [`rustup`](https://github.com/rust-lang-nursery/rustup.rs). Installation instructions for `rustup` can be found on its [website](https://www.rustup.rs/).

Once you've set up `rustup`, grab Rust beta by running
```sh
rustup install beta
```

Now we need to make sure that `steven` is compiled with beta. To do this without making beta the default across the entire system, run the following command in the `steven` directory:
```sh
rustup override set beta
```

### Installing dependencies on Linux
Install SDL2 (with headers) via your distro's package manager. Packages with headers generally end with `-dev`.
For example on Debian-based systems such as Ubuntu Linux:

```bash
apt-get install -y libsdl2-dev libsdl2-mixer-dev gcc libegl1-mesa-dev libgles2-mesa-dev
```

### Installing dependencies on OS X
Installing them is easiest with [Homebrew](http://brew.sh/). To install SDL2 issue this command:

```bash
brew install sdl2
```

### Installing dependencies on Windows
Build with Visual Studio 2015. May build with other compilers, but not tested
(previously was built with MinGW and the GNU toolchain).

Download [SDL2-devel-2.0.4-VC.zip](https://www.libsdl.org/release/SDL2-devel-2.0.4-VC.zip), extract and
copy SDL2-2.0.4\lib\x64\SDL2.lib to C:\Rust\lib\rustlib\x86_64-pc-windows-msvc\lib\SDL2.lib.

## Running

### Standalone

Just running steven via a double click (Windows) or `./steven` (everything else)
will bring up a login screen followed by a server list which you can select a server
from.
