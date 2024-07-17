use crc::Crc;
use hex::ToHex;
use std::error::Error;
use std::fmt::Display;

/// Maximum length of a single octet.
const MAX_SINGLE_BIT_LENGTH: u16 = 127;

/// Sets or clears a specific bit in a byte.
///
/// # Arguments
///
/// * `value` - The original byte value.
/// * `bit` - The bit position to set or clear.
/// * `set` - A boolean indicating whether to set or clear the bit.
///
/// # Returns
///
/// The modified byte value.
pub fn bit_set_to(value: u8, bit: u8, set: bool) -> u8 {
    if set {
        value | (1 << bit)
    } else {
        value & !(1 << bit)
    }
}

/// Generates a checksum for [`Frame`] by the address, control, and length fields.
pub fn checksum(addr: u8, control: u8, length: u16) -> Result<u8, Box<dyn Error>> {
    let crc = Crc::<u8>::new(&crc::CRC_8_ROHC);
    let mut data: Vec<u8> = vec![addr, control];
    if length > MAX_SINGLE_BIT_LENGTH {
        let len = length.to_be_bytes();
        data.extend_from_slice(&len);
    } else {
        data.push(length as u8);
    };
    let crc_value = crc.checksum(&data);
    Ok(!crc_value)
}

/// Data Link Connection Identifier
///
/// The Data Link Connection Identifier (DLCI) is a 6-bit field that identifies the logical channel between the DTE and DCE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DLCI {
    AT = 0x1,
    SMS = 0x3,
    VOICE = 0x4,
    DATA = 0x5,
}

impl From<Address> for DLCI {
    fn from(addr: Address) -> Self {
        match addr.0 & 0x3F {
            0x1 => DLCI::AT,
            0x3 => DLCI::SMS,
            0x4 => DLCI::VOICE,
            0x5 => DLCI::DATA,
            _ => DLCI::AT,
        }
    }
}

impl Display for DLCI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DLCI::AT => write!(f, "AT"),
            DLCI::SMS => write!(f, "SMS"),
            DLCI::VOICE => write!(f, "VOICE"),
            DLCI::DATA => write!(f, "DATA"),
        }
    }
}

/// Address
///
/// <table>
///   <tr>
///     <th>Bit No.</th>
///     <td>1</td>
///     <td>2</td>
///     <td>3</td>
///     <td>4</td>
///     <td>5</td>
///     <td>6</td>
///     <td>7</td>
///     <td>8</td>
///   </tr>
///   <tr>
///     <th>Data</th>
///     <td>EA</td>
///     <td>C/R</td>
///     <td colspan=6 align="center">DLCI</td>
///   </tr>
/// </table>
///
/// * EA: Extended Address Bit. This bit is always set to 1.
/// * C/R: Command/Response Bit. See below.
/// * [`DLCI`]: Data Link Connection Identifier. This field is 6 bits long.
///
/// | Command/response | Direction              | C/R value |
/// |------------------|------------------------|-----------|
/// | Command          | Initiator -> Responder | 1         |
/// |                  | Responder -> Initiator | 0         |
/// | Response         | Initiator -> Responder | 0         |
/// |                  | Responder -> Initiator | 1         |
///
/// # Example
///
/// ```
/// use gsm0710::types::Address;
/// use gsm0710::types::DLCI;
///
/// let addr = Address::default();
///
/// assert_eq!(addr.0, 0b111);
/// assert_eq!(addr.with_cr(true).0, 0b111);
/// assert_eq!(addr.with_cr(false).0, 0b101);
/// assert_eq!(addr.with_dlci(DLCI::DATA).0, 0b10111);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address(pub u8);

impl Address {
    pub fn with_cr(&self, cr: bool) -> Address {
        Address(bit_set_to(self.0, 1, cr))
    }

    pub fn with_dlci(&self, dlci: DLCI) -> Address {
        Address((self.0 & 0x3) | ((dlci as u8 & 0x3F) << 2))
    }
}

impl From<DLCI> for Address {
    fn from(dlci: DLCI) -> Self {
        Address::default().with_dlci(dlci)
    }
}

impl Default for Address {
    /// Default with DLCI::AT and C/R 1
    fn default() -> Self {
        Address(0b111)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dlci = DLCI::from(*self);
        write!(f, "{}", dlci)
    }
}

