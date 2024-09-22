use std::{
    fs::File,
    io::{self, LineWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};

#[track_caller]
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Represents an output sink, which can be either standard output or a file.
///
/// # Examples
///
/// ```rust,no_run
/// use std::io::{self, Write as _};
///
/// use clap::Parser as _;
/// use clap_file::Output;
///
/// #[derive(Debug, clap::Parser)]
/// struct Args {
///     /// output file. If not provided, reads from standard output.
///     output: Output,
/// }
///
/// fn main() -> io::Result<()> {
///     let args = Args::parse();
///     let mut output = args.output.lock();
///     writeln!(&mut output, "Hello, world!")?;
///     Ok(())
/// }
/// ```

// This struct should not implement `Clone`, but clap-derive requires Clone [1].
// So, I added `Clone` to the struct and wrap `File` with `Arc` and `Mutex`.
// This is not the best way to handle this, but it works for now.
//
// [1]: https://github.com/clap-rs/clap/issues/4286
#[derive(Debug, Clone)]
pub struct Output(OutputInner);

#[derive(Debug, Clone)]
enum OutputInner {
    Stdout,
    File {
        path: Arc<PathBuf>,
        writer: Arc<Mutex<LineWriter<File>>>,
    },
}

impl Output {
    /// Creates a new [`Output`] instance that writes to standard output.
    pub fn stdout() -> Self {
        Self(OutputInner::Stdout)
    }

    /// Opens a file at the given path and creates a new [`Output`] instance that writes to it.
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let path = Arc::new(path);
        let file = File::open(&*path)?;
        let writer = Arc::new(Mutex::new(LineWriter::new(file)));
        Ok(Self(OutputInner::File { path, writer }))
    }

    /// Returns `true` if this [`Output`] writes to standard output.
    pub fn is_stdout(&self) -> bool {
        matches!(self.0, OutputInner::Stdout)
    }

    /// Returns `true` if this [`Output`] writes to a file.

    pub fn is_file(&self) -> bool {
        matches!(self.0, OutputInner::File { .. })
    }

    /// Returns the path of the file this [`Output`] writes to.
    ///
    /// Returns `None` if this [`Output`] writes to standard output.
    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            OutputInner::Stdout => None,
            OutputInner::File { path, .. } => Some(path),
        }
    }

    /// Locks this [`Output`] for writing and returns a writable guard.
    ///
    /// This lock is released when the returned [`LockedOutput`] instance is dropped.
    /// The returned `LockedOutput` instance implements [`Write`] trait for writing data.

    pub fn lock(&self) -> LockedOutput<'_> {
        let inner = match &self.0 {
            OutputInner::Stdout => {
                let writer = io::stdout().lock();
                LockedOutputInner::Stdout { writer }
            }
            OutputInner::File { path, writer: file } => {
                let writer = lock(file);
                LockedOutputInner::File {
                    path: Arc::clone(path),
                    writer,
                }
            }
        };
        LockedOutput(inner)
    }
}

impl FromStr for Output {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            return Ok(Self::stdout());
        }
        Self::open(PathBuf::from(s))
    }
}

macro_rules! with_writer {
    ($inner:expr, $var:ident => $e:expr) => {
        match $inner {
            OutputInner::Stdout => {
                let mut $var = io::stdout();
                $e
            }
            OutputInner::File { writer, .. } => {
                let mut $var = lock(writer);
                $e
            }
        }
    };
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        with_writer!(&self.0, writer => writer.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        with_writer!(&self.0, writer => writer.flush())
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        with_writer!(&self.0, writer => writer.write_vectored(bufs))
    }

    // this method is not yet stable
    // fn is_write_vectored(&self) -> bool {
    //     with_writer!(&self.0, writer => writer.is_write_vectored())
    // }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        with_writer!(&self.0, writer => writer.write_all(buf))
    }

    // this method is not yet stable
    // fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
    //     with_writer!(&self.0, writer => writer.write_all_vectored(bufs))
    // }
}

/// A locked output sink that can be written to.
#[derive(Debug)]
pub struct LockedOutput<'a>(LockedOutputInner<'a>);

impl LockedOutput<'_> {
    /// Returns `true` if this [`LockedOutput`] writes to standard output.
    pub fn is_stdin(&self) -> bool {
        matches!(self.0, LockedOutputInner::Stdout { .. })
    }

    /// Returns `true` if this [`LockedOutput`] writes to a file.
    pub fn is_file(&self) -> bool {
        matches!(self.0, LockedOutputInner::File { .. })
    }

    /// Returns the path of the file this [`LockedOutput`] writes to.
    ///
    /// Returns `None` if this [`LockedOutput`] writes to standard output.
    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            LockedOutputInner::Stdout { .. } => None,
            LockedOutputInner::File { path, .. } => Some(path),
        }
    }
}

#[derive(Debug)]
enum LockedOutputInner<'a> {
    Stdout {
        writer: io::StdoutLock<'a>,
    },
    File {
        path: Arc<PathBuf>,
        writer: MutexGuard<'a, LineWriter<File>>,
    },
}

macro_rules! with_locked_writer {
    ($inner:expr, $var:ident => $e:expr) => {
        match $inner {
            LockedOutputInner::Stdout { writer } => {
                let $var = writer;
                $e
            }
            LockedOutputInner::File { writer, .. } => {
                let $var = writer;
                $e
            }
        }
    };
}

impl Write for LockedOutput<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        with_locked_writer!(&mut self.0, writer => writer.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        with_locked_writer!(&mut self.0, writer => writer.flush())
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        with_locked_writer!(&mut self.0, writer => writer.write_vectored(bufs))
    }

    // this method is not yet stable
    // fn is_write_vectored(&self) -> bool {
    //     with_locked_writer!(&self.0, writer => writer.is_write_vectored())
    // }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        with_locked_writer!(&mut self.0, writer => writer.write_all(buf))
    }

    // this method is not yet stable
    // fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
    //     with_locked_writer!(&mut self.0, writer => writer.write_all_vectored(bufs))
    // }
}
