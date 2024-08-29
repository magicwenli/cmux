use bitfield_struct::bitfield;
use crc::Crc;
use hex::ToHex;
use std::error::Error;
use std::fmt::Debug;

/// Maximum length of a single octet.
const MAX_SINGLE_BIT_LENGTH: u16 = 127;

#[derive(PartialEq, Eq, Clone)]
pub struct ContentStr(String);

impl Debug for ContentStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentStr")
            .field("str", &self.0)
            .field("raw", &format_args!("{:02X?}", self.0.as_bytes()))
            .finish()
    }
}

impl PartialEq<&str> for ContentStr {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Sets or clears a specific bit in a byte.
///
/// # Arguments
///
/// * `value` - The original byte value.
/// * `bit` - The bit position to set or clear. Starts from 0.
/// * `set` - A boolean indicating whether to set or clear the bit.
///
/// # Returns
///
/// The modified byte value.
pub const fn bit_set_to(value: u8, bit: u8, set: bool) -> u8 {
    if set {
        value | (1 << bit)
    } else {
        value & !(1 << bit)
    }
}

/// Generates a checksum for [`Frame`] by the address, control, and length fields.
pub fn checksum_uih(addr: u8, control: u8, length: u16) -> Result<u8, Box<dyn Error>> {
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

/// Generates a checksum for [`Frame`] by the address, control, length, and content fields.
pub fn checksum_ui(addr: u8, control: u8, length: u8, content: &str) -> Result<u8, Box<dyn Error>> {
    let crc = Crc::<u8>::new(&crc::CRC_8_ROHC);
    let mut data: Vec<u8> = vec![addr, control, length];
    data.extend_from_slice(content.as_bytes());
    let crc_value = crc.checksum(&data);
    Ok(!crc_value)
}

/// Data Link Connection Identifier
///
/// The Data Link Connection Identifier (DLCI) is a 6-bit field that identifies the logical channel between the DTE and DCE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DLCI {
    AT(u8),
    SMS(u8),
    VOICE(u8),
    DATA(u8),
    OTHER(u8),
}

impl DLCI {
    const fn into_bits(self) -> u8 {
        match self {
            DLCI::AT(_) => 0x1,
            DLCI::SMS(_) => 0x3,
            DLCI::VOICE(_) => 0x4,
            DLCI::DATA(_) => 0x5,
            DLCI::OTHER(value) => value,
        }
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0x1 => DLCI::AT(0x1),
            0x3 => DLCI::SMS(0x3),
            0x4 => DLCI::VOICE(0x4),
            0x5 => DLCI::DATA(0x5),
            _ => DLCI::OTHER(value),
        }
    }
}

/// Address Field of [`Frame`]
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
/// use cmux::types::Address;
/// use cmux::types::DLCI;
///
/// let addr = Address::default();
/// assert_eq!(addr.into_bits(), 0b111);
///
/// let addr = addr.with_cr(false);
/// assert_eq!(addr.into_bits(), 0b101);
///
/// let addr = addr.with_dlci(DLCI::DATA);
/// assert_eq!(addr.into_bits(), 0b10101);
/// ```

#[bitfield(u8, default = false)]
#[derive(PartialEq, Eq)]
pub struct Address {
    pub ea: bool,
    pub cr: bool,
    #[bits(6)]
    pub dlci: DLCI,
}

impl Default for Address {
    fn default() -> Self {
        Address(0b111)
    }
}

/// Frame Type of [`Frame`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    SABM,
    UA,
    DM,
    DISC,
    UIH,
    UI,
}

impl FrameType {
    const fn into_bits(self) -> u8 {
        match self {
            FrameType::SABM => 0b00101111,
            FrameType::UA => 0b01100011,
            FrameType::DM => 0b00001111,
            FrameType::DISC => 0b01000011,
            FrameType::UIH => 0b11101111,
            FrameType::UI => 0b00000011,
        }
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b00101111 => FrameType::SABM,
            0b01100011 => FrameType::UA,
            0b00001111 => FrameType::DM,
            0b01000011 => FrameType::DISC,
            0b11101111 => FrameType::UIH,
            0b00000011 => FrameType::UI,
            _ => FrameType::UI,
        }
    }
}

