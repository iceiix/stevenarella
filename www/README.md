# stevenarella-web

Web app for running Stevenarella as WebAssembly

Status: very incomplete. It currently compiles but does not run, due to required modifications to adapt to the web,
for progress see: [🕸️ Web support](https://github.com/iceiix/stevenarella/issues/446)

## Building

To build for wasm32-unknown-unknown, run in the top-level directory (not www):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-pack
cp -vr resources-*/assets/minecraft/* resources/assets/minecraft && git checkout resources
wasm-pack build --dev
```

or:

```sh
cargo web start --target wasm32-unknown-unknown
```

## Running

After building the Rust app, run the NodeJS web server as follows:

```sh
cd pkg
sudo npm link
cd ..
cd www
npm link stevenarella
npm install
npm start
open http://localhost:8080/
```

## Credits

Based on [rustwasm/create-wasm-app](https://github.com/rustwasm/create-wasm-app):

> An `npm init` template for kick starting a project that uses NPM packages
> containing Rust-generated WebAssembly and bundles them with Webpack.

