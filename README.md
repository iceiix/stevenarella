# Stevenarella
[![Build Status](https://travis-ci.org/iceiix/stevenarella.svg?branch=master)](https://travis-ci.org/iceiix/stevenarella)

Multi-protocol Minecraft-compatible client written in Rust

Don't expect it to go anywhere, just doing this for fun.

## Images

![Steven on Hypixel](https://i.imgur.com/PM5fLuu.png)
![Steven](https://i.imgur.com/RRspOQF.png)


In action: http://gfycat.com/NeedyElaborateGypsymoth

## Community chatroom

We have a chatroom on [EsperNet](https://esper.net): `irc.esper.net` server, `#stevenarella` channel.

Join with your favorite IRC client or [Matrix](https://matrix.to/#/#_espernet_#stevenarella:matrix.org).

## Protocol support

| Game version | Protocol version | Supported? |
| ------ | --- | --- |
| 1.14.1 | 477 | ✓ |
| 1.14 | 477 | ✓ |
| 19w02a | 452 | ✓ |
| 18w50a | 451 | ✓ |
| 1.13.2 | 404 | ✓ |
| 1.12.2 | 340 | ✓ |
| 1.11.2 | 316 | ✓ |
| 1.11   | 315 | ✓ |
| 1.10.2 | 210 | ✓ |
| 1.9.2  | 109 | ✓ |
| 1.9    | 107 | ✓ |
| 15w39c | 74  | ✓ |
| 1.8.9  | 47  | ✓ |
| 1.7.10 | 5   | ✓ |

Stevenarella is designed to support multiple protocol versions, so that client
development is not in lock-step with the server version. The level of
support varies, but the goal is to support major versions from 1.7.10
up to the current latest major version. Occasionally, snapshots are also supported.

Forge servers are supported on 1.7.10 - 1.12.2.

Support for older protocols will _not_ be dropped as newer protocols are added.

## Credits

Thanks to [@thinkofname](https://github.com/thinkofname/) for
the original [Steven (Rust)](https://github.com/thinkofname/steven),
which Stevenarella is an updated and enhanced version of.

## Downloads

Windows users can download pre-compiled builds from here: https://ci.appveyor.com/project/iceiix/stevenarella
(Select your platform, Click the artifacts tab and download Steven.zip)

The Visual Studio 2017 Redistributable is required to run these builds.

## Building

Requires Rust stable version 1.34.1 or newer to build.

Compile and run:
```bash
cargo run --release
```
Just compile:
```bash
cargo build --release
```

For progress on web support, see [www/](./www).

## Running

### Standalone

Just running Stevenarella via a double click (Windows) or `./stevenarella` (everything else)
will bring up a login screen followed by a server list which you can select a server
from.

## License

Dual-licensed MIT and ApacheV2
