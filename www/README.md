# stevenarella-web

Web app for running Stevenarella as WebAssembly

Status: very incomplete. It currently compiles, but **does not run** due to
missing web support from critical dependencies, at least:

* [glutin](https://github.com/tomaka/glutin) (temporary stub: [#1](https://github.com/iceiix/glutin/pull/1))
* [winit](https://github.com/tomaka/winit), watch for [stdweb suoort](https://github.com/tomaka/winit/pull/797) (temporary stub: [#2](https://github.com/iceiix/winit/pull/2))

## Building

To build for wasm32-unknown-unknown, run in the top-level directory (not www):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
wasm-pack build
```

## Running

After building the Rust app, run the NodeJS web server as follows:

```sh
cd pkg
npm link
cd ..
cd www
npm link stevenarella
npm install
npm start
open http://localhost:8080/
```

## Credits

Based on `[rustwasm/create-wasm-app](https://github.com/rustwasm/create-wasm-app)`:

> An `npm init` template for kick starting a project that uses NPM packages
> containing Rust-generated WebAssembly and bundles them with Webpack.

