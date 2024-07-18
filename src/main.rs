use clap::{Args, Parser, Subcommand};
use cmux::types::{Address, Control, Frame, FrameBuilder};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate GSM 07.10 Frame by given address, control and content field
    #[command(visible_alias = "g")]
    Generate(GenerateArgs),
    /// Parse a byte array to GSM 07.10 Frame
    #[command(visible_alias = "p")]
    Parse(ParseArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// address field
    #[arg(short, long, default_value = "7")]
    address: String,
    /// control field
    #[arg(short, long, default_value = "EF")]
    control: String,
    /// content field
    content: String,
}

#[derive(Args)]
struct ParseArgs {
    /// Bytes array like string. Example: "F9010203F9 F9010203F9"
    hexstring: Option<String>,
}

fn hexstring_to_bytes(hexstring: &str) -> Vec<u8> {
    let hexstring = hexstring
        .to_string()
        .replace([' ', '\n'], "")
        .replace("0x", "");
    hexstring
        .as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

fn hexbyte_to_bytes(hexbyte: &str) -> u8 {
    let hexbyte = hexbyte.replace("0x", "");
    u8::from_str_radix(&hexbyte, 16).unwrap()
}

fn string_eater<'a>(ori: &'a str, d: &str) -> Option<(&'a str, &'a str)> {
    let len = d.len();
    let start = match ori.find(d) {
        Some(i) => i,
        None => return None,
    };
    let end = match ori[start + len..].find(d) {
        Some(i) => i,
        None => return None,
    };
    Some((
        &ori[start..start + end + 2 * len],
        &ori[start + end + 2 * len..],
    ))
}

fn generate(address: &str, control: &str, content: String) -> Frame {
    let address = Address::from_bits(hexbyte_to_bytes(address));
    let control = Control::from_bits(hexbyte_to_bytes(control));

    FrameBuilder::default()
        .with_address(address)
        .with_control(control)
        .with_content(content)
        .build()
}

fn parse(hexstring: String) -> Vec<Frame> {
    let hex = hexstring.to_uppercase();
    let mut hex = hex.as_str();
    let mut frames = Vec::new();
    while let Some((curr, rest)) = string_eater(hex, "F9") {
        let frame = Frame::from_bytes(hexstring_to_bytes(curr));
        frames.push(frame);
        hex = rest;
    }
    frames
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => {
            let p = generate(&args.address, &args.control, args.content);
            println!("{}", p.to_hex_string());
            println!("{:?}", p);
        }
        Commands::Parse(args) => {
            if let Some(hexstring) = args.hexstring {
                let frames = parse(hexstring);
                for frame in frames {
                    let verify = match frame.verify() {
                        Ok(_) => "OK".to_string(),
                        Err(e) => e.to_string(),
                    };
                    println!(
                        "Origin: {} Verify: {}\n{:?}",
                        frame.to_hex_string().to_uppercase(),
                        verify,
                        frame
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexstring_to_bytes() {
        assert_eq!(hexstring_to_bytes("F9010203F9"), vec![249, 1, 2, 3, 249]);
        assert_eq!(
            hexstring_to_bytes("F9 01 02 03 F9"),
            vec![249, 1, 2, 3, 249]
        );
        assert_eq!(
            hexstring_to_bytes("F9\n01\n02\n03\nF9"),
            vec![249, 1, 2, 3, 249]
        );
        assert_eq!(
            hexstring_to_bytes("0xF9 0x01 0x02 0x03 0xF9"),
            vec![249, 1, 2, 3, 249]
        );
    }

    #[test]
    fn test_hexbyte_to_bytes() {
        assert_eq!(hexbyte_to_bytes("F9"), 249);
        assert_eq!(hexbyte_to_bytes("0xF9"), 249);
    }

    #[test]
    fn test_string_eater() {
        let s = "F9010203F9\r\nF9010203F9F9010203F9F9";

        let (curr, rest) = string_eater(s, "F9").unwrap();
        assert_eq!(curr, "F9010203F9",);
        assert_eq!(rest, "\r\nF9010203F9F9010203F9F9",);

        let (curr, rest) = string_eater(rest, "F9").unwrap();
        assert_eq!(curr, "F9010203F9",);
        assert_eq!(rest, "F9010203F9F9",);

        let (curr, rest) = string_eater(rest, "F9").unwrap();
        assert_eq!(curr, "F9010203F9",);
        assert_eq!(rest, "F9",);

        assert_eq!(string_eater(rest, "F9"), None);
    }

    #[test]
    fn test_generate() {
        let frame = generate("7", "EF", "010203".to_string());
        assert_eq!(frame.to_hex_string(), "f907ef113031303230330d0a2bf9");
    }

    #[test]
    fn test_parse() {
        let str = r#"
        F9033F011CF9
        F9073F01DEF9
        F90B3F0159F9
        F90F3F019BF9
        F9133F0196F9
        F9173F0154F9
        F91B3F01D3F9
        F91F3F0111F9
        "#;
        let frames = parse(str.to_string());
        assert_eq!(frames.len(), 8);
        let mut i = 0;
        str.to_string().replace(' ', "").split('\n').for_each(|s| {
            if !s.is_empty() {
                assert_eq!(frames[i].to_hex_string().to_uppercase(), s);
                assert!(frames[i].verify().is_ok());
                i += 1;
            }
        });
    }
}
