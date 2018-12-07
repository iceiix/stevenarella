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

Requires Rust stable version 1.31.0 or newer to build.

Compile and run:
```bash
cargo run --release
```
Just compile:
```bash
cargo build --release
```

## Running

### Standalone

Just running steven via a double click (Windows) or `./steven` (everything else)
will bring up a login screen followed by a server list which you can select a server
from.
