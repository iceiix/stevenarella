# stevenarella-web

Web app for running Stevenarella as WebAssembly

Status: very incomplete. It does not currently compile, due to required modifications to adapt to the web,
for progress see: [https://github.com/iceiix/stevenarella/issues/171](https://github.com/iceiix/stevenarella/issues/171).

## Building

To build for wasm32-unknown-unknown, run in the top-level directory (not www):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
wasm-pack build
```

or:

```sh
cargo web start --target wasm32-unknown-unknown
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

