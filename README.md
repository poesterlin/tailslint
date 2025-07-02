# Trayscale

Wrapper for Tailscale CLI using [Slint](https://docs.slint.dev/latest/docs/slint/), build in Rust.

There is a version that works as a window,

![screenshot](imgs/screenshot.png)

and one that works as a tray, build using [tray-icon](https://crates.io/crates/tray-icon). 

![screenshot](imgs/tray.png)

Checkout the `tray-version` branch for this.

## Building

```bash
cargo build --release
```

## Running

```bash
sudo tailscale set --operator=$USER # use tailscale without root
./target/release/tailslint

#or
sudo cp ./target/release/tailslint /usr/local/bin/
tailslint
```
