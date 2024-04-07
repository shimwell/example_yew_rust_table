Minimal example of a Yew Table making use [yew-custom-components](https://github.com/aknarts/yew-custom-components) examples


[Demo](https://shimwell.github.io/example_yew_rust_table/)

Install instructions on Ubuntu 22.04
```
sudo apt-get install curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
cargo install --locked trunk
trunk serve --open
```
