//! A lightweight, `no_std`-friendly logging library for Rust
//! with support for compile-time filtering and optional runtime level control.
//!
//! > **Minimum Supported Rust Version:** `1.56.0`
//!
//! # Usage
//!
//! Add `loggery` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! loggery = "0.1.0"
//! ```
//!
//! Then use the logging macros:
//!
//! ```
//! use loggery::{trace, debug, info, warn, error};
//!
//! trace!("This is a TRACE log!");
//! debug!("This is a DEBUG log!");
//! info!("This is an INFO log!");
//! warn!("This is a WARN log!");
//! error!("This is an ERROR log!");
//! ```
//!
//! Output (default logger format):
//!
//! ```text
//! [TRACE] This is a TRACE log!
//! [DEBUG] This is a DEBUG log!
//! [ INFO] This is an INFO log!
//! [ WARN] This is a WARN log!
//! [ERROR] This is an ERROR log!
//! ```
//!
//! # Custom Logger
//!
//! By default, logs are written in the format: `[LEVEL] message`,
//! e.g., `[ERROR] Something went wrong!`.
//!
//! But you can implement your own:
//!
//! ```
//! use loggery::{Payload, debug};
//!
//! fn my_logger(payload: Payload) {
//!     // Your custom implementation
//!
//!     // For example, you can change the format of the logger
//!     println!("[APPLICATION]-{}-({})", payload.level.as_str(), payload.args);
//! }
//!
//! fn main() {
//! #   #[cfg(not(feature = "static"))]
//!     loggery::set_logger(my_logger);
//!
//!     debug!("A log message using my custom logger!");
//! }
//! ```
//!
//! Output:
//!
//! ```text
//! [APPLICATION]-DEBUG-(A log message using my custom logger!)
//! ```
//!
//! > **Note:** [`set_logger`] isn't available if the `static` feature is enabled!
//! > Read [Static](#static) for more details.
//!
//! # Runtime Level
//!
//! > **Note:** Only available when the `runtime_level` feature is enabled (enabled by default).
//!
//! You can dynamically change the minimum log level at runtime using [`set_min_level`]:
//!
//! ```
//! use loggery::{Level, debug, warn};
//!
//! # #[cfg(feature = "runtime_level")]
//! loggery::set_min_level(Level::Warn);
//!
//! debug!("This will NOT be logged");
//! warn!("This will be logged");
//! ```
//!
//! This works alongside compile-time filtering using `min_level_*` features.
//! Runtime filtering can only be more restrictive, not less restrictive than compile-time feature.
//! For example if the `min_level_info` feature is enabled, [`debug!`], [`trace!`] calls are
//! removed at compile-time and cannot be re-enabled at runtime.
//!
//! # Static
//!
//! > **Note:** Only available when the `static` feature is enabled.
//!
//! For maximum performance or in embedded/performance-critical applications, use the `static`
//! feature to remove the runtime indirection. Your logger is linked directly at compile time:
//!
//! ```toml
//! [dependencies]
//! loggery = { version = "0.1.0", default-features = false, features = ["static"]}
//! ```
//!
//! Then define your logger implementation in your binary crate:
//!
//! ```no_run
//! use loggery::{Payload, debug};
//!
//! #[no_mangle]
//! pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
//!     // Your custom implementation
//! }
//!
//! fn main() {
//!     debug!("Direct call from custom static implementation!")
//! }
//! ```
//! > **Tip:** You can use `static` with `std` feature if you want the default stdout logger with
//! > static dispatch:
//! >
//! > ```toml
//! > loggery = { version = "0.1.0", features = ["static"] } # `std` is enabled by default
//! > ```
//! >
//! > This gives you direct compile-time linking without needing to define `__loggery_log_impl`.
//!
//! > **Tip:** Even with `static` feature, you can still use the `runtime_level` feature and
//! > therefore the [`set_min_level`] function to do runtime log level filtering.
//!
//! <div class="warning">
//!
//! When using the `static` feature, you **must** provide the `__loggery_log_impl` function
//! in your binary crate, or you'll get a linker error!
//!
//! </div>
//!
//! # Extensions
//!
//! > **Note:** Only available when the `extension` feature is enabled.
//!
//! Extensions provide a hook for extra processing *alongside* the actual logger. They're called
//! before the logger and receive a reference to the [`Payload`], giving you the ability to:
//! - Save logs to files
//! - Send logs to external services
//! - Collect metrics
//! - etc.
//!
//! ```
//! use loggery::{Payload, debug};
//!
//! fn my_extension(payload: &Payload) {
//!     // Your custom implementation
//!
//!     // For example, you can use the provided extension `save_to_file`
//! #   #[cfg(all(feature = "extension", not(feature = "static")))]
//!     let _ = loggery::extensions::save_to_file(payload, "path/to/app.log");
//! }
//!
//! fn main() {
//! #   #[cfg(all(feature = "extension", not(feature = "static")))]
//!     loggery::set_extension(my_extension);
//!
//!     debug!("A log message that will be saved to a file too!");
//! }
//! ```
//!
//! > **Note:** When the `static` feature is enabled, `set_extension` isn't available. Instead,
//! > you can do this:
//! >
//! > ```no_run
//! > use loggery::Payload;
//! >
//! > #[no_mangle]
//! > pub extern "Rust" fn __loggery_extension_impl(payload: &Payload) {
//! >     // Your custom implementation
//! > }
//! > ```
//!
//! # Features
//!
//! > **Default features:** `std`, `metadata`, `runtime_level`
//!
//! |      Feature      | Default |                          Description                          |
//! |-------------------|:-------:|---------------------------------------------------------------|
//! | `std`             |  __✓__  | Enables default stdout logger                                 |
//! | `static`          |  __✗__  | Enables static extern logger definition                       |
//! | `metadata`        |  __✓__  | Enables [`meta`](Metadata) field in the [`Payload`]           |
//! | `extension`       |  __✗__  | Enables extension hooks for extra functionality               |
//! | `runtime_level`   |  __✓__  | Allows changing log level filtering at runtime                |
//! | `min_level_off`   |  __✗__  | Disables all logs at compile time                             |
//! | `min_level_trace` |  __✗__  | Only logs [`trace`], [`debug`], [`info`], [`warn`], [`error`] |
//! | `min_level_debug` |  __✗__  | Only logs [`debug`], [`info`], [`warn`], [`error`]            |
//! | `min_level_info`  |  __✗__  | Only logs [`info`], [`warn`], [`error`]                       |
//! | `min_level_warn`  |  __✗__  | Only logs [`warn`], [`error`]                                 |
//! | `min_level_error` |  __✗__  | Only logs [`error`]                                           |