/// Control Field of [`Frame`]
///
/// The Control field is a 8-bit field, structured as follows:
///
/// | **Frame Type**                                 | **1** | **2** | **3** | **4** | **5** | **6** | **7** | **8** | **Notes** |
/// |------------------------------------------------|-------|-------|-------|-------|-------|-------|-------|-------|-----------|
/// | SABM (Set Asynchronous Balanced Mode)          | 1     | 1     | 1     | 1     | P/F   | 1     | 0     | 0     |           |
/// | UA (Unnumbered Acknowledgement)                | 1     | 1     | 0     | 0     | P/F   | 1     | 1     | 0     |           |
/// | DM (Disconnected Mode)                         | 1     | 1     | 1     | 1     | P/F   | 0     | 0     | 0     |           |
/// | DISC (Disconnect)                              | 1     | 1     | 0     | 0     | P/F   | 0     | 1     | 0     |           |
/// | UIH (Unnumbered Information with Header check) | 1     | 1     | 1     | 1     | P/F   | 1     | 1     | 1     |           |
/// | UI (Unnumbered Information)                    | 1     | 1     | 0     | 0     | P/F   | 0     | 0     | 0     | Optional  |
///
/// * P/F stands for Poll/Final bit.
/// * SABM (Set Asynchronous Balance Mode): SABM command shall be send by the TE (the host) to the UE (the target) to confirm the acceptance of SABM by transmission of UA response.
/// * UA (Unnumbered Acknowledgement): The UA response is sent by the module as an acknowledgement that a SABM or DISC command was accepted.
/// * DM (Disconnected Mode): In case if the module rejects SABM or DISC command, it will send DM response. For example, if SABM is sent for a DLCI not supported or if a DISC is sent to DLCI address already closed, this frame will be send.
/// * DISC (Disconnect): The DISC is used to close a previously established connection. If the application sends a DISC for the DLCI 1 and DLCI 1 is already established, then it will be closed. The module will answer to this command with an UA frame.
/// * UIH (Unnumbered Information with Header check): The UIH command/response will be used to send information. For the UIH frame, the FCS will be calculated over **only the address, control and length fields**. There is no specified response to the UIH command/response.
/// * UI (Unnumbered Information): The UI command/response will be used to send information. There is no specified response to the UI command/response. For the UI frame, the FCS shall be calculated over **all fields (Address, Control, Length Indicator, and Information)**. Support of UI frames is optional.
///
/// # Example
///
/// ```
/// use cmux::types::Control;
/// use cmux::types::FrameType;
///
/// let control = Control::default();
/// assert_eq!(control.into_bits(), 0b11101111);
///
/// let control = control.with_pf(true);
/// assert_eq!(control.pf(), true);
///
/// let control = control.with_frame_type(FrameType::UA);
/// assert_eq!(control.frame_type(), FrameType::UA);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Control(u8);

impl Control {
    pub const fn new() -> Self {
        Control(0)
    }

    pub const fn with_frame_type(self, frame_type: FrameType) -> Self {
        let pf = self.pf();
        let control = Control(frame_type.into_bits());
        control.with_pf(pf)
    }

    pub const fn frame_type(&self) -> FrameType {
        FrameType::from_bits(self.0 & 0b11101111)
    }

    pub fn set_frame_type(&mut self, frame_type: FrameType) {
        self.0 = self.with_frame_type(frame_type).0;
    }

    pub const fn with_pf(self, pf: bool) -> Self {
        let value = bit_set_to(self.0, 4, pf);
        Control(value)
    }

    pub const fn pf(&self) -> bool {
        (self.0 & (1 << 4)) == (1 << 4)
    }

    pub fn set_pf(&mut self, pf: bool) {
        self.0 = self.with_pf(pf).0;
    }

    pub fn into_bits(self) -> u8 {
        self.into()
    }

    pub fn from_bits(value: u8) -> Self {
        value.into()
    }
}

impl From<u8> for Control {
    fn from(value: u8) -> Self {
        Control(value)
    }
}

impl From<Control> for u8 {
    fn from(value: Control) -> Self {
        value.0
    }
}

impl Default for Control {
    fn default() -> Self {
        Control::new()
            .with_pf(false)
            .with_frame_type(FrameType::UIH)
    }
}

impl Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Control")
            .field("frame_type", &self.frame_type())
            .field("pf", &self.pf())
            .finish()
    }
}