/// Frame Builder for GSM 07.10 [`Frame`]
///
/// The FrameBuilder is a builder pattern for creating a Packet.
///
/// # Example
///
/// ```
/// use gsm0710::types::{Address, FrameBuilder};
/// let p = FrameBuilder::default()
///    .with_address(Address::default())
///    .with_content("AT+CMUX?".to_string())
///    .with_control(0xEF)
///    .build();
/// assert_eq!(p.header, 0xF9);
/// ```
///
/// # Note
///
/// FrameBuilder will automatically add `\r\n` to the end of content if it is not present.
pub struct FrameBuilder {
    address: Option<Address>,
    content: Option<String>,
    control: u8,
}

impl Default for FrameBuilder {
    fn default() -> Self {
        FrameBuilder {
            address: None,
            content: None,
            control: 0xEF,
        }
    }
}

/// The `FrameBuilder` struct is responsible for building frames.
impl FrameBuilder {
    /// Calculates the length of the frame.
    ///
    /// # Returns
    ///
    /// - `Ok(u16)`: The length of the frame if the content is present.
    /// - `Err(Box<dyn Error>)`: An error indicating that the content is required.
    fn length(&self) -> Result<u16, Box<dyn Error>> {
        match &self.content {
            Some(content) => {
                let len = content.len() as u16;
                if len > MAX_SINGLE_BIT_LENGTH {
                    Ok(len << 1)
                } else {
                    Ok((len << 1) + 1)
                }
            }
            None => Err("Content is required".into()),
        }
    }

    /// Calculates the checksum of the frame.
    ///
    /// # Returns
    ///
    /// - `Ok(u8)`: The checksum of the frame if the address is present.
    /// - `Err(Box<dyn Error>)`: An error indicating that the address is required.
    fn checksum(&self) -> Result<u8, Box<dyn Error>> {
        match &self.address {
            None => Err("Address is required".into()),
            Some(addr) => {
                let len = self.length()?;
                checksum(addr.0, self.control, len)
            }
        }
    }

    /// Sets the address of the frame.
    ///
    /// # Arguments
    ///
    /// - `address`: The address to set.
    ///
    /// # Returns
    ///
    /// - `&mut Self`: A mutable reference to the `FrameBuilder` object.
    pub fn with_address(&mut self, address: Address) -> &mut Self {
        self.address = Some(address);
        self
    }

    /// Sets the content of the frame.
    ///
    /// # Arguments
    ///
    /// - `content`: The content to set.
    ///
    /// # Returns
    ///
    /// - `&mut Self`: A mutable reference to the `FrameBuilder` object.
    pub fn with_content(&mut self, content: String) -> &mut Self {
        if content.ends_with("\r\n") {
            self.content = Some(content);
        } else {
            self.content = Some(format!("{}\r\n", content));
        }
        self
    }

    /// Sets the control of the frame.
    ///
    /// # Arguments
    ///
    /// - `control`: The control to set.
    ///
    /// # Returns
    ///
    /// - `&mut Self`: A mutable reference to the `FrameBuilder` object.
    pub fn with_control(&mut self, control: u8) -> &mut Self {
        self.control = control;
        self
    }

    /// Builds the frame.
    ///
    /// # Returns
    ///
    /// - [`Frame`]: The built frame.
    pub fn build(&self) -> Frame {
        Frame {
            header: 0xF9,
            address: self.address.unwrap(),
            control: self.control,
            length: self.length().unwrap(),
            content: self.content.clone().unwrap(),
            checksum: self.checksum().unwrap(),
            footer: 0xF9,
        }
    }
}

/// Represents a frame in the GSM0710 protocol.
///
/// The Frame struct is defined as follows:
///
/// | **Name** | Flag    | [`Address`] | Control | Length Indicator | Information                                      | FCS     | Flag    |
/// |----------|---------|-------------|---------|------------------|--------------------------------------------------|---------|---------|
/// | **Size** | 1 octet |   1 octet   | 1 octet | 1 or 2 octets    | Unspecified length but integral number of octets | 1 octet | 1 octet |
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub header: u8,
    pub address: Address,
    pub control: u8,
    pub length: u16,
    pub content: String,
    pub checksum: u8,
    pub footer: u8,
}

