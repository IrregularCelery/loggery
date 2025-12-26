# Changelog

## [Unreleased]

## [0.1.0] - 2025-12-26

### Added

- Initial release of `loggery`
- Core logging macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`
- Full `no_std` support for embedded and bare-metal environments
- Compile-time log level filtering via `min_level_*` feature flags
  (`min_level_off`, `min_level_trace`, `min_level_debug`, `min_level_info`, `min_level_warn`,
  `min_level_error`)
- Runtime log level control with `set_min_level()` and `get_min_level()` functions
  (requires `runtime_level` feature, _enabled by default_)
- Custom logger support via `set_logger()` function for dynamic logger registration
- Static logger mode via `static` feature for performance intensive environments with direct
  compile-time linking (requires defining `__loggery_log_impl` function)
- Built-in stdout logger when `std` feature is enabled (_enabled by default_)
- Extension system via `set_extension()` for hooking additional functionality before logging
  (requires `extension` feature)
- Static extension mode when both `static` and `extension` features are enabled
  (requires defining `__loggery_extension_impl` function)
- Built-in file logging extension via `loggery::extensions::save_to_file()`
  (requires `std` + `extension` features)
- Optional metadata capture (module path, file name, line number) via `metadata` feature
  (_enabled by default_)
- MSRV: Rust 1.56.0

[Unreleased]: https://github.com/IrregularCelery/loggery/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/IrregularCelery/loggery/releases/tag/v0.1.0
