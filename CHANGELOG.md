# Changelog

All notable changes to ryml will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0]

### Added

- Added `no_std` support. This makes a few methods like `Tree::emit()`, which produced
  owned Strings, dependent upon the new `std` feature.
