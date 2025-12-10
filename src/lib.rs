//! A lightweight, `no_std`-friendly logging library for Rust.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

/// Function type for custom logger implementation.
pub type LoggerFn = fn(Level, core::fmt::Arguments);

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
    /// Returns the string representation with consistent width for aligned output.
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

/// Compile-time minimum log level set by feature flags.
const COMPILE_TIME_MIN_LEVEL: Option<u8> = match () {
    _ if cfg!(feature = "min_level_off") => None,
    _ if cfg!(feature = "min_level_trace") => Some(Level::Trace as u8),
    _ if cfg!(feature = "min_level_debug") => Some(Level::Debug as u8),
    _ if cfg!(feature = "min_level_info") => Some(Level::Info as u8),
    _ if cfg!(feature = "min_level_warn") => Some(Level::Warn as u8),
    _ if cfg!(feature = "min_level_error") => Some(Level::Error as u8),
    _ => Some(Level::Trace as u8), // By default, allow all logs
};

/// Global logger function pointer storage.
static LOGGER_FN: core::sync::atomic::AtomicPtr<()> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
/// Runtime minimum log level storage.
static RUNTIME_MIN_LEVEL: core::sync::atomic::AtomicU8 =
    core::sync::atomic::AtomicU8::new(Level::Trace as u8);

/// Sets the global logger function.
///
/// It's recommended to call once during the initialization.
///
/// # Example
/// ```
/// use loggery::{Level, debug};
///
/// fn my_custom_logger(level: Level, args:core::fmt::Arguments) {
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
/// When the `std` feature is enabled, a default logger is automatically initialized if no logger
/// has been set. This function can still be used to override that default.
#[inline(always)]
pub fn set_logger(logger_fn: LoggerFn) {
    LOGGER_FN.store(
        logger_fn as usize as *mut (),
        core::sync::atomic::Ordering::Release,
    )
}

/// Set the runtime minimum log level.
///
/// Note
/// This cannot enabled levels that were filtered at compile time.
/// If compiled with feature `min_level_info`, calling `set_min_level(Level::Debug)` will have
/// no effect.
///
/// # Example
/// ```
/// use loggery::{Level, debug, warn};
///
/// loggery::set_min_level(Level::Warn);
///
/// debug!("This will NOT be logged");
/// warn!("This will be logged");
/// ```
#[inline(always)]
pub fn set_min_level(level: Level) {
    RUNTIME_MIN_LEVEL.store(level as u8, core::sync::atomic::Ordering::Release);
}

/// Returns the effective minimum log level (the stricter of compile-time and runtime levels).
///
/// # Example
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
    let runtime = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);

    // Return whichever is stricter
    Level::from_u8(compile_time.max(runtime))
}

/// Core logging function.
///
/// It's recommended to use the macros instead of calling directly.
#[inline(always)]
pub fn log(level: Level, args: core::fmt::Arguments) {
    let is_compile_time_enabled = match COMPILE_TIME_MIN_LEVEL {
        Some(min_level) => (level as u8) >= min_level,
        None => false,
    };

    let runtime_min_level = RUNTIME_MIN_LEVEL.load(core::sync::atomic::Ordering::Relaxed);
    let is_runtime_enabled = (level as u8) >= runtime_min_level;

    if is_compile_time_enabled
        && is_runtime_enabled
        && let Some(logger_fn) = get_logger()
    {
        logger_fn(level, args)
    }
}

/// Converts a raw pointer back to a `LoggerFn`.
///
/// # Safety
/// Safe only when `ptr` was created by casting a valid `LoggerFn` to `*mut ()`.
/// The caller must ensure:
/// - Pointer originated from a valid function pointer cast
/// - Fnuction pointer has 'static lifetime (guaranteed for all fn pointers)
/// - Proper synchronization (handled by atomic ops)
#[inline(always)]
fn ptr_to_logger_fn(ptr: *mut ()) -> LoggerFn {
    // SAFETY: `ptr` was created from `LoggerFn` in `set_logger`. Function pointers are 'static.
    // Atomics ensure cross-thread visibility.
    unsafe { core::mem::transmute::<*mut (), LoggerFn>(ptr) }
}

/// Gets the logger, auto-initializing a default stdout logger if needed (std only).
#[cfg(feature = "std")]
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
#[cfg(not(feature = "std"))]
#[inline(always)]
fn get_logger() -> Option<LoggerFn> {
    let ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        return None;
    }

    Some(ptr_to_logger_fn(ptr))
}

/// Default stdout logger (std only).
#[cfg(feature = "std")]
fn std_logger_fn(level: Level, args: core::fmt::Arguments) {
    std::println!("[{}] {}", level.as_str(), args)
}

/// Logs a message at the `trace` level.
///
/// # Example
/// ```
/// use loggery::trace;
///
/// trace!("Entering function...");
/// ```
///
/// # Compile-time filtering
/// If feature `min_level_debug` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Trace, format_args!($($arg)*))
    };
}

/// Logs a message at the `debug` level.
///
/// # Example
/// ```
/// use loggery::debug;
///
/// let x = 69;
///
/// debug!("Variable `x` is {}", x);
/// ```
///
/// # Compile-time filtering
/// If feature `min_level_info` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Debug, format_args!($($arg)*))
    };
}

/// Logs a message at the `info` level.
///
/// # Example
/// ```
/// use loggery::info;
///
/// const PORT: u16 = 8080;
///
/// info!("Server is started on port {}", PORT);
/// ```
///
/// # Compile-time filtering
/// If feature `min_level_warn` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Info, format_args!($($arg)*))
    };
}

/// Logs a message at the `warn` level.
///
/// # Example
/// ```
/// use loggery::warn;
///
/// warn!("Configuration file not found, using defaults");
/// ```
///
/// # Compile-time filtering
/// If feature `min_level_error` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Warn, format_args!($($arg)*))
    };
}

/// Logs a message at the `error` level.
///
/// # Example
/// ```
/// use loggery::error;
///
/// error!("Failed to connect to database!");
/// ```
///
/// # Compile-time filtering
/// If feature `min_level_off` or higher is enabled, this compiles to nothing in release builds
/// with optimizations.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Error, format_args!($($arg)*))
    };
}
