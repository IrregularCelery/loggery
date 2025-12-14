//! A lightweight, `no_std`-friendly logging library for Rust
//! with support for compile-time filtering and optional runtime level control.
//!
//! # Usage
//!
//! Add `loggery` to your `Cargo.toml`:
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
//! # Custom Logger
//!
//! ```
//! use loggery::{Payload, Level, debug};
//!
//! fn my_custom_logger(payload: Payload) {
//!     // Your custom implementation
//! }
//!
//! fn main () {
//!     loggery::set_logger(my_custom_logger);
//!     loggery::set_min_level(Level::Trace);
//!
//!     debug!("A log message using my custom logger!");
//! }
//! ```
//!
//! # Static Logger
//!
//! For maximum performance in embedded or performance-critical applications, use the `static`
//! feature to remove the runtime indirection. Your logger is linked directly at compile time:
//!
//! ```toml
//! [dependencies]
//! loggery = { version = "0.1.0", default-features = false, features = ["static"]}
//! ```
//!
//! Then define your logger implementation in your binary crate:
//!
//! ```
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
//! **Note:** Even with `static` feature, you can still use the `runtime_levels` feature and
//! therefore the [`set_min_level`] function to do runtime log level filtering.
//!
//! # Features
//!
//! > **Default features:** `std`, `runtime_levels`
//! - `std`: Enables default stdout logger
//! - `static`: Enables static extern logger definition
//! - `metadata`: Enables `meta` field in the [`Payload`]
//! - `runtime_levels`: Allows changing log level filtering at runtime
//! - `min_level_*`: Compile-time log level filtering
//!     - `min_level_off`: Disables all the logs
//!     - `min_level_trace`: Enables log levels ([`trace`], [`debug`], [`info`], [`warn`], [`error`])
//!     - `min_level_debug`: Enables log levels ([`debug`], [`info`], [`warn`], [`error`])
//!     - `min_level_info`:  Enables log levels ([`info`], [`warn`], [`error`])
//!     - `min_level_warn`:  Enables log levels ([`warn`], [`error`])
//!     - `min_level_error`: Enables log levels ([`error`])

#![no_std]

#[cfg(feature = "std")]
extern crate std;

/// Log levels in order of incraesing severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
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
pub struct Metadata {
    pub module_path: &'static str,
    pub file: &'static str,
    pub line: u32,
}

pub struct Payload<'a> {
    pub level: Level,
    pub args: core::fmt::Arguments<'a>,
    #[cfg(feature = "metadata")]
    pub meta: Metadata,
}

/// Function type for custom logger implementation.
pub type LoggerFn = fn(Payload);

/// Compile-time minimum log level set by `min_level_*` feature flags.
///
/// If no specific level is set, all logs are enabled by default (`min_level_trace`).
const COMPILE_TIME_MIN_LEVEL: Option<u8> = match () {
    _ if cfg!(feature = "min_level_off") => None,
    _ if cfg!(feature = "min_level_trace") => Some(Level::Trace as u8),
    _ if cfg!(feature = "min_level_debug") => Some(Level::Debug as u8),
    _ if cfg!(feature = "min_level_info") => Some(Level::Info as u8),
    _ if cfg!(feature = "min_level_warn") => Some(Level::Warn as u8),
    _ if cfg!(feature = "min_level_error") => Some(Level::Error as u8),
    _ => Some(Level::Trace as u8), // By default, allow all logs
};

#[cfg(feature = "static")]
extern "Rust" {
    /// External logger implementation that must be provided when using the `static` feature.
    ///
    /// # Safety
    ///
    /// When the `static` feature is enabled, you must define this function in your binary crate:
    /// ```
    /// #[no_mangle]
    /// pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
    ///     // Your custom implementation
    /// }
    /// ```
    ///
    /// Not providing this function will result in a linker error!
    fn __loggery_log_impl(payload: Payload);
}

