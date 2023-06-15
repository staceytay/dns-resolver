// TODO: brief explainer on how DNS works and links to resources and talks and
// references. Also perhaps the new to Rust and just translated this from
// Julia's python code caveat should be in this comment too.

use rand::Rng;
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::str;

#[derive(Debug)]
struct DNSHeader {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}

#[derive(Debug)]
struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}

impl DNSHeader {
    fn parse_header(buf: &[u8]) -> DNSHeader {
        DNSHeader {
            id: u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            flags: u16::from_be_bytes(buf[2..4].try_into().unwrap()),
            num_questions: u16::from_be_bytes(buf[4..6].try_into().unwrap()),
            num_additionals: u16::from_be_bytes(buf[6..8].try_into().unwrap()),
            num_authorities: u16::from_be_bytes(buf[8..10].try_into().unwrap()),
            num_answers: u16::from_be_bytes(buf[10..12].try_into().unwrap()),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, &'static str> {
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
    // TODO: Return as Result.
    fn parse_question(buf: &[u8]) -> (usize, DNSQuestion) {
        let (length, name) = decode_name(buf);
        let cursor = length + 1;
        let type_ = u16::from_be_bytes(buf[cursor..cursor + 2].try_into().unwrap());
        let class = u16::from_be_bytes(buf[cursor + 2..cursor + 4].try_into().unwrap());
        println!("parse_question: {}: {}", cursor, name.clone());
        (cursor + 4, DNSQuestion { name, type_, class })
    }

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
}

const TYPE_A: u16 = 1;

fn build_query(domain_name: String, record_type: u16) -> Vec<u8> {
    const CLASS_IN: u16 = 1;
    const RECURSION_DESIRED: u16 = 1 << 8;

    let h = DNSHeader {
        id: rand::thread_rng().gen_range(1..65535),
        flags: RECURSION_DESIRED,
        num_questions: 1,
        num_additionals: 0,
        num_authorities: 0,
        num_answers: 0,
    };

    let q = DNSQuestion {
        name: encode_dns_name(domain_name),
        type_: record_type,
        class: CLASS_IN,
    };

    let mut query = h.to_bytes().unwrap();
    query.extend(q.to_bytes().unwrap());

    query
}

// From https://doc.rust-lang.org/std/primitive.u16.html#method.from_be_bytes
fn read_be_u16(input: &mut &[u8]) -> u16 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u16>());
    *input = rest;
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

struct DNSRecord {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u16,
    data: Vec<u8>,
}

struct DNSPacket {}

impl DNSRecord {
    fn parse_record() {}
}

impl DNSPacket {
    // fn parse(&self, buf: &[u8]) -> DNSPacket {}
}

fn decode_name(buf: &[u8]) -> (usize, String) {
    let mut labels: Vec<String> = Vec::new();
    let mut cursor: usize = 0;
    let mut length: usize = buf[0].into();

    while length != 0 {
        // Ignore length value in `start`.
        let (start, end) = (cursor + 1, cursor + length + 1);
        labels.push(String::from_utf8((&buf[start..end]).to_vec()).unwrap());
        cursor += length + 1;
        length = buf[cursor].into();
    }

    (cursor, labels.join("."))
}

fn main() -> std::io::Result<()> {
    {
        let query = build_query("www.example.com".to_string(), TYPE_A);
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;

        socket
            .send_to(&query, "8.8.8.8:53")
            .expect("couldn't send data");

        let mut buf = [0; 1024];
        let (amt, src) = socket.recv_from(&mut buf)?;

        let h = DNSHeader::parse_header(&buf[..]);
        println!("{:?}", h);

        let (len, q) = DNSQuestion::parse_question(&buf[12..]);
        println!("{:?}", q);
        println!("{:?}", q.name);
        println!("{:?}", String::from("www.example.com"));

        println!("{}", show(&buf[12 + len..]));
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
