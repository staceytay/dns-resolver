// TODO: brief explainer on how DNS works and links to resources and talks and
// references. Also perhaps the new to Rust and just translated this from
// Julia's python code caveat should be in this comment too.

use rand::Rng;
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::str;

struct Header {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}

struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}

impl Header {
    // fn to_bytes(&self) -> Result<&'static [u8; 12], &'static str> {
    fn to_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let bytes2: [u8; 2] = [
            self.id.to_be_bytes()[0],
            self.id.to_be_bytes()[1],
            // self.flags.to_be_bytes(),
            // self.num_questions.to_be_bytes(),
            // self.num_answers.to_be_bytes(),
            // self.num_authorities.to_be_bytes(),
            // self.num_additionals.to_be_bytes(),
        ];
        let bytes: Vec<u8> = [
            self.id.to_be_bytes(),
            self.flags.to_be_bytes(),
            self.num_questions.to_be_bytes(),
            self.num_answers.to_be_bytes(),
            self.num_authorities.to_be_bytes(),
            self.num_additionals.to_be_bytes(),
        ]
        .concat();
        Ok(bytes)
    }
}

impl DNSQuestion {
    fn to_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let bytes: Vec<u8> = [
            self.name.clone().into_bytes(),
            self.type_.to_be_bytes().to_vec(),
            self.class.to_be_bytes().to_vec(),
        ]
        .concat();
        Ok(bytes)
    }
}

fn encode_dns_name(name: String) -> String {
    name.split('.')
        .map(|t| String::from_utf8((t.len() as u8).to_be_bytes().to_vec()).unwrap() + t)
        .collect::<Vec<_>>()
        .join("")
        + &String::from_utf8((0 as u8).to_be_bytes().to_vec()).unwrap()
    // + b'\x00'
}

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;

use std::{fmt::Write, num::ParseIntError};

// From https://stackoverflow.com/questions/52987181/how-can-i-convert-a-hex-string-to-a-u8-slice
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn main() -> std::io::Result<()> {
    {
        let id = rand::thread_rng().gen_range(1..65535);
        const RECURSION_DESIRED: u16 = 1 << 8;
        let h = Header {
            id: 0x8298,
            // id: id,
            flags: RECURSION_DESIRED,
            num_questions: 1,
            num_additionals: 0,
            num_authorities: 0,
            num_answers: 0,
        };

        let q = DNSQuestion {
            name: encode_dns_name("www.example.com".to_string()),
            type_: TYPE_A,
            class: CLASS_IN,
        };

        // println!("{:?}", h.to_bytes());
        // println!("{:?}", q.to_bytes());

        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        // socket
        //     .send_to(b"D\xcb\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01", "8.8.8.8:53")
        //     .expect("couldn't send data");
        let mut query = h.to_bytes().unwrap();
        query.extend(q.to_bytes().unwrap());

        println!(
            "{:?}",
            decode_hex("82980100000100000000000003777777076578616d706c6503636f6d0000010001")
                .unwrap()
        );
        println!("{:?}", query);
        socket
            .send_to(&query, "8.8.8.8:53")
            .expect("couldn't send data");
        // let mut buf = [0; 1024];
        // let (amt, src) = socket.recv_from(&mut buf)?;
        // println!("{}", show(&buf));
    } // the socket is closed here
    Ok(())
}

use std::ascii::escape_default;

// From https://stackoverflow.com/questions/41449708/how-to-print-a-u8-slice-as-text-if-i-dont-care-about-the-particular-encoding
fn show(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }
    visible
}
