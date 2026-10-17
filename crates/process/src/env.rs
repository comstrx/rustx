use error::{Code, Result};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use parse::{Lines, Parse};
use core::time::Duration;
use std::env::var_os;
use file::File;

use crate::arch::Parser;
use crate::{Env, Faults};

impl Env {

    /// The process environment alone.
    #[inline]
    #[must_use]
    pub fn system () -> Self {

        Self::default()

    }

    /// The process environment under a dotenv file; a missing file is fine, a variable already
    /// set in the process wins over the file.
    ///
    /// # Errors
    /// The file exists and cannot be read.
    #[inline]
    pub async fn with_file ( path: &Path ) -> Result<Self> {

        if !File::exists(path).await { return Ok(Self::default()); }

        let overlay = Self::parse(&String::from_utf8_lossy(&File::read(path).await?));

        Ok(Self { overlay })

    }

    /// The value of `name`, as text.
    #[inline]
    #[must_use]
    pub fn text ( &self, name: &str ) -> Option<String> {

        var_os(name).map(|value| value.to_string_lossy().into_owned()).or_else(|| self.overlay.get(name).cloned())

    }

    /// Whether `name` is set at all.
    #[inline]
    #[must_use]
    pub fn has ( &self, name: &str ) -> bool {

        var_os(name).is_some() || self.overlay.contains_key(name)

    }

    /// The value of `name`, or a `Config` failure naming it.
    ///
    /// # Errors
    /// The variable is not set.
    #[inline]
    pub fn need ( &self, name: &str ) -> Result<String> {

        self.text(name).ok_or_else(|| Faults::ENV_MISSING.because(format!("{name} is not set")).about(name.to_owned()))

    }

    /// A `u16`, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn u16 ( &self, name: &str, fallback: u16 ) -> Result<u16> {

        self.typed(name, fallback, Parse::u16)

    }

    /// A `u64`, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn u64 ( &self, name: &str, fallback: u64 ) -> Result<u64> {

        self.typed(name, fallback, Parse::u64)

    }

    /// A `usize`, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn usize ( &self, name: &str, fallback: usize ) -> Result<usize> {

        self.typed(name, fallback, Parse::usize)

    }

    /// A flag, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn bool ( &self, name: &str, fallback: bool ) -> Result<bool> {

        self.typed(name, fallback, Parse::bool)

    }

    /// A duration such as `30s`, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn duration ( &self, name: &str, fallback: Duration ) -> Result<Duration> {

        self.typed(name, fallback, Parse::duration)

    }

    /// A size such as `64mb`, `fallback` when unset.
    ///
    /// # Errors
    /// The value is set and is not one.
    #[inline]
    pub fn size ( &self, name: &str, fallback: u64 ) -> Result<u64> {

        self.typed(name, fallback, Parse::size)

    }

    /// A path, `fallback` when unset.
    #[inline]
    #[must_use]
    pub fn path ( &self, name: &str, fallback: impl Into<PathBuf> ) -> PathBuf {

        self.text(name).map_or_else(|| fallback.into(), PathBuf::from)

    }

    /// How many variables the file added.
    #[inline]
    #[must_use]
    pub fn loaded ( &self ) -> usize {

        self.overlay.len()

    }

    fn typed <T> ( &self, name: &str, fallback: T, parse: Parser<T> ) -> Result<T> {

        let Some(text) = self.text(name) else { return Ok(fallback); };

        parse(&text).map_err(|failure| failure.coded(Code::Config).about(name.to_owned()).context(format!("{name}={text}")))

    }

    fn parse ( text: &str ) -> HashMap<String, String> {

        Lines::new(text).map(|( key, value )| ( key.to_owned(), value.to_owned() )).collect()

    }

}
