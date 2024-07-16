use crc::Crc;
use hex::ToHex;
use std::error::Error;
use std::fmt::Display;

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

#[derive(Debug, Clone)]
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
/// +------+------+------+------+------+------+------+--- --+------+
/// | Bit  |   1  |   2  |   3  |   4  |   5  |   6  |   7  |   8  |
/// +------+------+------+------+------+------+------+------+------+
/// | Data |  EA  | C/R  |                 D L C I                 |
/// +------+------+------+------+------+------+------+------+------+
///
/// * EA: Extended Address Bit. This bit is always set to 1.
/// * C/R: Command/Response Bit. See below.
/// * DLCI: Data Link Connection Identifier. This field is 6 bits long.
///
/// +------------------+------------------------+-----------+
/// | Command/response | Direction              | C/R value |
/// +------------------+------------------------+-----------+
/// | Command          | Initiator -> Responder | 1         |
/// |                  | Responder -> Initiator | 0         |
/// | Response         | Initiator -> Responder | 0         |
/// |                  | Responder -> Initiator | 1         |
/// +------------------+------------------------+-----------+
#[derive(Debug, Clone)]
struct Address(u8);

impl Address {
    pub fn with_cr(&self, cr: bool) -> Address {
        Address(bit_set_to(self.0, 1, cr))
    }

    pub fn with_dlci(&self, dlci: DLCI) -> Address {
        Address((self.0 & 0x3) | ((dlci as u8 & 0x3F) << 2))
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
        let dlci = DLCI::from(self.clone());
        write!(f, "{}", dlci)
    }
}

struct PacketBuilder {
    address: Option<Address>,
    content: Option<String>,
    control: u8,
}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder {
            address: None,
            content: None,
            control: 0xEF,
        }
    }

    fn length(&self) -> Result<usize, Box<dyn Error>> {
        match &self.content {
            Some(content) => {
                let len = content.len();
                if len > 2usize.pow(7) {
                    Ok(len << 1)
                } else {
                    Ok((len << 1) + 1)
                }
            }
            None => Err("Content is required".into()),
        }
    }

    fn checksum(&self) -> Result<u8, Box<dyn Error>> {
        match &self.address {
            None => Err("Address is required".into()),
            Some(addr) => {
                let crc = Crc::<u8>::new(&crc::CRC_8_ROHC);
                let data: Vec<u8> = vec![addr.0, self.control, self.length()? as u8];
                let crc_value = crc.checksum(&data);
                Ok(!crc_value as u8)
            }
        }
    }

    pub fn address(&mut self, address: Address) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn content(&mut self, content: String) -> &mut Self {
        if content.ends_with("\r\n") {
            self.content = Some(content);
        } else {
            self.content = Some(format!("{}\r\n", content));
        }
        self
    }

    pub fn control(&mut self, control: u8) -> &mut Self {
        self.control = control;
        self
    }

    pub fn build(&self) -> Packet {
        Packet {
            header: 0xF9,
            address: self.address.clone().unwrap().0,
            control: self.control,
            length: self.length().unwrap() as u16,
            content: self.content.clone().unwrap(),
            checksum: self.checksum().unwrap(),
            footer: 0xF9,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Packet {
    header: u8,
    address: u8,
    control: u8,
    length: u16,
    content: String,
    checksum: u8,
    footer: u8,
}

impl Packet {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![self.header, self.address.clone().into(), self.control];
        if self.length > 255 {
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

    fn to_hex_string(&self) -> String {
        self.to_bytes().encode_hex::<String>()
    }

    fn from_bytes(data: Vec<u8>) -> Packet {
        let mut p = 0;
        let header = data[p];
        p += 1;
        let address = data[1];
        p += 1;
        let control = data[2];
        p += 1;
        let length = if data[3] & 0x1 == 0 {
            p += 2;
            ((data[3] as u16) << 8) | data[4] as u16
        } else {
            p += 1;
            data[3] as u16
        };
        let content = String::from_utf8(data[p..data.len() - 2].to_vec()).unwrap();
        let checksum = data[data.len() - 2];
        let footer = data[data.len() - 1];
        Packet {
            header,
            address,
            control,
            length,
            content,
            checksum,
            footer,
        }
    }
}

fn main() {
    let p = PacketBuilder::new()
        .address(Address::default())
        .content("AT+CMUX?".to_string())
        .build();
    println!("{}", p.to_hex_string());
    println!("{:?}", p);

    let d = Packet::from_bytes(p.to_bytes());
    println!("{:?}", d);
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;

    #[test]
    fn test_packet_builder() {
        let p = PacketBuilder::new()
            .address(Address::default())
            .content("AT+CMUX?".to_string())
            .build();
        assert_eq!(p.header, 0xF9);
        assert_eq!(p.address, 0x07);
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
        let p = PacketBuilder::new()
            .address(Address::default())
            .content(content)
            .build();
        assert_eq!(p.length, len as u16);
    }

    #[test]
    fn test_packet_to_bytes() {
        let p = PacketBuilder::new()
            .address(Address::default())
            .content("AT+CMUX?".to_string())
            .build();
        let data = p.to_hex_string();
        assert_eq!(data, "f907ef1541542b434d55583f0d0a2cf9".to_string());
    }

    #[test]
    fn test_packet_from_bytes() {
        let content = "AT+CMUX?".to_string();
        let len = (content.len() + 2) * 2 + 1; // less than 128, so bit 1 is set 1
        let p = PacketBuilder::new()
            .address(Address::default())
            .content("AT+CMUX?".to_string())
            .build();
        let d = Packet::from_bytes(p.to_bytes());
        assert_eq!(p, d);
        assert_eq!(d.length, len as u16);
        assert_eq!(d.address, Address::default().0);
    }

    #[test]
    fn test_packet_from_bytes_very_long() {
        let content = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .to_string()
            .repeat(10);
        let len = (content.len() + 2) * 2; // more than 128, so bit 1 is set zero
        let p = PacketBuilder::new()
            .address(Address::default())
            .content(content)
            .build();
        let d = Packet::from_bytes(p.to_bytes());
        assert_eq!(p, d);
        assert_eq!(d.length, len as u16);
    }
}
