use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};

#[track_caller]
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Represents an input source, which can be either standard input or a file.
///
/// # Examples
///
/// ```rust,no_run
/// use std::io::{self, BufRead as _};
///
/// use clap::Parser as _;
/// use clap_file::Input;
///
/// #[derive(Debug, clap::Parser)]
/// struct Args {
///     /// Input file. If not provided, reads from standard input.
///     input: Input,
/// }
///
/// fn main() -> io::Result<()> {
///     let args = Args::parse();
///     let input = args.input.lock();
///     for line in input.lines() {
///         let line = line?;
///         println!("{line}");
///     }
///     Ok(())
/// }
/// ```

// This struct should not implement `Clone`, but clap-derive requires Clone [1].
// So, I added `Clone` to the struct and wrap `File` with `Arc` and `Mutex`.
// This is not the best way to handle this, but it works for now.
//
// [1]: https://github.com/clap-rs/clap/issues/4286
#[derive(Debug, Clone)]
pub struct Input(InputInner);

#[derive(Debug, Clone)]
enum InputInner {
    Stdin,
    File {
        path: Arc<PathBuf>,
        reader: Arc<Mutex<BufReader<File>>>,
    },
}

impl Input {
    /// Creates a new [`Input`] instance that reads from standard input.
    pub fn stdin() -> Self {
        Self(InputInner::Stdin)
    }

    /// Opens a file at the given path and creates a new [`Input`] instance that reads from it.
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let path = Arc::new(path);
        let file = File::open(&*path)?;
        let reader = Arc::new(Mutex::new(BufReader::new(file)));
        Ok(Self(InputInner::File { path, reader }))
    }

    /// Returns `true` if this [`Input`] reads from standard input.
    pub fn is_stdin(&self) -> bool {
        matches!(self.0, InputInner::Stdin)
    }

    /// Returns `true` if this [`Input`] reads from a file.

    pub fn is_file(&self) -> bool {
        matches!(self.0, InputInner::File { .. })
    }

    /// Returns the path of the file this [`Input`] reads from.
    ///
    /// Returns `None` if this [`Input`] reads from standard input.
    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            InputInner::Stdin => None,
            InputInner::File { path, .. } => Some(path),
        }
    }

    /// Locks the input source and returns a [`LockedInput`] instance.
    ///
    /// This lock is released when the returned [`LockedInput`] instance is dropped.
    /// The returned `LockedInput` instance implements [`Read`] and [`BufRead`] traits.

    pub fn lock(&self) -> LockedInput<'_> {
        let inner = match &self.0 {
            InputInner::Stdin => {
                let reader = io::stdin().lock();
                LockedInputInner::Stdin { reader }
            }
            InputInner::File { path, reader: file } => {
                let reader = lock(file);
                LockedInputInner::File {
                    path: Arc::clone(path),
                    reader,
                }
            }
        };
        LockedInput(inner)
    }
}

impl FromStr for Input {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            return Ok(Self::stdin());
        }
        Self::open(PathBuf::from(s))
    }
}

macro_rules! with_reader {
    ($inner:expr, $var:ident => $e:expr) => {
        match $inner {
            InputInner::Stdin => {
                let mut $var = io::stdin();
                $e
            }
            InputInner::File { reader, .. } => {
                let mut $var = lock(reader);
                $e
            }
        }
    };
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        with_reader!(&self.0, r => r.read(buf))
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        with_reader!(&self.0, r => r.read_vectored(bufs))
    }

    // this method is not yet stable
    // fn is_read_vectored(&self) -> bool {
    //     with_reader!(&self.0, r => r.is_read_vectored())
    // }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        with_reader!(&self.0, r => r.read_to_end(buf))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        with_reader!(&self.0, r => r.read_to_string(buf))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        with_reader!(&self.0, r => r.read_exact(buf))
    }

    // this method is not yet stable
    // fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
    //     with_reader!(&self.0, r => r.read_buf(buf))
    // }

    // this method is not yet stable
    // fn read_buf_exact(&mut self, cursor: io::BorrowedCursor<'_>) -> io::Result<()> {
    //     with_reader!(&self.0, r => r.read_buf_exact(cursor))
    // }
}

/// A locked input source that implements [`Read`] and [`BufRead`] traits.
#[derive(Debug)]
pub struct LockedInput<'a>(LockedInputInner<'a>);

impl LockedInput<'_> {
    /// Returns `true` if this [`LockedInput`] reads from standard input.
    pub fn is_stdin(&self) -> bool {
        matches!(self.0, LockedInputInner::Stdin { .. })
    }

    /// Returns `true` if this [`LockedInput`] reads from a file.
    pub fn is_file(&self) -> bool {
        matches!(self.0, LockedInputInner::File { .. })
    }

    /// Returns the path of the file this [`LockedInput`] reads from.
    ///
    /// Returns `None` if this [`LockedInput`] reads from standard input.
    pub fn path(&self) -> Option<&Path> {
        match &self.0 {
            LockedInputInner::Stdin { .. } => None,
            LockedInputInner::File { path, .. } => Some(path),
        }
    }
}

#[derive(Debug)]
enum LockedInputInner<'a> {
    Stdin {
        reader: io::StdinLock<'a>,
    },
    File {
        path: Arc<PathBuf>,
        reader: MutexGuard<'a, BufReader<File>>,
    },
}

macro_rules! with_locked_reader {
    ($inner:expr, $var:ident => $e:expr) => {
        match $inner {
            LockedInputInner::Stdin { reader } => {
                let $var = reader;
                $e
            }
            LockedInputInner::File { reader, .. } => {
                let $var = reader;
                $e
            }
        }
    };
}

impl Read for LockedInput<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        with_locked_reader!(&mut self.0, r => r.read(buf))
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        with_locked_reader!(&mut self.0, r => r.read_vectored(bufs))
    }

    // this method is not yet stable
    // fn is_read_vectored(&self) -> bool {
    //     with_locked_reader!(&mut self.0, r => r.is_read_vectored())
    // }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        with_locked_reader!(&mut self.0, r => r.read_to_end(buf))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        with_locked_reader!(&mut self.0, r => r.read_to_string(buf))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        with_locked_reader!(&mut self.0, r => r.read_exact(buf))
    }

    // this method is not yet stable
    // fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
    //     with_locked_reader!(&mut self.0, r => r.read_buf(buf))
    // }

    // this method is not yet stable
    // fn read_buf_exact(&mut self, cursor: io::BorrowedCursor<'_>) -> io::Result<()> {
    //     with_locked_reader!(&mut self.0, r => r.read_buf_exact(cursor))
    // }
}

impl BufRead for LockedInput<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        with_locked_reader!(&mut self.0, r => r.fill_buf())
    }

    fn consume(&mut self, amt: usize) {
        with_locked_reader!(&mut self.0, r => r.consume(amt))
    }
}