/// Global logger function pointer storage. (NOT `static` feature)
#[cfg(not(feature = "static"))]
static LOGGER_FN: core::sync::atomic::AtomicPtr<()> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
/// Runtime minimum log level storage. (`runtime_levels` feature)
#[cfg(feature = "runtime_levels")]
static RUNTIME_MIN_LEVEL: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(Level::Trace as u8);

/// Sets the global logger function. (NOT `static` feature)
///
/// It's recommended to call once during the initialization.
///
/// # Example
///
/// ```
/// use loggery::{Payload, Level, debug};
///
/// fn my_custom_logger(payload: Payload) {
///     // Your custom implementation
/// }
///
/// fn main () {
///     loggery::set_logger(my_custom_logger);
///     loggery::set_min_level(Level::Trace);
///
///     debug!("A log message using my custom logger!");
/// }
/// ```
///
/// # Note
///
/// When the `static` feature is enabled, this function isn't available. Instead, you must define
/// this function in your binary crate:
/// ```
/// use loggery::Payload;
///
/// #[no_mangle]
/// pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
///     // Your custom implementation
/// }
/// ```
/// When the `std` feature is enabled, a default logger is automatically initialized if no logger
/// has been set. This function can still be used to override that default.
#[cfg(not(feature = "static"))]
#[inline(always)]
pub fn set_logger(logger_fn: LoggerFn) {
    LOGGER_FN.store(
        logger_fn as usize as *mut (),
        core::sync::atomic::Ordering::Release,
    )
}

/// Set the runtime minimum log level. (`runtime_levels` feature)
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
/// If the `runtime_levels` feature *isn't* enabled, you can use the `min_level_*` features for
/// compile-time level filtering.
#[cfg(feature = "runtime_levels")]
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
/// loggery::set_min_level(Level::Debug);
///
/// assert_eq!(loggery::get_min_level(), Some(Level::Debug));
/// ```
#[inline(always)]
pub fn get_min_level() -> Option<Level> {
    let compile_time = COMPILE_TIME_MIN_LEVEL?;

    let level = {
        #[cfg(feature = "runtime_levels")]
        {
            let runtime = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);

            // Return whichever is stricter (higher level)
            compile_time.max(runtime)
        }

        #[cfg(not(feature = "runtime_levels"))]
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

    #[cfg(feature = "runtime_levels")]
    {
        let runtime_min_level = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);

        if (payload.level as u8) < runtime_min_level {
            return;
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

/// Gets the logger, auto-initializing a default stdout logger if needed (`std` feature).
#[cfg(all(feature = "std", not(feature = "static")))]
#[inline(always)]
fn get_logger() -> Option<LoggerFn> {
    let ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        let _ = LOGGER_FN.compare_exchange(
            core::ptr::null_mut(),
            std_logger_fn as usize as *mut (),
            core::sync::atomic::Ordering::AcqRel,
            core::sync::atomic::Ordering::Acquire,
        );

        // Reload after initialization
        let new_ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

        return Some(ptr_to_logger_fn(new_ptr));
    }

    Some(ptr_to_logger_fn(ptr))
}

/// Gets the logger, Returns `None` if not set (no_std).
#[cfg(all(not(feature = "std"), not(feature = "static")))]
#[inline(always)]
fn get_logger() -> Option<LoggerFn> {
    let ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        return None;
    }

    Some(ptr_to_logger_fn(ptr))
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

/// Default stdout logger (`std` feature).
#[cfg(feature = "std")]
#[inline(always)]
fn std_logger_fn(payload: Payload) {
    std::println!("[{}] {}", payload.level.as_str(), payload.args)
}

#[cfg(all(feature = "std", feature = "static"))]
mod stdout {
    use crate::Payload;

    /// Default logger implementation for when the `std` and `static` features are enabled.
    #[no_mangle]
    pub extern "Rust" fn __loggery_log_impl(payload: Payload) {
        super::std_logger_fn(payload);
    }
}
