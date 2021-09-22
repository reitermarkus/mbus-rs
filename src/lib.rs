#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_debug_implementations)]

use core::fmt;
use core::num::NonZeroUsize;

/// An M-Bus error.
#[derive(Debug)]
pub enum Error {
  /// Telegram does not start with the expected character.
  InvalidStartCharacter,
  /// Telegram format is wrong.
  InvalidFormat,
  /// Telegram is incomplete.
  Incomplete(Option<NonZeroUsize>),
  /// Checksum does not match.
  ChecksumMismatch,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidStartCharacter => write!(f, "invalid start character"),
      Self::InvalidFormat => write!(f, "invalid format"),
      Self::Incomplete(_) => write!(f, "incomplete"),
      Self::ChecksumMismatch => write!(f, "checksum mismatch"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl<I> nom::error::ParseError<I> for Error {
  fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
    Error::InvalidFormat
  }

  fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
    other
  }
}

mod address;
pub use address::Address;

mod control;
pub use control::Control;

mod telegram;
pub use telegram::Telegram;
