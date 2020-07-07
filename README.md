# Stevenarella
[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Ficeiix%2Fstevenarella%2Fbadge%3Fref%3Dmaster&style=plastic)](https://actions-badge.atrox.dev/iceiix/stevenarella/goto?ref=master)

Multi-protocol Minecraft-compatible client written in Rust

Don't expect it to go anywhere, just doing this for fun.

## Images

![Steven on Hypixel](https://i.imgur.com/PM5fLuu.png)
![Steven](https://i.imgur.com/RRspOQF.png)


In action: http://gfycat.com/NeedyElaborateGypsymoth

## Community chatroom

We have a chatroom on [EsperNet](https://esper.net): `irc.esper.net` server, `#stevenarella` channel.

Join with your favorite IRC client.

## Protocol support

| Game version | Protocol version | Supported? |
| ------ | --- | --- |
| 1.16.1 | 736 | ✓ |
| 1.16 | 735 | ✓ |
| 1.15.2 | 578 | ✓ |
| 1.15.1 | 575 | ✓ |
| 1.14.4 | 498 | ✓ |
| 1.14.3 | 490 | ✓ |
| 1.14.2 | 485 | ✓ |
| 1.14.1 | 480 | ✓ |
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

Forge servers are currently supported on 1.7.10 - 1.12.2.

Support for older protocols will _not_ be dropped as newer protocols are added.

## Downloads

Windows, Ubuntu Linux, and macOS users can download pre-compiled builds
from [GitHub Actions](https://actions-badge.atrox.dev/iceiix/stevenarella/goto?ref=master).
(Click the artifacts drop-down and select your platform.)

## Building

Requires Rust stable version 1.44.1 or newer to build.

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

## Contributions

Stevenarella is an [OPEN Open Source Project](https://github.com/openopensource/openopensource.github.io):

> Individuals making significant and valuable contributions are given
> commit-access to the project to contribute as they see fit. This project
> is more like an open wiki than a standard guarded open source project.

### Rules

There are a few basic ground-rules for contributors:

1. **No `--force` pushes** or modifying the Git history on the `master` branch.
1. **Non-master branches** ought to be used for ongoing work.
1. **External API changes and significant modifications** ought to be subject to an **internal pull-request** to solicit feedback from other contributors.
1. Internal pull-requests to solicit feedback are *encouraged* for any other non-trivial contribution but left to the discretion of the contributor.
1. Contributors should attempt to adhere to the prevailing code-style. Please install and run [cargo fmt](https://github.com/rust-lang/rustfmt) before merging any changes.

### Changes to this arrangement

This is an experiment and feedback is welcome! This document may also be
subject to pull-requests or changes by contributors where you believe
you have something valuable to add or change.

### Credits

Thanks to [@thinkofname](https://github.com/thinkofname/) for
the original [Steven (Rust)](https://github.com/thinkofname/steven),
which Stevenarella is an updated and enhanced version of.

### License

Dual-licensed MIT and ApacheV2