#![no_std]

/// Log levels in order of incraesing severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    /// Trace
    Trace = 0,
    /// Debug
    Debug = 1,
    /// Info
    Info = 2,
    /// Warn
    Warn = 3,
    /// Error
    Error = 4,
}

impl Level {
    /// Returns the string representation with consistent width for right aligned output.
    #[inline(always)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => " INFO",
            Level::Warn => " WARN",
            Level::Error => "ERROR",
        }
    }

    /// Converts a u8 to a level, returning `None` if invalid.
    #[inline(always)]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Level::Trace),
            1 => Some(Level::Debug),
            2 => Some(Level::Info),
            3 => Some(Level::Warn),
            4 => Some(Level::Error),
            _ => None,
        }
    }
}

#[cfg(feature = "metadata")]
/// Extra context and information for a log.
pub struct Metadata {
    /// The module path where the log was generated.
    pub module_path: &'static str,
    /// The source file containing the log call.
    pub file: &'static str,
    /// The line number of the log call.
    pub line: u32,
}

/// The data passed to the logger and extensions.
pub struct Payload<'a> {
    /// The severity level of the log.
    pub level: Level,
    /// The formatted message arguments.
    pub args: core::fmt::Arguments<'a>,
    #[cfg(feature = "metadata")]
    /// Additional context and metadata (requires `metadata` feature).
    pub meta: Metadata,
}

/// Function type for custom logger implementation.
pub type LoggerFn = fn(Payload);

/// Function type for custom extension implementation.
#[cfg(feature = "extension")]
pub type ExtensionFn = fn(&Payload);

