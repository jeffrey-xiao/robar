# Changelog

## 2.0.1 - 2021-09-13

### Fixed

- Fix crash on server start.

## 2.0.0 - 2021-09-13

### Changed

- Change the `robar show` interface to take in an integer in the range [0, 100] instead of a float
  in the range [0, 1].
- Optimize rendering of bar.

### Added

- Add `robar show-stream` that accepts newline separated updates in the form of `<profile> <value>`
  from standard input. This interface is intended to be used with streams or subscriptions (e.g.
  `pactl subscribe`).

## 1.0.0 - 2018-12-23

### Changed

- Upgrade to Rust 2018.

## 0.1.0 - 2018-09-18

### Added

- Initial functional bar.
