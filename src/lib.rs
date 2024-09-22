//! Provides types for clap's derive interface, enabling easy handling of input/output with automatically opened files or standard input/output based on command-line arguments.
//!
//! # Usage
//!
//! Run `cargo add clap-file` or add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! clap-file = "0.0.0"
//! ```
//!
//! # Examples
//!
//! Example usage of [`Input`] ans [`Output`] types:
//!
//! ```rust,no_run
//! use std::io::{self, BufRead as _, Write as _};
//!
//! use clap::Parser as _;
//! use clap_file::{Input, Output};
//!
//! #[derive(Debug, clap::Parser)]
//! struct Args {
//!     /// Input file. If not provided, reads from standard input.
//!     input: Input,
//!     /// output file. If not provided, reads from standard output.
//!     output: Output,
//! }
//!
//! fn main() -> io::Result<()> {
//!     let args = Args::parse();
//!     let input = args.input.lock();
//!     let mut output = args.output.lock();
//!     for line in input.lines() {
//!         let line = line?;
//!         writeln!(&mut output, "{line}")?;
//!     }
//!     Ok(())
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/clap-file/0.0.0")]
#![warn(missing_docs)]

pub use self::{input::*, output::*};

mod input;
mod output;