/// Compile-time minimum log level set by `min_level_*` feature flags.
///
/// If no specific level is set, all logs are enabled by default (`min_level_trace`).
const COMPILE_TIME_MIN_LEVEL: Option<u8> = match () {
    _ if cfg!(feature = "min_level_off") => None,
    _ if cfg!(feature = "min_level_error") => Some(Level::Error as u8),
    _ if cfg!(feature = "min_level_warn") => Some(Level::Warn as u8),
    _ if cfg!(feature = "min_level_info") => Some(Level::Info as u8),
    _ if cfg!(feature = "min_level_debug") => Some(Level::Debug as u8),
    _ if cfg!(feature = "min_level_trace") => Some(Level::Trace as u8),
    _ => Some(Level::Trace as u8), // By default, allow all logs
};

#[cfg(feature = "static")]
extern "Rust" {
    /// External logger implementation that *MUST* be provided when using the `static` feature.
    ///
    /// # Safety
    ///
    /// When the `static` feature is enabled, you must define this function in your binary crate:
    ///
    /// ```no_run
    /// use loggery::Payload;
    ///
    /// #[no_mangle]
    /// pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
    ///     // Your custom implementation
    /// }
    /// ```
    ///
    /// **Warning:** Not providing this function will result in a linker error!
    fn __loggery_log_impl(payload: Payload);

    /// External extension implementation that *CAN* be provided when using the `external` and
    /// `static` features.
    ///
    /// # Safety
    ///
    /// When the `external` and `static` features are enabled, you can define this function in
    /// your binary crate:
    ///
    /// ```no_run
    /// use loggery::Payload;
    ///
    /// #[no_mangle]
    /// pub extern "Rust" fn __loggery_extension_impl(payload: &Payload) {
    ///     // Your custom implementation
    /// }
    /// ```
    ///
    /// **Note:** Not providing this function is fine since a NOP version is implementated
    /// by default.
    #[cfg(feature = "extension")]
    fn __loggery_extension_impl(payload: &Payload);
}

/// Global logger function pointer storage. (NOT `static` feature)
#[cfg(not(feature = "static"))]
static LOGGER_FN: core::sync::atomic::AtomicPtr<()> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
/// Global extension function pointer storage. (`extension` feature, NOT `static` feature)
#[cfg(all(feature = "extension", not(feature = "static")))]
static EXTENSION_FN: core::sync::atomic::AtomicPtr<()> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
/// Runtime minimum log level storage. (`runtime_level` feature)
#[cfg(feature = "runtime_level")]
static RUNTIME_MIN_LEVEL: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(Level::Trace as u8);

/// Sets the global logger function. (NOT `static` feature)
///
/// It's recommended to call once during the initialization.
///
/// # Example
///
/// ```
/// use loggery::{Payload, debug};
///
/// fn my_logger(payload: Payload) {
///     // Your custom implementation
/// }
///
/// fn main() {
///     loggery::set_logger(my_logger);
///
///     debug!("A log message using my custom logger!");
/// }
/// ```
///
/// # Note
///
/// When the `static` feature is enabled, this function isn't available. Instead, you must define
/// this function in your binary crate:
///
/// ```no_run
/// use loggery::Payload;
///
/// #[no_mangle]
/// pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
///     // Your custom implementation
/// }
/// ```
///
/// When the `std` feature is enabled, a default logger is automatically initialized if no logger
/// has been set. This function can still be used to override that default.
#[cfg(not(feature = "static"))]
#[inline(always)]
pub fn set_logger(logger_fn: LoggerFn) {
    LOGGER_FN.store(logger_fn as *mut (), core::sync::atomic::Ordering::Release)
}

