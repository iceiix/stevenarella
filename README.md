# Stevenarella
[![Build Status](https://travis-ci.org/iceiix/steven.svg?branch=updates)](https://travis-ci.org/iceiix/steven)

Multi-protocol Minecraft-compatible client written in Rust

Don't expect it to go anywhere, just doing this for fun.

## Images

![Steven on Hypixel](https://i.imgur.com/PM5fLuu.png)
![Steven](https://i.imgur.com/RRspOQF.png)


In action: http://gfycat.com/NeedyElaborateGypsymoth

## Credits

Thanks to [@thinkofname](https://github.com/thinkofname/) for
the original [Steven (Rust)](https://github.com/thinkofname/steven),
which Stevenarella is an updated version of.

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

## License

Dual-licensed MIT and ApacheV2
