use clap::{Args, Parser, Subcommand};
use gsm0710::types::{Address, Control, Frame, FrameBuilder};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
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
    #[arg(short, long)]
    content: String,
}

#[derive(Args)]
struct ParseArgs {
    /// Bytes array like string. Example: "F9010203F9"
    hexstring: Option<String>,
}

fn hexstring_to_bytes(hexstring: &str) -> Vec<u8> {
    let hexstring = hexstring
        .to_string()
        .replace(" ", "")
        .replace("\n", "")
        .replace("0x", "");
    hexstring
        .as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

fn hexbyte_to_bytes(hexbyte: &str) -> u8 {
    u8::from_str_radix(hexbyte, 16).unwrap()
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => {
            let p = FrameBuilder::default()
                .with_address(Address::from_bits(hexbyte_to_bytes(&args.address)))
                .with_control(Control::from_bits(hexbyte_to_bytes(&args.control)))
                .with_content(args.content)
                .build();
            println!("{}", p.to_hex_string());
            println!("{:?}", p);
        }
        Commands::Parse(args) => {
            let d = Frame::from_bytes(hexstring_to_bytes(&args.hexstring.unwrap()));
            println!("{:?}", d);
            if d.verify().is_err() {
                println!("An error occurred while verifying the frame");
            }
        }
    }
}
