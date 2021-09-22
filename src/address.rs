/// An M-Bus address field.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Address {
  Unconfigured,
  Configured(u8),
  AddressingPerformedInNetworkLayer,
  Broadcast,
  Reserved,
}

impl From<u8> for Address {
  fn from(address: u8) -> Self {
    match address {
      0 => Self::Unconfigured,
      1..=250 => Self::Configured(address),
      251 | 252 => Self::Reserved,
      253 => Self::AddressingPerformedInNetworkLayer,
      254 | 255 => Self::Broadcast,
    }
  }
}
