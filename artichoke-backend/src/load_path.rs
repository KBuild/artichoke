//! Virtual filesystem.
//!
//! Artichoke proxies all filesystem access through a virtual filesystem. The
//! filesystem can store Ruby sources and [extension hooks] in memory and will
//! support proxying to the host filesystem for reads and writes.
//!
//! Artichoke uses the virtual filesystem to track metadata about loaded
//! features.
//!
//! Artichoke has several virtual filesystem implementations. Only some of them
//! support reading from the system fs.
//!
//! [extension hooks]: ExtensionHook

use bstr::ByteVec;
use std::path::{Component, Path, PathBuf};

use crate::error::Error;
use crate::platform_string::ConvertBytesError;
use crate::Artichoke;

#[cfg(feature = "load-path-native-filesystem-loader")]
mod hybrid;
mod memory;
#[cfg(feature = "load-path-native-filesystem-loader")]
mod native;
#[cfg(feature = "load-path-rubylib-native-filesystem-loader")]
mod rubylib;

#[cfg(feature = "load-path-native-filesystem-loader")]
pub use hybrid::Hybrid;
pub use memory::Memory;
#[cfg(feature = "load-path-native-filesystem-loader")]
pub use native::Native;
#[cfg(feature = "load-path-rubylib-native-filesystem-loader")]
pub use rubylib::Rubylib;

/// Directory at which Ruby sources and extensions are stored in the virtual
/// filesystem.
///
/// `RUBY_LOAD_PATH` is the default current working directory for
/// [`Memory`] filesystems.
///
/// [`Hybrid`] filesystems locate the this path on a [`Memory`] filesystem.
#[cfg(not(windows))]
pub const RUBY_LOAD_PATH: &str = "/artichoke/virtual_root/src/lib";

/// Directory at which Ruby sources and extensions are stored in the virtual
/// filesystem.
///
/// `RUBY_LOAD_PATH` is the default current working directory for
/// [`Memory`] filesystems.
///
/// [`Hybrid`] filesystems locate the this path on a [`Memory`] filesystem.
#[cfg(windows)]
pub const RUBY_LOAD_PATH: &str = "c:/artichoke/virtual_root/src/lib";

/// Function type for extension hooks stored in the virtual filesystem.
///
/// This signature is equivalent to the signature for [`File::require`] as
/// defined by the `artichoke-backend` implementation of [`LoadSources`].
///
/// [`File::require`]: artichoke_core::file::File::require
/// [`LoadSources`]: crate::core::LoadSources
pub type ExtensionHook = fn(&mut Artichoke) -> Result<(), Error>;

#[cfg(all(feature = "load-path-native-filesystem-loader", not(any(test, doctest))))]
pub type Adapter = Hybrid;
#[cfg(any(not(feature = "load-path-native-filesystem-loader"), test, doctest))]
pub type Adapter = Memory;

fn absolutize_relative_to<T, U>(path: T, cwd: U) -> PathBuf
where
    T: AsRef<Path>,
    U: AsRef<Path>,
{
    let mut iter = path.as_ref().components().peekable();
    let hint = iter.size_hint();
    let (mut components, cwd_is_relative) = if let Some(Component::RootDir) = iter.peek() {
        (Vec::with_capacity(hint.1.unwrap_or(hint.0)), false)
    } else {
        let mut components = cwd.as_ref().components().map(Component::as_os_str).collect::<Vec<_>>();
        components.reserve(hint.1.unwrap_or(hint.0));
        (components, cwd.as_ref().is_relative())
    };
    for component in iter {
        match component {
            Component::CurDir => {}
            Component::ParentDir if cwd_is_relative => {
                components.pop();
            }
            Component::ParentDir => {
                components.pop();
                if components.is_empty() {
                    components.push(Component::RootDir.as_os_str());
                }
            }
            c => {
                components.push(c.as_os_str());
            }
        }
    }
    components.into_iter().collect()
}

pub fn normalize_slashes(path: PathBuf) -> Result<Vec<u8>, ConvertBytesError> {
    let mut path = Vec::from_path_buf(path).map_err(|_| ConvertBytesError::new())?;
    if cfg!(windows) {
        for byte in &mut path {
            if *byte == b'\\' {
                *byte = b'/';
            }
        }
    }
    Ok(path)
}

#[cfg(test)]
#[allow(clippy::shadow_unrelated)]
mod tests {
    use std::path::Path;

    use super::absolutize_relative_to;

    #[test]
    fn absolutize_absolute_path() {
        let path = Path::new("/foo/bar");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), path);
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), path);
    }

    #[test]
    fn absolutize_absolute_path_dedot_current_dir() {
        let path = Path::new("/././foo/./bar/./././.");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo/bar"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo/bar"));
    }

    #[test]
    fn absolutize_absolute_path_dedot_parent_dir() {
        let path = Path::new("/foo/bar/..");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo"));

        let path = Path::new("/foo/../../../../bar/../../../");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/"));

        let path = Path::new("/foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/boom/baz"));
    }

    #[test]
    fn absolutize_relative_path() {
        let path = Path::new("foo/bar");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/home/artichoke/foo/bar"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("relative/path/foo/bar"));
    }

    #[test]
    fn absolutize_relative_path_dedot_current_dir() {
        let path = Path::new("././././foo/./bar/./././.");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/home/artichoke/foo/bar"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("relative/path/foo/bar"));
    }

    #[test]
    #[cfg(unix)]
    fn absolutize_relative_path_dedot_parent_dir_unix() {
        let path = Path::new("foo/bar/..");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/home/artichoke/foo"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new("foo/../../../../bar/../../../");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new("foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }

    #[test]
    #[cfg(windows)]
    fn absolutize_relative_path_dedot_parent_dir_windows_forward_slash() {
        let path = Path::new("foo/bar/..");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("C:/Users/artichoke/foo"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new("foo/../../../../bar/../../../");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new("foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }

    #[test]
    #[cfg(windows)]
    fn absolutize_relative_path_dedot_parent_dir_windows_backward_slash() {
        let path = Path::new(r"foo\bar\..");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("C:/Users/artichoke/foo"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new(r"foo\..\..\..\..\bar\..\..\..\");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new(r"foo\..\..\..\..\bar\..\..\..\boom\baz");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }
}
