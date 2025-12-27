# Loggery

[![Build Status](https://img.shields.io/github/actions/workflow/status/IrregularCelery/loggery/ci.yml?branch=master)](https://github.com/IrregularCelery/loggery/actions)
[![Crates.io](https://img.shields.io/crates/v/loggery.svg)](https://crates.io/crates/loggery)
[![Documentation](https://docs.rs/loggery/badge.svg)](https://docs.rs/loggery)
[![License](https://img.shields.io/crates/l/loggery.svg)](https://github.com/IrregularCelery/loggery/blob/master/LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.56.0-yellow)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)

### A lightweight, `no_std`-friendly logging library for Rust.

## Usage

Add `loggery` to your `Cargo.toml`:

```toml
[dependencies]
loggery = "0.1.0"
```

Then use the logging macros:

```rust
use loggery::{trace, debug, info, warn, error};

trace!("This is a TRACE log!");
debug!("This is a DEBUG log!");
info!("This is an INFO log!");
warn!("This is a WARN log!");
error!("This is an ERROR log!");
```

Output (default logger format):

```text
[TRACE] This is a TRACE log!
[DEBUG] This is a DEBUG log!
[ INFO] This is an INFO log!
[ WARN] This is a WARN log!
[ERROR] This is an ERROR log!
```

## Custom Logger

By default, logs are written in the format: `[LEVEL] message`,
e.g., `[ERROR] Something went wrong!`.

But you can implement your own:

```rust
use loggery::{Payload, debug};

fn my_logger(payload: Payload) {
    // Your custom implementation

    // For example, you can change the format of the logger
    println!("[APPLICATION]-{}-({})", payload.level.as_str(), payload.args);
}

fn main() {
    loggery::set_logger(my_logger);

    debug!("A log message using my custom logger!");
}
```

Output:

```text
[APPLICATION]-DEBUG-(A log message using my custom logger!)
```

> [!NOTE]
> `set_logger` isn't available if the `static` feature is enabled! Read [Static](#static) for
> more details.

## Runtime Level

> [!NOTE]
> Only available when the `runtime_level` feature is enabled (enabled by default).

You can dynamically change the minimum log level at runtime using `set_min_level`:

```rust
use loggery::{Level, debug, warn};

loggery::set_min_level(Level::Warn);

debug!("This will NOT be logged");
warn!("This will be logged");
```

This works alongside compile-time filtering using `min_level_*` features.
Runtime filtering can only be more restrictive, not less restrictive than compile-time feature.
For example if the `min_level_info` feature is enabled, `debug!`, `trace!` calls are removed
at compile-time and cannot be re-enabled at runtime.

## Static

> [!NOTE]
> Only available when the `static` feature is enabled.

For maximum performance or in embedded/performance-critical applications, use the `static` feature
to remove the runtime indirection. Your logger is linked directly at compile time:

```toml
[dependencies]
loggery = { version = "0.1.0", default-features = false, features = ["static"]}
```

Then define your logger implementation in your binary crate:

```rust
use loggery::{Payload, debug};

#[no_mangle]
pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
    // Your custom implementation
}

fn main() {
    debug!("Direct call from custom static implementation!")
}
```

> [!TIP]
> You can use `static` with `std` feature if you want the default stdout logger with static
> dispatch:
>
> ```toml
> loggery = { version = "0.1.0", features = ["static"] } # `std` is enabled by default
> ```
>
> This gives you direct compile-time linking without needing to define `__loggery_log_impl`.

> [!TIP]
> Even with `static` feature, you can still use the `runtime_level` feature and therefore
> the `set_min_level` function to do runtime log level filtering.

> [!WARNING]
> When using the `static` feature, you **must** provide the `__loggery_log_impl` function
> in your binary crate, or you'll get a linker error!

## Extensions

> [!NOTE]
> Only available when the `extension` feature is enabled.

Extensions provide a hook for extra processing _alongside_ the actual logger. They're called
before the logger and receive a reference to the `Payload`, giving you the ability to:

- Save logs to files
- Send logs to external services
- Collect metrics
- etc.

```rust
use loggery::{Payload, debug};

fn my_extension(payload: &Payload) {
    // Your custom implementation

    // For example, you can use the provided extension `save_to_file`
    let _ = loggery::extensions::save_to_file(payload, "path/to/app.log");
}

fn main() {
    loggery::set_extension(my_extension);

    debug!("A log message that will be saved to a file too!");
}
```

> [!NOTE]
> When the `static` feature is enabled, `set_extension` isn't available. Instead, you can do this:
>
> ```rust
> use loggery::Payload;
>
> #[no_mangle]
> pub extern "Rust" fn __loggery_extension_impl(payload: &Payload) {
>    // Your custom implementation
> }
> ```

## Features

> **Default features:** `std`, `metadata`, `runtime_level`

| Feature           | Default | Description                                         |
| ----------------- | :-----: | --------------------------------------------------- |
| `std`             |  **✓**  | Enables default stdout logger                       |
| `static`          |  **✗**  | Enables static extern logger definition             |
| `metadata`        |  **✓**  | Enables `meta` field in the `Payload`               |
| `extension`       |  **✗**  | Enables extension hooks for extra functionality     |
| `runtime_level`   |  **✓**  | Allows changing log level filtering at runtime      |
| `min_level_off`   |  **✗**  | Disables all logs at compile time                   |
| `min_level_trace` |  **✗**  | Only logs `trace`, `debug`, `info`, `warn`, `error` |
| `min_level_debug` |  **✗**  | Only logs `debug`, `info`, `warn`, `error`          |
| `min_level_info`  |  **✗**  | Only logs `info`, `warn`, `error`                   |
| `min_level_warn`  |  **✗**  | Only logs `warn`, `error`                           |
| `min_level_error` |  **✗**  | Only logs `error`                                   |
