# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

* **(Breaking Change)** Rename `LockedOutput::is_stdin` to `LockedOutput::is_stdout`
* Bump MSRV to 1.88
* Improve documentation examples and fix typos

## [0.2.0] - 2024-09-22

* **(Breaking Change)** Rename `Output::open` to `Output::create` and use `File::create` to create the file

## [0.1.0] - 2024-09-22

* First release

<!-- next-url -->
[Unreleased]: https://github.com/gifnksm/clap-file/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gifnksm/clap-file/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gifnksm/clap-file/commits/v0.1.0
