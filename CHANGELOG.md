# Changelog
All notable changes to `fon` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://github.com/AldaronLau/semver).

## [0.2.0] - Unreleased
### Added
 - `From<Audio> for Box<[f64]>` impl
 - `From<Audio> for Box<[f32]>` impl
 - `From<Audio> for Box<[i8]>` impl
 - `From<Audio> for Box<[i16]>` impl
 - `Audio::samples` method
 - `Audio::samples_mut` method
 - `Audio::with_i8_buffer` function
 - `Audio::with_i16_buffer` function
 - `Audio::with_f32_buffer` function
 - `Audio::with_f64_buffer` function
 - `Audio::copy_silence` method
 - `Audio::copy_sample` method
 - `Surround4` and `Surround4x{}` aliases
 - `Audio::sample` method
 - `Audio::sample_mut` method
 - `Resampler` struct
 - `Stream` trait
 - `Sink` trait
 - `Audio::stream` method
 - `Audio::drain` method

### Changed
 - `Audio::blend_sample` and `Audio::blend_audio` now take an additional `reg`
   argument to enable blending on part of the `Audio` buffer.
 - Rename `Surround` to `Surround5`
 - Rename `SurroundHD` to `Surround7`
 - Rename `Surround{}` to `Surround5x{}`
 - Rename `SurroundHD{}` to `Surround7x{}`
 - Speaker (channel) configuration order now matches the default on Linux ALSA

### Removed
 - `From<Audio> for Box<[u8]>` impl
 - `From<Audio> for Box<[u16]>` impl
 - `Audio::as_u8_slice` method
 - `Audio::as_u8_slice_mut` method
 - `Audio::with_u8_buffer` function
 - `Audio::with_u16_buffer` function
 - `Hz` type

### Fixed
 - Channel and Sample types not being marked `#[repr(transparent)]`

## [0.1.0] - 2020-08-15
### Added
 - `Audio` buffer
 - `Hz` newtype
 - `Config` trait
 - `mono`, `stereo` and `surround` modules
 - `Sample1`, `Sample2`, `Sample6`, and `Sample8` implementing `Sample`
 - `ops` module with `Amplify`, `Clear`, `Compress`, `Dest`, `Mix` and `Src`
 - `Ch8`, `Ch16`, `Ch32` and `Ch64` implementing `Channel`
