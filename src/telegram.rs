use core::convert::TryFrom;

use nom::{
  IResult,
  branch::alt,
  number::streaming::u8,
  bytes::streaming::{tag, take},
  combinator::cut,
  sequence::tuple,
};

use super::{Error, Control, Address};

/// An M-Bus telegram.
#[derive(Debug, Clone, PartialEq)]
pub enum Telegram<'ud> {
  SingleCharacter,
  ShortFrame {
    control: Control,
    address: Address,
  },
  ControlFrame {
    control: Control,
    address: Address,
    control_information: u8,
  },
  LongFrame {
    control: Control,
    address: Address,
    control_information: u8,
    user_data: &'ud [u8],
  },
}

impl<'ud> Telegram<'ud> {
  const SINGLE_CHAR: u8 = 0xe5;
  const SHORT_START_CHAR: u8 = 0x10;
  const LONG_START_CHAR: u8 = 0x68;
  const STOP_CHAR: u8 = 0x16;

  fn map_error(nom_error: nom::Err<nom::error::Error<&[u8]>>, error: Error) -> nom::Err<Error> {
    nom_error.map(|_| error)
  }

  fn parse_single(input: &'ud [u8]) -> IResult<&'ud [u8], Self, Error> {
    let (input, _) = tag([Telegram::SINGLE_CHAR])(input)
      .map_err(|err| Self::map_error(err, Error::InvalidStartCharacter))?;
    Ok((input, Self::SingleCharacter))
  }

  fn parse_short(input: &'ud [u8]) -> IResult<&'ud [u8], Self, Error> {
    let (input, _) = tag([Self::SHORT_START_CHAR])(input)
      .map_err(|err| Self::map_error(err, Error::InvalidStartCharacter))?;
    Self::parse_payload(input, 2)
  }

  fn parse_long(input: &'ud [u8]) -> IResult<&'ud [u8], Self, Error> {
    let start_char = tag([Self::LONG_START_CHAR]);
    let (input, (_, payload_len, payload_len_check, _)) =
      tuple((&start_char, u8, u8, &start_char))(input)
        .map_err(|err| Self::map_error(err, Error::InvalidStartCharacter))?;

    if payload_len != payload_len_check {
      return Err(nom::Err::Error(Error::InvalidStartCharacter))
    }

    Self::parse_payload(input, payload_len.into())
  }

  fn parse_payload(input: &'ud [u8], mut payload_len: usize) -> IResult<&'ud [u8], Self, Error> {
    let mut calculated_checksum = 0u8;

    let mut checksummed_u8 = |input| {
      let (input, value) = cut(u8)(input)?;
      calculated_checksum = calculated_checksum.wrapping_add(value);
      Ok((input, value))
    };

    let (input, control) = checksummed_u8(input)?;
    let control = Control::try_from(control)
      .map_err(|_| nom::Err::Failure(Error::InvalidFormat))?;
    payload_len -= 1;

    let (input, address) = checksummed_u8(input)?;
    let address = Address::from(address);
    payload_len -= 1;

    let (input, control_information) = if payload_len > 0 {
      let (input, control_information) = checksummed_u8(input)?;
      payload_len -= 1;
      (input, Some(control_information))
    } else {
      (input, None)
    };

    let (input, payload) = take(payload_len)(input)?;
    for &value in payload {
      calculated_checksum = calculated_checksum.wrapping_add(value);
    }

    let (input, checksum) = cut(u8)(input)?;

    let (input, _stop_char) = cut(tag([Self::STOP_CHAR]))(input)?;

    if calculated_checksum != checksum {
      return Err(nom::Err::Failure(Error::ChecksumMismatch))
    }

    if let Some(control_information) = control_information {
      if payload.len() > 0 {
        Ok((input, Self::LongFrame { control, address, control_information, user_data: payload }))
      } else {
        Ok((input, Self::ControlFrame { control, address, control_information }))
      }
    } else {
      Ok((input, Self::ShortFrame { control, address }))
    }
  }

  pub fn parse(input: &'ud [u8]) -> Result<(&'ud [u8], Telegram<'ud>), Error> {
    use nom::Finish;

    alt((Self::parse_single, Self::parse_short, Self::parse_long))(input)
      .map_err(|err| match err {
        nom::Err::Incomplete(needed) => nom::Err::Failure(Error::Incomplete(match needed {
          nom::Needed::Unknown => None,
          nom::Needed::Size(size) => Some(size),
        })),
        err => err,
      })
      .finish()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  const TELEGRAMS: [u8; 376] = [
    0x68, // Start Character
    0xfa, // Payload Length
    0xfa, // Payload Length
    0x68, // Start Character
    0x53, // Control Field
    0xff, // Address Field
    0x00, // Control Information Field
    0x01, 0x67, 0xdb, 0x08, 0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, 0x82, 0x01, 0x55, 0x21,
    0x00, 0x02, 0xbc, 0x66, 0x2f, 0xba, 0x85, 0x66, 0x9e, 0x76, 0xef, 0x03, 0x47, 0x06, 0xcc, 0xfc,
    0x93, 0x70, 0xf9, 0xb7, 0xab, 0x49, 0xd0, 0x35, 0xdd, 0x0e, 0xe7, 0x5a, 0x95, 0x36, 0xfc, 0x7a,
    0x19, 0x48, 0xf7, 0x9c, 0x69, 0x8f, 0xac, 0xc2, 0xfc, 0x5f, 0x7b, 0xe5, 0xf1, 0x47, 0xf3, 0xee,
    0x87, 0x40, 0xbe, 0xe9, 0x11, 0x8c, 0x8f, 0x7b, 0xc5, 0xb2, 0xc5, 0x12, 0x53, 0x29, 0xce, 0xb4,
    0xbe, 0xad, 0xe1, 0x16, 0xd1, 0x61, 0x2c, 0x7f, 0x82, 0xa0, 0x4f, 0xaa, 0x70, 0xc3, 0x5d, 0x06,
    0x67, 0xd9, 0xee, 0xec, 0xf6, 0x86, 0xaf, 0xb6, 0x43, 0x19, 0xdc, 0x13, 0xec, 0x3c, 0x9a, 0x39,
    0x10, 0x5b, 0x46, 0x37, 0x2a, 0xca, 0x24, 0xf2, 0xa1, 0x73, 0x39, 0x32, 0x5f, 0x27, 0x0a, 0xb9,
    0x8c, 0x40, 0xe7, 0xaa, 0x8a, 0x61, 0xec, 0xf5, 0x88, 0x4c, 0x4e, 0x3b, 0x7e, 0xb4, 0xef, 0x5b,
    0xb2, 0x92, 0x68, 0x4a, 0x6b, 0xba, 0x91, 0xb3, 0x49, 0xbb, 0x70, 0x41, 0x62, 0xa2, 0x10, 0xc3,
    0x6d, 0xc1, 0xf2, 0x52, 0x88, 0x42, 0x71, 0x7b, 0xfe, 0x64, 0xf8, 0x05, 0xce, 0xab, 0x98, 0x0e,
    0x14, 0xc1, 0xe2, 0x9e, 0x10, 0x19, 0x5e, 0x1b, 0xa2, 0xef, 0x24, 0xe8, 0xf9, 0xcb, 0x0f, 0xe6,
    0x09, 0xd3, 0x2b, 0xb8, 0xc3, 0x6e, 0x23, 0xb8, 0x47, 0x7b, 0x14, 0xda, 0xc2, 0x37, 0x63, 0xa2,
    0x5b, 0xee, 0x27, 0xa8, 0x1f, 0x20, 0xa7, 0x6c, 0x2f, 0x8e, 0x28, 0xc9, 0x2b, 0x3e, 0xbe, 0x04,
    0x48, 0x6d, 0xc2, 0xdc, 0x07, 0x41, 0x63, 0xbe, 0x49, 0xdf, 0x25, 0x96, 0x30, 0x9c, 0x86, 0x39,
    0x53, 0x31, 0x65, 0x35, 0xd1, 0xf0, 0xdf,
    0x8a, // Checksum
    0x16, // Stop Character

    0x68, // Start Character
    0x72, // Payload Length
    0x72, // Payload Length
    0x68, // Start Character
    0x53, // Control Field
    0xff, // Address Field
    0x11, // Control Information Field
    0x01, 0x67, 0x5b, 0x5f, 0x0f, 0xb3, 0x7e, 0xde, 0xda, 0xb5, 0xaf, 0xed, 0x57, 0xbd, 0xa7, 0x5a,
    0x2e, 0x17, 0xcf, 0x11, 0x79, 0xc8, 0x1d, 0xbe, 0xb4, 0xac, 0xc8, 0x80, 0x2c, 0xb1, 0xdb, 0xf8,
    0x74, 0xe6, 0x76, 0xa3, 0x42, 0xf6, 0xe5, 0xde, 0x97, 0x29, 0x86, 0x1f, 0x07, 0x67, 0xac, 0xf9,
    0x04, 0xf8, 0x0a, 0x44, 0xa0, 0xdd, 0x16, 0x46, 0xf2, 0x08, 0x83, 0x44, 0x5e, 0x11, 0x91, 0xe3,
    0x52, 0x49, 0x58, 0x0e, 0xaa, 0x4b, 0xec, 0x58, 0xaa, 0xee, 0x1a, 0xdf, 0xda, 0x60, 0x14, 0x5f,
    0x51, 0xb8, 0xbe, 0xd4, 0x36, 0x10, 0xdf, 0xee, 0x5b, 0x2c, 0xe3, 0x38, 0x0d, 0xe7, 0xf3, 0x4d,
    0x9f, 0xca, 0x2a, 0x15, 0x6f, 0x68, 0x79, 0xf4, 0x1e, 0xec, 0x8d, 0x20, 0xef, 0xa2, 0xdf,
    0x38, // Checksum
    0x16, // Stop Character
  ];

  #[test]
  fn test_parse() {
    let mut telegrams = vec![];
    let mut input = &TELEGRAMS[..];

    for _ in 0..2 {
      let (next_input, telegram) = Telegram::parse(input).unwrap();
      input = next_input;
      telegrams.push(telegram);
    }

    assert_eq!(telegrams, vec![
      Telegram::LongFrame {
        control: Control::SndUd { fcb: false },
        address: Address::Broadcast,
        control_information: 0x00,
        user_data: &TELEGRAMS[7..254],
      },
      Telegram::LongFrame {
        control: Control::SndUd { fcb: false },
        address: Address::Broadcast,
        control_information: 0x11,
        user_data: &TELEGRAMS[263..374],
      },
    ])
  }
}
