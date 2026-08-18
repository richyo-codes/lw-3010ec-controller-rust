# LW-3010EC Power Supply Controller

Control a Topshak/Longwei LW-3010EC bench power supply (0–30 V, 0–10 A) from the command line or a browser-based WebSerial UI.

Rust keeps the Modbus protocol implementation portable: the same core code compiles to a native CLI and to WebAssembly for the browser client. The included WebSerial client demonstrates browser-only PSU control and automation—no native controller application or backend API is required. The browser communicates directly with the PSU over WebSerial; the command-line tool communicates directly over Modbus RTU.

## Features

- Scan compatible CH340 serial adapters
- Read voltage, current, output state, and full status
- Set voltage/current limits and toggle output
- Interactive REPL and shell completions
- Browser-only WebSerial UI with live volts, amps, watts, session energy tracking, and automation-friendly refresh

## Installation

Rust 2021 and serial-port access are required (typically membership of the `dialout` group on Linux).

```bash
cargo build --release
```

## Command-line usage

```bash
# Find candidate serial ports
cargo run --release -- scan

# Read output state
cargo run --release -- --port /dev/ttyUSB0 status

# Set a 12.5 V output with a 2 A limit, then enable it
cargo run --release -- --port /dev/ttyUSB0 set-voltage 12.5
cargo run --release -- --port /dev/ttyUSB0 set-current 2.0
cargo run --release -- --port /dev/ttyUSB0 on

# Interactive mode
cargo run --release -- --port /dev/ttyUSB0 repl

# Generate shell completions
cargo run --release -- completions bash
```

Global options:

| Flag | Default | Description |
|---|---:|---|
| `-p`, `--port` | — | Serial port, such as `/dev/ttyUSB0` |
| `-u`, `--unit-id` | `1` | Modbus unit ID |
| `-t`, `--timeout` | `3` | Response timeout in seconds |

## Browser UI

WebSerial requires a secure context, so serve the UI from `localhost`. Supported desktop browsers include Chromium/Chrome 89+ ([Chrome documentation](https://developer.chrome.com/docs/capabilities/serial)) and Firefox 151+ ([Firefox 151 release notes](https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Releases/151)). Firefox prompts to install a site-permission add-on before granting serial access.

```bash
./run-python-webui.sh
```

Open `http://127.0.0.1:5000`, click **Connect via WebSerial**, and select the PSU. Do not run a CLI command against the same serial port while the browser is connected.

The UI shows live voltage, current, output state, power, session energy in Wh, and peak power/current. Enable auto-refresh for continuous consumption tracking.

### Rebuilding the browser protocol module

The generated WebAssembly files are committed under `web/pkg/`. Rebuild them after changing `lw3010ec-core`:

```bash
wasm-pack build --target web --out-name lw3010ec_core --out-dir web/pkg lw3010ec-core
```

## Publishing crates

The controller depends on `lw3010ec-core`, so publish the core crate first and then the controller crate at the matching version:

```bash
cargo publish --manifest-path lw3010ec-core/Cargo.toml
cargo publish
```

## Hardware

- Device: Topshak/Longwei LW-3010EC
- Example hardware listing: [AliExpress product page](https://www.aliexpress.com/item/1005008591226737.html)
- Connection: Modbus RTU over USB serial (CH340)
- Serial settings: 9600 baud, 8N1, no flow control

## Acknowledgments

The LW-3010EC protocol implementation was informed by the reverse-engineering work in [tttonyyy/lw-3010ec-controller](https://github.com/tttonyyy/lw-3010ec-controller).

## License

This project is licensed under the [MIT License](LICENSE).