/// Frame Builder for GSM 07.10 [`Frame`]
///
/// The FrameBuilder is a builder pattern for creating a Packet.
///
/// # Example
///
/// ```
/// use cmux::types::{Address, Control, FrameBuilder};
/// let p = FrameBuilder::default()
///    .with_address(Address::default())
///    .with_content("AT+CMUX?".to_string())
///    .with_control(Control::default())
///    .build();
/// assert_eq!(p.header, 0xF9);
/// ```
///
/// # Note
///
/// FrameBuilder will automatically add `\r\n` to the end of content if it is not present.
#[derive(Debug)]
pub struct FrameBuilder {
    address: Option<Address>,
    control: Option<Control>,
    content: Option<String>,
}

impl Default for FrameBuilder {
    fn default() -> Self {
        FrameBuilder {
            address: Some(Address::default()),
            control: Some(Control::default()),
            content: None,
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
        let addr = self.address.expect("Address is required").into_bits();
        let control = self.control.expect("Control is required").into_bits();
        let length = self.length().expect("Length is required");

        if self.control.unwrap().frame_type() == FrameType::UI {
            checksum_ui(addr, control, length as u8, self.content.as_ref().unwrap())
        } else {
            checksum_uih(addr, control, length)
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
    pub fn with_control(&mut self, control: Control) -> &mut Self {
        self.control = Some(control);
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
            address: self.address.expect("Address is required"),
            control: self.control.expect("Control is required"),
            length: self.length().expect("Length is required"),
            content: ContentStr(self.content.clone().expect("Content is required")),
            checksum: self.checksum().expect("Checksum is required"),
            footer: 0xF9,
        }
    }
}

/// Represents a frame in the cmux protocol.
///
/// The Frame struct is defined as follows:
///
/// | **Name** | Flag    | [`Address`] | [`Control`] | Length Indicator | Information                                      | FCS     | Flag    |
/// |----------|---------|-------------|---------|------------------|--------------------------------------------------|---------|---------|
/// | **Size** | 1 octet |   1 octet   | 1 octet | 1 or 2 octets    | Unspecified length but integral number of octets | 1 octet | 1 octet |
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub header: u8,
    pub address: Address,
    pub control: Control,
    pub length: u16,
    pub content: ContentStr,
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
        let mut data = vec![
            self.header,
            self.address.into_bits(),
            self.control.into_bits(),
        ];
        if self.length > MAX_SINGLE_BIT_LENGTH {
            data.push((self.length >> 8) as u8);
            data.push((self.length & 0xFF) as u8);
        } else {
            data.push(self.length as u8);
        }
        data.extend(self.content.0.as_bytes());
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
        let address = Address::from_bits(data[p]);
        p += 1;
        let control = Control::from_bits(data[p]);
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
        let content = ContentStr(String::from_utf8_lossy(&data[p..data.len() - 2]).to_string());
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
    /// - `Ok(())`: If the frame is valid.
    /// - `Err(Box<dyn Error>)`: If the frame is invalid.
    pub fn verify(&self) -> Result<(), Box<dyn Error>> {
        let content_len = self.content.0.len() as u16;
        if content_len > MAX_SINGLE_BIT_LENGTH {
            if self.length != (content_len << 1) {
                return Err("Length field is invalid".into());
            }
        } else if self.length != (content_len << 1) + 1 {
            return Err("Length field is invalid".into());
        }

        if let Ok(c) = checksum_uih(
            self.address.into_bits(),
            self.control.into_bits(),
            self.length,
        ) {
            if c != self.checksum {
                Err("Checksum is invalid".into())
            } else {
                Ok(())
            }
        } else {
            Err("Checksum calculation failed".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_builder() {
        let p = FrameBuilder::default()
            .with_content("AT+CMUX?".to_string())
            .build();
        assert_eq!(p.header, 0xF9);
        assert_eq!(p.address, Address::default());
        assert_eq!(p.control, Control::default());
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
        assert!(d.verify().is_ok());
    }

    #[test]
    fn test_packet_checksum() {
        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content("AT+CMUX?".to_string())
            .build();
        let ori = p.checksum;
        let exp = checksum_uih(p.address.into_bits(), p.control.into_bits(), p.length).unwrap();
        assert_eq!(ori, exp);

        let p = FrameBuilder::default()
            .with_address(Address::default())
            .with_content("AT+CMUX?".to_string())
            .with_control(Control::default().with_frame_type(FrameType::UI))
            .build();
        let ori = p.checksum;
        let exp = checksum_ui(
            p.address.into_bits(),
            p.control.into_bits(),
            p.length as u8,
            &p.content.0,
        )
        .unwrap();
        assert_eq!(ori, exp);
    }
}
