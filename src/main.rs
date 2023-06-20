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
    fn parse(buf: &[u8]) -> DNSHeader {
        DNSHeader {
            id: u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            flags: u16::from_be_bytes(buf[2..4].try_into().unwrap()),
            num_questions: u16::from_be_bytes(buf[4..6].try_into().unwrap()),
            num_answers: u16::from_be_bytes(buf[6..8].try_into().unwrap()),
            num_authorities: u16::from_be_bytes(buf[8..10].try_into().unwrap()),
            num_additionals: u16::from_be_bytes(buf[10..12].try_into().unwrap()),
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
    fn parse(buf: &[u8], cursor: usize) -> (usize, DNSQuestion) {
        let (length, name) = decode_name(buf, cursor);
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

#[derive(Debug)]
struct DNSRecord {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u32,
    data: Vec<u8>,
}

#[derive(Debug)]
struct DNSPacket {
    header: DNSHeader,
    questions: Vec<DNSQuestion>,
    answers: Vec<DNSRecord>,
    authorities: Vec<DNSRecord>,
    additionals: Vec<DNSRecord>,
}

impl DNSRecord {
    fn parse(buf: &[u8], cursor_start: usize) -> (usize, DNSRecord) {
        let mut cursor = cursor_start;

        let (length, name) = decode_name(buf, cursor);
        cursor += length;

        let (type_, class, ttl, data_length) = (
            u16::from_be_bytes(buf[cursor..cursor + 2].try_into().unwrap()),
            u16::from_be_bytes(buf[cursor + 2..cursor + 4].try_into().unwrap()),
            u32::from_be_bytes(buf[cursor + 4..cursor + 8].try_into().unwrap()),
            (u16::from_be_bytes(buf[cursor + 8..cursor + 10].try_into().unwrap()) as usize),
        );
        cursor += 10;

        let data = (&buf[cursor..cursor + data_length]).to_vec();
        cursor += data_length;

        (
            cursor - cursor_start,
            DNSRecord {
                name: name.into(),
                type_,
                class,
                ttl,
                data,
            },
        )
    }
}

impl DNSPacket {
    fn parse(buf: &[u8]) -> DNSPacket {
        let header = DNSHeader::parse(buf);

        const HEADER_LENGTH: usize = 12;
        let mut cursor = HEADER_LENGTH;

        let mut questions = Vec::new();
        for _ in 0..header.num_questions {
            let (length, question) = DNSQuestion::parse(buf, cursor);
            cursor += length;
            questions.push(question);
        }

        let mut answers = Vec::new();
        for _ in 0..header.num_answers {
            let (length, answer) = DNSRecord::parse(buf, cursor);
            cursor += length;
            answers.push(answer);
        }

        let mut authorities = Vec::new();
        for _ in 0..header.num_authorities {
            let (length, authority) = DNSRecord::parse(buf, cursor);
            cursor += length;
            authorities.push(authority);
        }

        let mut additionals = Vec::new();
        for _ in 0..header.num_additionals {
            let (length, additional) = DNSRecord::parse(buf, cursor);
            cursor += length;
            additionals.push(additional);
        }

        DNSPacket {
            header,
            questions,
            answers,
            authorities,
            additionals,
        }
    }
}

// Returns length of bytes read from buf and decoded name.
fn decode_name(buf: &[u8], cursor_start: usize) -> (usize, String) {
    println!("decode_name: cursor_start = {}", cursor_start);
    let mut cursor: usize = cursor_start;
    let mut labels: Vec<String> = Vec::new();
    let mut length: usize = buf[cursor].into();

    while length != 0 {
        if ((length as u8) & 0b11000000) != 0 {
            println!("decode_name: bitwise triggered at {}", cursor);
            // Assumption: if the name is compressed, we'll get the entire domain name here.
            labels.push(decode_compressed_name(buf, cursor));
            cursor += 2;
            break;
        } else {
            // Ignore length value in `start`.
            let (start, end) = (cursor + 1, cursor + length + 1);
            labels.push(String::from_utf8((&buf[start..end]).to_vec()).unwrap());
            cursor += length + 1;
            length = buf[cursor].into();
        }
    }

    (cursor - cursor_start, labels.join("."))
}

fn decode_compressed_name(buf: &[u8], cursor_start: usize) -> String {
    let pointer =
        u16::from_be_bytes([(buf[cursor_start] & 0b00111111), buf[cursor_start + 1]]) as usize;
    decode_name(buf, pointer).1
}

// TODO: Maybe update domain_name to &str?
fn lookup_domain(domain_name: String) -> String {
    let query = build_query(domain_name, TYPE_A);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();

    socket
        .send_to(&query, "8.8.8.8:53")
        .expect("couldn't send data");

    let mut buf = [0; 1024];
    socket.recv_from(&mut buf).unwrap();

    let p = DNSPacket::parse(&buf[..]);
    println!("p: {:#?}", p);

    p.answers[0]
        .data
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn main() -> std::io::Result<()> {
    let ip = lookup_domain("www.example.com".to_string());
    println!("ip = {}", ip);
    let ip = lookup_domain("recurse.com".to_string());
    println!("ip = {}", ip);
    let ip = lookup_domain("stace.dev".to_string());
    println!("ip = {}", ip);
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
