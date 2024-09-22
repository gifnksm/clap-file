<!-- cargo-sync-rdme title [[ -->
# clap-file
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
[![Maintenance: passively-maintained](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/clap-file.svg?style=flat-square)](#license)
[![crates.io](https://img.shields.io/crates/v/clap-file.svg?logo=rust&style=flat-square)](https://crates.io/crates/clap-file)
[![docs.rs](https://img.shields.io/docsrs/clap-file.svg?logo=docs.rs&style=flat-square)](https://docs.rs/clap-file)
[![Rust: ^1.74.0](https://img.shields.io/badge/rust-^1.74.0-93450a.svg?logo=rust&style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/clap-file/ci.yml.svg?label=CI&logo=github&style=flat-square)](https://github.com/gifnksm/clap-file/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/clap-file.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/gifnksm/clap-file)
<!-- cargo-sync-rdme ]] -->

<!-- cargo-sync-rdme rustdoc [[ -->
Provides types for clap’s derive interface, enabling easy handling of input/output with automatically opened files or standard input/output based on command-line arguments.

## Usage

Run `cargo add clap-file` or add this to your `Cargo.toml`:

````toml
[dependencies]
clap-file = "0.2.0"
````

## Examples

Example usage of [`Input`](https://docs.rs/clap-file/0.2.0/clap_file/input/struct.Input.html) ans [`Output`](https://docs.rs/clap-file/0.2.0/clap_file/output/struct.Output.html) types:

````rust,no_run
use std::io::{self, BufRead as _, Write as _};

use clap::Parser as _;
use clap_file::{Input, Output};

struct Args {
    /// Input file. If not provided, reads from standard input.
    input: Input,
    /// output file. If not provided, reads from standard output.
    output: Output,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let input = args.input.lock();
    let mut output = args.output.lock();
    for line in input.lines() {
        let line = line?;
        writeln!(&mut output, "{line}")?;
    }
    Ok(())
}
````
<!-- cargo-sync-rdme ]] -->

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.74.0**.
At least the last 3 versions of stable Rust are supported at any given time.

While a crate is a pre-release status (0.x.x) it may have its MSRV bumped in a patch release.
Once a crate has reached 1.x, any MSRV bump will be accompanied by a new minor version.

## License

This project is licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
