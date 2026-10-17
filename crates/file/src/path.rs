use std::path::{Component, Path as StdPath, PathBuf};
use error::Result;

use crate::{Faults, Path};

impl Path {

    /// `root` joined with a relative `inside`, refusing anything that could leave the root: an absolute
    /// path, a drive or root prefix, or a `..` that climbs above where it started.
    ///
    /// # Errors
    /// `Forbidden` keyed `escapes_root`.
    #[inline]
    pub fn confine ( root: &StdPath, inside: &StdPath ) -> Result<PathBuf> {

        let mut depth = 0usize;
        let mut joined = root.to_path_buf();

        for component in inside.components() {

            depth = Self::step(&mut joined, component, depth)?;

        }

        Ok(joined)

    }

    /// The path with `.` dropped and `..` folded where it can be, without touching the file system.
    #[inline]
    #[must_use]
    pub fn normalize ( path: &StdPath ) -> PathBuf {

        path.components().fold(PathBuf::new(), Self::fold)

    }

    /// The extension, lower-cased, without the dot.
    #[inline]
    #[must_use]
    pub fn extension ( path: &StdPath ) -> Option<String> {

        path.extension().and_then(|extension| extension.to_str()).map(str::to_ascii_lowercase)

    }

    /// The file name without its extension.
    #[inline]
    #[must_use]
    pub fn stem ( path: &StdPath ) -> Option<&str> {

        path.file_stem().and_then(|stem| stem.to_str())

    }

    /// Whether the last component starts with a dot.
    #[inline]
    #[must_use]
    pub fn is_hidden ( path: &StdPath ) -> bool {

        path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with('.'))

    }

    fn step ( joined: &mut PathBuf, component: Component<'_>, depth: usize ) -> Result<usize> {

        match component {
            Component::Normal(part)                 => { joined.push(part); Ok(depth + 1) }
            Component::CurDir                       => Ok(depth),
            Component::ParentDir if depth > 0       => { joined.pop(); Ok(depth - 1) }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => Err(Faults::ESCAPES_ROOT.fail()),
        }

    }

    fn fold ( mut out: PathBuf, component: Component<'_> ) -> PathBuf {

        match component {
            Component::CurDir                                                  => {}
            Component::ParentDir if Self::pops(&out)                           => { out.pop(); }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) | Component::Normal(_) => out.push(component),
        }

        out

    }

    fn pops ( out: &StdPath ) -> bool {

        matches!(out.components().next_back(), Some(Component::Normal(_)))

    }

}
