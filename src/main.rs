use gsm0710::types::{Address, Frame, FrameBuilder};

fn main() {
    let p = FrameBuilder::default()
        .with_address(Address::default())
        .with_content("AT+CMUX?".to_string())
        .build();
    println!("{}", p.to_hex_string());
    println!("{:?}", p);

    let d = Frame::from_bytes(p.to_bytes());
    println!("{:?}", d);
}
