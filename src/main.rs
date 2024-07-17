use gsm0710::types::{Address, Frame, FrameBuilder};

fn main() {
    let p = FrameBuilder::default()
        .address(Address::default())
        .content("AT+CMUX?".to_string())
        .build();
    println!("{}", p.to_hex_string());
    println!("{:?}", p);

    let d = Frame::from_bytes(p.to_bytes());
    println!("{:?}", d);
}
