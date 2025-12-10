#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub type LoggerFn = fn(Level, core::fmt::Arguments);

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => " INFO",
            Level::Warn => " WARN",
            Level::Error => "ERROR",
        }
    }

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

static LOGGER_FN: core::sync::atomic::AtomicPtr<()> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

const MIN_LEVEL: Option<u8> = match () {
    _ if cfg!(feature = "min_level_off") => None,
    _ if cfg!(feature = "min_level_trace") => Some(Level::Trace as u8),
    _ if cfg!(feature = "min_level_debug") => Some(Level::Debug as u8),
    _ if cfg!(feature = "min_level_info") => Some(Level::Info as u8),
    _ if cfg!(feature = "min_level_warn") => Some(Level::Warn as u8),
    _ if cfg!(feature = "min_level_error") => Some(Level::Error as u8),
    _ => Some(Level::Trace as u8), // By default, allow all logs
};

#[inline(always)]
pub fn set_logger(logger_fn: LoggerFn) {
    LOGGER_FN.store(
        logger_fn as usize as *mut (),
        core::sync::atomic::Ordering::Release,
    )
}

#[inline(always)]
pub fn get_min_level() -> Option<Level> {
    Level::from_u8(MIN_LEVEL?)
}

#[inline(always)]
pub fn log(level: Level, args: core::fmt::Arguments) {
    let is_level_enabled = match MIN_LEVEL {
        Some(min_level) => (level as u8) >= min_level,
        None => false,
    };

    if is_level_enabled && let Some(logger_fn) = get_logger() {
        logger_fn(level, args)
    }
}

#[inline(always)]
fn ptr_to_logger_fn(ptr: *mut ()) -> LoggerFn {
    // SAFETY: `ptr` was created from `LoggerFn` in `set_logger`. Function pointers are 'static.
    // Atmoics ensure cross-thread visibility.
    unsafe { core::mem::transmute_copy::<*mut (), LoggerFn>(&ptr) }
}

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

#[cfg(not(feature = "std"))]
#[inline(always)]
fn get_logger() -> Option<LoggerFn> {
    let ptr = LOGGER_FN.load(core::sync::atomic::Ordering::Acquire);

    if ptr.is_null() {
        return None;
    }

    Some(ptr_to_logger_fn(ptr))
}

#[cfg(feature = "std")]
fn std_logger_fn(level: Level, args: core::fmt::Arguments) {
    std::println!("[{}] {}", level.as_str(), args)
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Trace, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Debug, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Warn, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Error, format_args!($($arg)*))
    };
}