/// Sets the global extension function. (`extension` feature, NOT `static` feature)
///
/// Extensions are called before the logger and receive a reference to the [`Payload`], giving us
/// the ability to do additional functionality like saving logs to file.
///
/// It's recommended to call once during the initialization.
///
/// # Example
///
/// ```no_run
/// use loggery::{Payload, debug};
///
/// fn my_extension(payload: &Payload) {
///     // Your custom implementation
///
///     // For example, you can use the provided extension `save_to_file`
///     let _ = loggery::extensions::save_to_file(payload, "path/to/app.log");
/// }
///
/// fn main() {
///     loggery::set_extension(my_extension);
///
///     debug!("A log message that will be saved to a file too!");
/// }
/// ```
///
/// # Note
///
/// When the `static` feature is enabled, this function isn't available. Instead, you must define
/// this function in your binary crate:
///
/// ```no_run
/// use loggery::Payload;
///
/// #[no_mangle]
/// pub extern "Rust" fn __loggery_extension_impl(payload: &Payload) {
///     // Your custom implementation
/// }
/// ```
#[cfg(all(feature = "extension", not(feature = "static")))]
#[inline(always)]
pub fn set_extension(extension_fn: ExtensionFn) {
    EXTENSION_FN.store(
        extension_fn as *mut (),
        core::sync::atomic::Ordering::Release,
    )
}

/// Sets the runtime minimum log level. (`runtime_level` feature)
///
/// > You can also use the `min_level_*` features for compile-time level filtering.
/// > (e.g., `min_level_warn` will disable all the log levels below [`warn!`] at compile-time.)
///
/// # Note
///
/// This cannot enable levels that were filtered at compile time.
/// If compiled with feature `min_level_info`, calling `set_min_level(Level::Debug)` will have
/// no effect.
///
/// # Example
///
/// ```
/// use loggery::{Level, debug, warn};
///
/// loggery::set_min_level(Level::Warn);
///
/// debug!("This will NOT be logged");
/// warn!("This will be logged");
/// ```
#[cfg(feature = "runtime_level")]
#[inline(always)]
pub fn set_min_level(level: Level) {
    RUNTIME_MIN_LEVEL.store(level as u8, core::sync::atomic::Ordering::Release);
}

/// Returns the effective minimum log level (the stricter of compile-time and runtime levels).
///
/// # Example
///
/// ```
/// use loggery::Level;
///
/// # #[cfg(feature = "runtime_level")]
/// loggery::set_min_level(Level::Debug);
///
/// # #[cfg(all(feature = "runtime_level", not(feature = "min_level_off")))]
/// assert_eq!(loggery::get_min_level(), Some(Level::Debug));
/// ```
#[inline(always)]
pub fn get_min_level() -> Option<Level> {
    let compile_time = COMPILE_TIME_MIN_LEVEL?;

    let level = {
        #[cfg(feature = "runtime_level")]
        {
            let runtime = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);

            // Return whichever is stricter (higher level)
            compile_time.max(runtime)
        }

        #[cfg(not(feature = "runtime_level"))]
        compile_time
    };

    Level::from_u8(level)
}

/// Core logging entry point used internally by macros like [`debug!`] and [`error!`].
///
/// It's recommended to use the macros instead of calling directly.
#[inline(always)]
pub fn log(payload: Payload) {
    let is_compile_time_enabled = match COMPILE_TIME_MIN_LEVEL {
        Some(level) => (payload.level as u8) >= level,
        None => false,
    };

    if !is_compile_time_enabled {
        return;
    }

    #[cfg(feature = "runtime_level")]
    {
        let runtime_min_level = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);

        if (payload.level as u8) < runtime_min_level {
            return;
        }
    }

    #[cfg(all(feature = "extension", feature = "static"))]
    {
        unsafe { __loggery_extension_impl(&payload) };
    }

    #[cfg(all(feature = "extension", not(feature = "static")))]
    {
        if let Some(extension_fn) = get_extension() {
            extension_fn(&payload)
        }
    }

    #[cfg(feature = "static")]
    {
        unsafe { __loggery_log_impl(payload) };
    }

    #[cfg(not(feature = "static"))]
    {
        if let Some(logger_fn) = get_logger() {
            logger_fn(payload)
        }
    }
}

