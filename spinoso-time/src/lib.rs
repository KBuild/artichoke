#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::needless_borrow)]
#![allow(clippy::let_underscore_drop)]
// https://github.com/rust-lang/rust-clippy/pull/5998#issuecomment-731855891
#![allow(clippy::map_err_ignore)]
#![allow(clippy::option_if_let_else)]
#![allow(unknown_lints)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Time is an abstraction of dates and times.
//!
//! This module implements the [`Time`] class from Ruby Core.
//!
//! In Artichoke, Time is represented as a 64-bit signed integer of seconds
//! since January 1, 1970 UTC (the Unix Epoch) and an unsigned 32-bit integer of
//! subsecond nanoseconds. This allows representing roughly 584 billion years.
//!
//! You can use this class in your application by accessing it directly. As a
//! Core class, it is globally available:
//!
//! ```ruby
//! Time.now
//! ```
//!
//! This implementation of `Time` supports the system clock via the
//! [`chrono`] crate.
//!
//! # Crate features
//!
//! This crate requires [`std`], the Rust Standard Library.
//!
//! [`Time`]: https://ruby-doc.org/core-2.6.3/Time.html
//! [`chrono`]: https://crates.io/crates/chrono

// Ensure code blocks in README.md compile
#[cfg(doctest)]
macro_rules! readme {
    ($x:expr) => {
        #[doc = $x]
        mod readme {}
    };
    () => {
        readme!(include_str!("../README.md"));
    };
}
#[cfg(doctest)]
readme!();

use core::fmt;
use core::time::Duration;
use std::error::Error;

mod time;

pub use time::chrono::{Offset, Time, ToA};

/// Number of nanoseconds in one second.
#[allow(clippy::cast_possible_truncation)] // 1e9 < u32::MAX
pub const NANOS_IN_SECOND: u32 = Duration::from_secs(1).as_nanos() as u32;

/// Number of microseconds in one nanosecond.
#[allow(clippy::cast_possible_truncation)] // 1000 < u32::MAX
pub const MICROS_IN_NANO: u32 = Duration::from_micros(1).as_nanos() as u32;

/// Error returned when constructing a [`Time`] from a [`ToA`].
///
/// This error is returned when a time component in the `ToA` exeeds the maximum
/// permissible value for a datetime. For example, invalid values include a
/// datetime 5000 days or 301 seconds.
///
/// # Examples
///
/// Invalid date component:
///
/// ```
/// # use core::convert::TryFrom;
/// # use spinoso_time::{Offset, Time, ToA, ComponentOutOfRangeError};
/// let to_a = ToA {
///     sec: 21,
///     min: 3,
///     hour: 23,
///     day: 5000,
///     month: 4,
///     year: 2020,
///     wday: 0,
///     yday: 96,
///     isdst: true,
///     zone: Offset::Local,
/// };
/// let time = Time::try_from(to_a);
/// assert_eq!(time, Err(ComponentOutOfRangeError::Date));
/// ```
///
/// Invalid time component:
///
/// ```
/// # use core::convert::TryFrom;
/// # use spinoso_time::{Offset, Time, ToA, ComponentOutOfRangeError};
/// let to_a = ToA {
///     sec: 301,
///     min: 3,
///     hour: 23,
///     day: 5,
///     month: 4,
///     year: 2020,
///     wday: 0,
///     yday: 96,
///     isdst: true,
///     zone: Offset::Local,
/// };
/// let time = Time::try_from(to_a);
/// assert_eq!(time, Err(ComponentOutOfRangeError::Time));
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentOutOfRangeError {
    /// Date component (year, month, day) out of range.
    Date,
    /// Time component (hour, minute, second) out of range.
    Time,
}

impl fmt::Display for ComponentOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date => f.write_str("Date component (year, month, day) out of range"),
            Self::Time => f.write_str("Time component (hour, minute, second) out of range"),
        }
    }
}

impl Error for ComponentOutOfRangeError {}
