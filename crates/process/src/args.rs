use error::{Failure, Result};
use std::env::args_os;

use crate::{Args, Faults};

impl Args {

    /// The process's own command line.
    #[inline]
    #[must_use]
    pub fn current () -> Self {

        Self::from_iter(args_os().map(|arg| arg.to_string_lossy().into_owned()))

    }

    /// A command line from words, the first being the program.
    #[inline]
    #[must_use]
    pub fn from_words ( words: &[&str] ) -> Self {

        Self::from_iter(words.iter().map(|word| (*word).to_owned()))

    }

    /// The program as invoked.
    #[inline]
    #[must_use]
    pub fn program ( &self ) -> &str {

        &self.program

    }

    /// Whether nothing at all followed the program: no command, no flag, no word.
    #[inline]
    #[must_use]
    pub const fn is_bare ( &self ) -> bool {

        self.command.is_none() && self.flags.is_empty() && self.rest.is_empty()

    }

    /// The first bare word, when there is one.
    #[inline]
    #[must_use]
    pub fn command ( &self ) -> Option<&str> {

        self.command.as_deref()

    }

    /// Whether `--name` or `-n` was given, with or without a value.
    #[inline]
    #[must_use]
    pub fn has ( &self, name: &str ) -> bool {

        self.flags.iter().any(|( flag, _ )| flag == name)

    }

    /// `<name> <version>` and a newline when `--version` was given, for a binary to answer before anything else.
    #[inline]
    #[must_use]
    pub fn version ( &self, name: &str, version: &str ) -> Option<String> {

        self.has("version").then(|| format!("{name} {version}\n"))

    }

    /// The value of `--name value` or `--name=value`, the last one when repeated.
    #[inline]
    #[must_use]
    pub fn get ( &self, name: &str ) -> Option<&str> {

        self.flags.iter().rev().find(|( flag, _ )| flag == name).and_then(|( _, value )| value.as_deref())

    }

    /// Every value given under `name`, in order.
    #[inline]
    pub fn all <'a> ( &'a self, name: &'a str ) -> impl Iterator<Item = &'a str> {

        self.flags.iter().filter(move |( flag, _ )| flag == name).filter_map(|( _, value )| value.as_deref())

    }

    /// The value of `--name`, or a `Usage` failure naming it.
    ///
    /// # Errors
    /// The flag is absent or has no value.
    #[inline]
    pub fn need ( &self, name: &str ) -> Result<&str> {

        self.get(name).ok_or_else(|| Faults::FLAG_MISSING.because(format!("--{name} is required")).about(name.to_owned()))

    }

    /// The bare words after the command, in order.
    #[inline]
    #[must_use]
    pub fn rest ( &self ) -> &[String] {

        &self.rest

    }

    /// The `index`-th bare word after the command.
    #[inline]
    #[must_use]
    pub fn positional ( &self, index: usize ) -> Option<&str> {

        self.rest.get(index).map(String::as_str)

    }

    /// A `Usage` failure for a command this tool does not have.
    #[inline]
    #[must_use]
    pub fn unknown ( &self ) -> Failure {

        Faults::UNKNOWN_COMMAND.because(format!("{} does not know the command {}", self.program, self.command.as_deref().unwrap_or("(none)")))

    }

    fn from_iter ( words: impl Iterator<Item = String> ) -> Self {

        let mut args  = Self::default();
        let mut words = words;
        let mut only  = false;

        args.program = words.next().unwrap_or_default();

        for word in words { only = args.take(word, only); }

        args

    }

    fn take ( &mut self, word: String, only: bool ) -> bool {

        if only || !word.starts_with('-') || word == "-" { return self.bare(word, only); }

        if word == "--" { return true; }

        let body = word.trim_start_matches('-');

        match body.split_once('=') {
            Some(( name, value )) => self.flags.push(( name.to_owned(), Some(value.to_owned()) )),
            None                  => self.flags.push(( body.to_owned(), None )),
        }

        false

    }

    fn bare ( &mut self, word: String, only: bool ) -> bool {

        match ( self.command.is_none(), self.flags.last_mut() ) {
            ( _, Some(( _, slot @ None )) ) if !only => *slot = Some(word),
            ( true, _ )                               => self.command = Some(word),
            ( false, _ )                              => self.rest.push(word),
        }

        only

    }

}
