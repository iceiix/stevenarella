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

The Visual Studio 2017 Redistributable is required to run these builds.

## Building

Currently requires Rust version 1.31.0-beta or newer to build.

Compile and run:
```bash
cargo run --release
```
Just compile:
```bash
cargo build --release
```

### Prerequisites

An easy way to manage multiple Rust toolchains is [`rustup`](https://github.com/rust-lang-nursery/rustup.rs). Installation instructions for `rustup` can be found on its [website](https://www.rustup.rs/).

Once you've set up `rustup`, grab Rust beta by running
```sh
rustup install beta
```

Now we need to make sure that `steven` is compiled with beta. To do this without making beta the default across the entire system, run the following command in the `steven` directory:
```sh
rustup override set beta
```

## Running

### Standalone

Just running steven via a double click (Windows) or `./steven` (everything else)
will bring up a login screen followed by a server list which you can select a server
from.
