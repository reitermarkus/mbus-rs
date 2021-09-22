use core::convert::TryFrom;

/// An M-Bus control field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Control {
  /// Initialization of slave.
  SndNke,
  /// Send user data to slave.
  SndUd { fcb: bool },
  /// Request for class 1 data.
  ReqUd1 { fcb: bool },
  /// Request for class 2 data.
  ReqUd2 { fcb: bool },
  /// Data transfer from slave to master after request.
  RspUd { acd: bool, dfc: bool },
}

impl TryFrom<u8> for Control {
  type Error = u8;

  fn try_from(control: u8) -> Result<Self, Self::Error> {
    Ok(match control {
      0b01000000 => Self::SndNke,
      0b01010011 => Self::SndUd { fcb: false },
      0b01110011 => Self::SndUd { fcb: true },
      0b01011011 => Self::ReqUd1 { fcb: false },
      0b01111011 => Self::ReqUd1 { fcb: true },
      0b01011010 => Self::ReqUd2 { fcb: false },
      0b01111010 => Self::ReqUd2 { fcb: true },
      0b00001000 => Self::RspUd { acd: false, dfc: false },
      0b00011000 => Self::RspUd { acd: false, dfc: true },
      0b00101000 => Self::RspUd { acd: true, dfc: false },
      0b00111000 => Self::RspUd { acd: true, dfc: true },
      unknown => return Err(unknown),
    })
  }
}