/// Converts a raw pointer back to a `LoggerFn`.
///
/// # Safety
///
/// Safe only when `ptr` was created by casting a valid `LoggerFn` to `*mut ()`.
/// The caller must ensure:
/// - Pointer originated from a valid function pointer cast
/// - Fnuction pointer has 'static lifetime (guaranteed for all fn pointers)
/// - Proper synchronization (handled by atomic ops)
#[cfg(not(feature = "static"))]
#[inline(always)]
fn ptr_to_logger_fn(ptr: *mut ()) -> LoggerFn {
    // SAFETY: `ptr` was created from `LoggerFn` in `set_logger`. Function pointers are 'static.
    // Atomics ensure cross-thread visibility.
    unsafe { core::mem::transmute::<*mut (), LoggerFn>(ptr) }
}

/// Converts a raw pointer back to an `ExtensionFn`.
///
/// # Safety
///
/// Safe only when `ptr` was created by casting a valid `ExtensionFn` to `*mut ()`.
/// The caller must ensure:
/// - Pointer originated from a valid function pointer cast
/// - Fnuction pointer has 'static lifetime (guaranteed for all fn pointers)
/// - Proper synchronization (handled by atomic ops)
#[cfg(all(feature = "extension", not(feature = "static")))]
#[inline(always)]
fn ptr_to_extension_fn(ptr: *mut ()) -> ExtensionFn {
    // SAFETY: `ptr` was created from `ExtensionFn` in `set_extension`.
    // Function pointers are 'static. Atomics ensure cross-thread visibility.
    unsafe { core::mem::transmute::<*mut (), ExtensionFn>(ptr) }
}

/// Gets the logger, auto-initializing a default stdout logger if needed (`std` feature).
/// Returns `None` if not set and `std` feature isn't enabled.
#[cfg(not(feature = "static"))]
#[inline(always)]
fn get_logger() -> Option<LoggerFn> {
    let ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        #[cfg(feature = "std")]
        {
            let _ = LOGGER_FN.compare_exchange(
                core::ptr::null_mut(),
                stdout::logger_fn as usize as *mut (),
                core::sync::atomic::Ordering::AcqRel,
                core::sync::atomic::Ordering::Acquire,
            );

            // Reload after initialization
            let new_ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

            return Some(ptr_to_logger_fn(new_ptr));
        }

        #[cfg(not(feature = "std"))]
        return None;
    }

    Some(ptr_to_logger_fn(ptr))
}

/// Gets the extension function if one is set, otherwise returns None.
#[cfg(all(feature = "extension", not(feature = "static")))]
#[inline(always)]
fn get_extension() -> Option<ExtensionFn> {
    let ptr = EXTENSION_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        return None;
    }

    Some(ptr_to_extension_fn(ptr))
}

/// Logs a message at the specified level.
///
/// This is the underlying macro used by [`trace!`], [`debug!`], [`info!`], [`warn!`] and [`error!`].
/// While you mostly use those level-specific macros, `log!` can be useful when you want to specify
/// the log level dynamically or crate your own logging abstractions.
///
/// # Example
///
/// ```
/// use loggery::{Level, log};
///
/// let level = if cfg!(debug_assertions) {
///     Level::Debug
/// } else {
///     Level::Info
/// };
///
/// log!(level, "This is a log with dynamically set level")
/// ```
#[cfg(feature = "metadata")]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::log($crate::Payload {
            level: $level,
            args: format_args!($($arg)*),
            meta: $crate::Metadata {
                module_path: module_path!(),
                file: file!(),
                line: line!(),
            },
        })
    };
}

/// Logs a message at the specified level.
///
/// This is the underlying macro used by [`trace!`], [`debug!`], [`info!`], [`warn!`] and [`error!`].
/// While you mostly use those level-specific macros, `log!` can be useful when you want to specify
/// the log level dynamically or crate your own logging abstractions.
///
/// # Example
///
/// ```
/// use loggery::{Level, log};
///
/// let level = if cfg!(debug_assertions) {
///     Level::Debug
/// } else {
///     Level::Info
/// };
///
/// log!(level, "This is a log with dynamically set level")
/// ```
#[cfg(not(feature = "metadata"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::log($crate::Payload {
            level: $level,
            args: format_args!($($arg)*),
        })
    };
}

