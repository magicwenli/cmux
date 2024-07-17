/// This module contains types and functions related to GSM 07.10 protocol.
use crc::Crc;
use hex::ToHex;
use std::error::Error;
use std::fmt::Debug;

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
/// use gsm0710::types::Address;
/// use gsm0710::types::DLCI;
///
/// let addr = Address::default();
/// assert_eq!(addr.0, 0b111);
///
/// let addr = addr.with_cr(false);
/// assert_eq!(addr.0, 0b101);
///
/// let addr = addr.with_dlci(DLCI::DATA);
/// assert_eq!(addr.0, 0b10101);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address(pub u8);

impl Address {
    /// Creates a new `Address` with the specified EA (Extension Address) bit.
    ///
    /// # Arguments
    ///
    /// * `ea` - The value of the EA bit.
    ///
    /// # Returns
    ///
    /// A new `Address` with the specified EA bit.
    fn with_ea(&self, ea: bool) -> Address {
        Address(bit_set_to(self.0, 0, ea))
    }

    /// Creates a new `Address` with the specified CR (Command/Response) bit.
    ///
    /// # Arguments
    ///
    /// * `cr` - The value of the CR bit.
    ///
    /// # Returns
    ///
    /// A new `Address` with the specified CR bit.
    pub fn with_cr(&self, cr: bool) -> Address {
        Address(bit_set_to(self.0, 1, cr))
    }

    /// Creates a new `Address` with the specified [`DLCI`] (Data Link Connection Identifier).
    ///
    /// # Arguments
    ///
    /// * `dlci` - The [`DLCI`] value.
    ///
    /// # Returns
    ///
    /// A new `Address` with the specified [`DLCI`].
    pub fn with_dlci(&self, dlci: DLCI) -> Address {
        let ea = self.0 & (1 << 0);
        let cr = self.0 & (1 << 1);
        let address = Address::from(dlci);
        Address(address.0 | ea | cr)
    }
}

impl From<DLCI> for Address {
    fn from(dlci: DLCI) -> Self {
        Address((dlci as u8) << 2)
    }
}

impl Default for Address {
    /// Default with DLCI::AT and C/R 1
    fn default() -> Self {
        Address::from(DLCI::AT).with_cr(true).with_ea(true)
    }
}

/// Frame Type of [`Frame`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    SABM,
    UA,
    DM,
    DISC,
    UIH,
    UI,
}

/// Converts a [`Control`] enum variant into a [`FrameType`] enum variant.
impl From<Control> for FrameType {
    /// Converts the given `Control` variant into a corresponding `FrameType` variant.
    ///
    /// # Arguments
    ///
    /// * `control` - The [`Control`] variant to convert.
    ///
    /// # Returns
    ///
    /// The converted [`FrameType`] variant.
    fn from(control: Control) -> Self {
        match bit_set_to(control.0, 4, false) {
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
/// ```
/// use gsm0710::types::Control;
/// use gsm0710::types::FrameType;
///
/// let control = Control::default();
/// assert_eq!(control, Control(0b11101111));
///
/// let control = control.with_pf(true);
/// assert_eq!(control, Control(0b11111111));
///
/// let control = control.with_type(FrameType::UA);
/// assert_eq!(control, Control(0b01110011));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Control(pub u8);

impl From<FrameType> for Control {
    fn from(frame_type: FrameType) -> Self {
        match frame_type {
            FrameType::SABM => Control(0b00101111),
            FrameType::UA => Control(0b01100011),
            FrameType::DM => Control(0b00001111),
            FrameType::DISC => Control(0b01000011),
            FrameType::UIH => Control(0b11101111),
            FrameType::UI => Control(0b00000011),
        }
    }
}

impl Default for Control {
    /// Default with P/F 1 and UIH type
    fn default() -> Self {
        Control::from(FrameType::UIH).with_pf(false)
    }
}

/// Implementation of the `Control` struct.
impl Control {
    /// Creates a new `Control` instance with the specified `pf` value.
    ///
    /// # Arguments
    ///
    /// * `pf` - A boolean value indicating the value of the PF bit.
    ///
    /// # Returns
    ///
    /// A new `Control` instance with the specified `pf` value.
    pub fn with_pf(&self, pf: bool) -> Control {
        Control(bit_set_to(self.0, 4, pf))
    }

    /// Creates a new `Control` instance with the specified `frame_type` value.
    ///
    /// # Arguments
    ///
    /// * `frame_type` - A [`FrameType`] value indicating the type of the frame.
    ///
    /// # Returns
    ///
    /// A new `Control` instance with the specified `frame_type` value.
    pub fn with_type(&self, frame_type: FrameType) -> Control {
        let pf = self.0 & (1 << 4);
        let control = Control::from(frame_type);
        Control(bit_set_to(control.0, 4, pf != 0))
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
#[derive(Debug, Default)]
pub struct FrameBuilder {
    address: Option<Address>,
    content: Option<String>,
    control: Control,
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
                checksum(addr.0, self.control.0, len)
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
        self.control = Control(control);
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
/// | **Name** | Flag    | [`Address`] | [`Control`] | Length Indicator | Information                                      | FCS     | Flag    |
/// |----------|---------|-------------|---------|------------------|--------------------------------------------------|---------|---------|
/// | **Size** | 1 octet |   1 octet   | 1 octet | 1 or 2 octets    | Unspecified length but integral number of octets | 1 octet | 1 octet |
#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub header: u8,
    pub address: Address,
    pub control: Control,
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
        let mut data = vec![self.header, self.address.0, self.control.0];
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
        let control = Control(data[p]);
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
    /// - `Ok(())`: If the frame is valid.
    /// - `Err(Box<dyn Error>)`: If the frame is invalid.
    pub fn verify(&self) -> Result<(), Box<dyn Error>> {
        let content_len = self.content.len() as u16;
        if content_len > MAX_SINGLE_BIT_LENGTH {
            if self.length != (content_len << 1) {
                return Err("Length field is invalid".into());
            }
        } else if self.length != (content_len << 1) + 1 {
            return Err("Length field is invalid".into());
        }

        if let Ok(c) = checksum(self.address.0, self.control.0, self.length) {
            if c != self.checksum {
                return Err("Checksum is invalid".into());
            } else {
                Ok(())
            }
        } else {
            return Err("Checksum calculation failed".into());
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
        assert_eq!(d.verify().is_ok(), true);
    }
}