impl Frame {
    /// Converts the frame to a byte vector.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the byte representation of the frame.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![self.header, self.address.0, self.control];
        if self.length > MAX_SINGLE_BIT_LENGTH {
            data.push((self.length >> 8) as u8);
            data.push((self.length & 0xFF) as u8);
        } else {
            data.push(self.length as u8);
        }
        data.extend(self.content.as_bytes());
        data.push(self.checksum);
        data.push(self.footer);
        data
    }

    /// Converts the frame to a hexadecimal string.
    ///
    /// # Returns
    ///
    /// A `String` containing the hexadecimal representation of the frame.
    pub fn to_hex_string(&self) -> String {
        self.to_bytes().encode_hex::<String>()
    }

    /// Creates a frame from a byte vector.
    ///
    /// # Arguments
    ///
    /// * `data` - A `Vec<u8>` containing the byte representation of the frame.
    ///
    /// # Returns
    ///
    /// A `Frame` object created from the byte vector.
    pub fn from_bytes(data: Vec<u8>) -> Frame {
        let mut p = 0;
        let header = data[p];
        p += 1;
        let address = Address(data[p]);
        p += 1;
        let control = data[p];
        p += 1;
        let length = if data[p] & 0x1 == 0 {
            let l = ((data[p] as u16) << 8) | data[p + 1] as u16;
            p += 2;
            l
        } else {
            let l = data[p] as u16;
            p += 1;
            l
        };
        let content = String::from_utf8(data[p..data.len() - 2].to_vec()).unwrap();
        let checksum = data[data.len() - 2];
        let footer = data[data.len() - 1];
        Frame {
            header,
            address,
            control,
            length,
            content,
            checksum,
            footer,
        }
    }

    /// Verifies the integrity of the frame.
    ///
    /// * If the length field matches the content length, the length field is valid.
    /// * If the checksum matches the calculated checksum, the checksum is valid.
    ///
    /// # Returns
    ///
    /// `true` if the frame is valid, `false` otherwise.
    pub fn verify(&self) -> bool {
        let content_len = self.content.len() as u16;
        if content_len > MAX_SINGLE_BIT_LENGTH {
            if self.length != (content_len << 1) {
                return false;
            }
        } else if self.length != (content_len << 1) + 1 {
            return false;
        }

        match checksum(self.address.0, self.control, self.length) {
            Ok(c) => c == self.checksum,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_builder() {
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content("AT+CMUX?".to_string())
            .build();
        assert_eq!(p.header, 0xF9);
        assert_eq!(p.address, DLCI::AT.into());
        assert_eq!(p.control, 0xEF);
        assert_eq!(p.length, 0x15);
        assert_eq!(p.content, "AT+CMUX?\r\n");
        assert_eq!(p.checksum, 0x2C);
        assert_eq!(p.footer, 0xF9);
    }

    #[test]
    fn test_packet_builder_very_long() {
        let content = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .to_string()
            .repeat(10);
        let len = (content.len() + 2) * 2; // more than 128, so bit 1 is set zero
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content(content)
            .build();
        assert_eq!(p.length, len as u16);
    }

    #[test]
    fn test_packet_to_bytes() {
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content("AT+CMUX?".to_string())
            .build();
        let data = p.to_hex_string();
        assert_eq!(data, "f907ef1541542b434d55583f0d0a2cf9".to_string());
    }

    #[test]
    fn test_packet_from_bytes() {
        let content = "AT+CMUX?".to_string();
        let len = (content.len() + 2) * 2 + 1; // less than 128, so bit 1 is set 1
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content("AT+CMUX?".to_string())
            .build();
        let d = Frame::from_bytes(p.to_bytes());
        assert_eq!(p, d);
        assert_eq!(d.length, len as u16);
        assert_eq!(d.address, Address::default());
    }

    #[test]
    fn test_packet_from_bytes_very_long() {
        let content = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .to_string()
            .repeat(10);
        let len = (content.len() + 2) * 2; // more than 128, so bit 1 is set zero
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content(content)
            .build();
        let d = Frame::from_bytes(p.to_bytes());
        assert_eq!(p, d);
        assert_eq!(d.length, len as u16);
        assert!(d.verify());
    }
}