/// Logs a message at the `trace` level.
///
/// # Example
///
/// ```
/// use loggery::trace;
///
/// trace!("Entering function...");
/// ```
///
/// # Compile-time filtering
///
/// If feature `min_level_debug` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log!($crate::Level::Trace, $($arg)*);
    };
}

/// Logs a message at the `debug` level.
///
/// # Example
///
/// ```
/// use loggery::debug;
///
/// let x = 69;
///
/// debug!("Variable `x` is {}", x);
/// ```
///
/// # Compile-time filtering
///
/// If feature `min_level_info` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log!($crate::Level::Debug, $($arg)*);
    };
}

/// Logs a message at the `info` level.
///
/// # Example
///
/// ```
/// use loggery::info;
///
/// const PORT: u16 = 8080;
///
/// info!("Server is started on port {}", PORT);
/// ```
///
/// # Compile-time filtering
///
/// If feature `min_level_warn` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log!($crate::Level::Info, $($arg)*);
    };
}

/// Logs a message at the `warn` level.
///
/// # Example
///
/// ```
/// use loggery::warn;
///
/// warn!("Configuration file not found, using defaults");
/// ```
///
/// # Compile-time filtering
///
/// If feature `min_level_error` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log!($crate::Level::Warn, $($arg)*);
    };
}

/// Logs a message at the `error` level.
///
/// # Example
///
/// ```
/// use loggery::error;
///
/// error!("Failed to connect to database!");
/// ```
///
/// # Compile-time filtering
///
/// If feature `min_level_off` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log!($crate::Level::Error, $($arg)*);
    };
}

/// Built-in extension utilities for common logging tasks.
///
/// These functions are desigend to be called from within your custom extension function.
/// They must not be passed directly to the [`set_extension`] because they might need parameters.
///
/// # Example
///
/// ```no_run
/// use loggery::{Payload, debug};
///
/// fn my_extension(payload: &Payload) {
///     // Your custom implementation
///
///     // For example, you can use the provided extension `save_to_file`
/// #   #[cfg(all(feature = "extension", not(feature = "static")))]
///     let _ = loggery::extensions::save_to_file(payload, "path/to/app.log");
/// }
///
/// fn main() {
/// #   #[cfg(all(feature = "extension", not(feature = "static")))]
///     loggery::set_extension(my_extension);
///
///     debug!("A log message that will be saved to a file too!");
/// }
/// ```
#[cfg(feature = "extension")]
pub mod extensions {
    use crate::Payload;

    #[cfg(feature = "std")]
    extern crate std;

    /// Appends a log entry to a file (`std` feature)
    ///
    /// The file at the `path` is opened in append mode.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use loggery::{Payload, debug};
    ///
    /// fn my_extension(payload: &Payload) {
    /// #   #[cfg(all(feature = "extension", not(feature = "static")))]
    ///     let _ = loggery::extensions::save_to_file(payload, "path/to/app.log");
    /// }
    ///
    /// fn main() {
    /// #   #[cfg(all(feature = "extension", not(feature = "static")))]
    ///     loggery::set_extension(my_extension);
    ///
    ///     debug!("A log message that will be saved to a file too!");
    /// }
    /// ```
    ///
    /// # Format
    ///
    /// Logs are written in the format: `[LEVEL] message`
    #[cfg(feature = "std")]
    #[inline]
    pub fn save_to_file(payload: &Payload, path: &str) -> std::io::Result<()> {
        use std::io::Write as _;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(file, "[{}] {}", payload.level.as_str(), payload.args)
    }
}

#[cfg(feature = "std")]
mod stdout {
    extern crate std;

    use crate::Payload;

    /// Default stdout logger (`std` feature).
    #[inline(always)]
    pub(super) fn logger_fn(payload: Payload) {
        use std::io::Write as _;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        let _ = writeln!(handle, "[{}] {}", payload.level.as_str(), payload.args);
    }
}

/// This module is included if the `static` and `std` features are enabled to provide
/// the default definition for log function.
#[cfg(feature = "static_default")]
mod static_impl_std {
    use crate::Payload;

    /// Default logger implementation for when the `std` and `static` features are enabled.
    #[no_mangle]
    pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
        crate::stdout::logger_fn(payload);
    }
}
