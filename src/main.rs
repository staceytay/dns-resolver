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
    fn parse_question(buf: &[u8], cursor: usize) -> (usize, DNSQuestion) {
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

// From https://doc.rust-lang.org/std/primitive.u16.html#method.from_be_bytes
fn read_be_u16(input: &mut &[u8]) -> u16 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u16>());
    *input = rest;
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

#[derive(Debug)]
struct DNSRecord {
    name: Vec<u8>,
    type_: u16,
    class: u16,
    ttl: u32,
    data: Vec<u8>,
}

struct DNSPacket {}

impl DNSRecord {
    fn parse_record(buf: &[u8], cursor_start: usize) -> (usize, DNSRecord) {
        println!("parse_record: cursor_start = {}", cursor_start);
        let mut cursor: usize = cursor_start;
        let (length, name) = decode_name(buf, cursor);
        cursor += length;

        println!("parse_record: cursor = {}", cursor);
        println!("parse_record: length = {}, name = {}", length, name.clone());
        println!("{}", show(&buf[cursor..]));

        let data_length: usize =
            u16::from_be_bytes(buf[cursor + 8..cursor + 10].try_into().unwrap()).into();
        let dns_record = DNSRecord {
            name: name.into(),
            type_: u16::from_be_bytes(buf[cursor..cursor + 2].try_into().unwrap()),
            class: u16::from_be_bytes(buf[cursor + 2..cursor + 4].try_into().unwrap()),
            ttl: u32::from_be_bytes(buf[cursor + 4..cursor + 8].try_into().unwrap()),
            data: (&buf[cursor + 10..cursor + 10 + data_length]).to_vec(),
        };
        cursor += (10 + data_length);

        (cursor - cursor_start, dns_record)

        //         name = decode_name(reader)
        // data = reader.read(10)
        // type_, class_, ttl, data_len = struct.unpack("!HHIH", data)
        // data = reader.read(data_len)
        // return DNSRecord(name, type_, class_, ttl, data)
    }
}

impl DNSPacket {
    // fn parse(&self, buf: &[u8]) -> DNSPacket {}
}

// TODO: Right now it's a big mess where cursor is treated as length and vice
// versa, need to standardise what to do so that I can read the buffer from
// caller and handle the indexing correctly
// One approach is to pass in a cursor but just return the length read? And use
// this to avoid the hardcording within function
//

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
            // return (2, labels.join("."));
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
    println!(
        "decode_compressed_name: buf[cursor_start] = {},  buf[cursor_start] & 0b00111111 = {}, buf[cursor_start + 1] = {}",
        buf[cursor_start],
        buf[cursor_start] & 0b00111111,
        buf[cursor_start + 1],
    );
    let pointer = u16::from_be_bytes([(buf[cursor_start] & 0b00111111), buf[cursor_start + 1]]);
    println!("decode_compressed_name: pointer = {}", pointer);
    decode_name(buf, pointer as usize).1

    // println!("decode_compressed_name: {}: {}", name_length, name.clone());
    // String::from("TODO")

    // //    pointer_bytes = bytes([]) + reader.read(1)
    // pointer = struct.unpack("!H", pointer_bytes)[0]
    // current_pos = reader.tell()
    // reader.seek(pointer)
    // result = decode_name(reader)
    // reader.seek(current_pos)
    // return result
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

        println!("{}", show(&buf[..]));

        let h = DNSHeader::parse_header(&buf[..]);
        println!("h: {:?}", h);

        const HEADER_LENGTH: usize = 12;
        let (q_len, q) = DNSQuestion::parse_question(&buf[..], HEADER_LENGTH);
        println!("q: {}: {:?}", q_len, q);

        let (r_len, r) = DNSRecord::parse_record(&buf[..], HEADER_LENGTH + q_len);
        println!("r: {}: {:?}", r_len, r);
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
