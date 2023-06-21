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

    fn to_bytes(&self) -> Vec<u8> {
        [
            self.id.to_be_bytes(),
            self.flags.to_be_bytes(),
            self.num_questions.to_be_bytes(),
            self.num_answers.to_be_bytes(),
            self.num_authorities.to_be_bytes(),
            self.num_additionals.to_be_bytes(),
        ]
        .concat()
    }
}

impl DNSQuestion {
    fn parse(buf: &[u8], cursor_start: usize) -> (usize, DNSQuestion) {
        let mut cursor = cursor_start;

        let (length, name) = decode_name(buf, cursor_start);
        cursor += length;

        let type_ = u16::from_be_bytes(buf[cursor..cursor + 2].try_into().unwrap());
        let class = u16::from_be_bytes(buf[cursor + 2..cursor + 4].try_into().unwrap());
        cursor += 4;

        (cursor - cursor_start, DNSQuestion { name, type_, class })
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            self.name.clone().into_bytes(),
            self.type_.to_be_bytes().to_vec(),
            self.class.to_be_bytes().to_vec(),
        ]
        .concat()
    }
}

fn encode_dns_name(name: &str) -> String {
    name.split('.')
        .map(|t| String::from_utf8((t.len() as u8).to_be_bytes().to_vec()).unwrap() + t)
        .collect::<Vec<_>>()
        .join("")
        + &String::from_utf8((0 as u8).to_be_bytes().to_vec()).unwrap()
}

const TYPE_A: u16 = 1;
const TYPE_NS: u16 = 2;

fn build_query(domain_name: &str, record_type: u16) -> Vec<u8> {
    const CLASS_IN: u16 = 1;

    let h = DNSHeader {
        id: rand::thread_rng().gen_range(1..65535),
        flags: 0,
        num_questions: 1,
        num_additionals: 0,
        num_authorities: 0,
        num_answers: 0,
    };

    let q = DNSQuestion {
        name: encode_dns_name(&domain_name),
        type_: record_type,
        class: CLASS_IN,
    };

    let mut query = h.to_bytes();
    query.extend(q.to_bytes());

    query
}

#[derive(Debug)]
enum DNSRecordData {
    Data(Vec<u8>),
    Ipv4Addr(Ipv4Addr),
    Name(String),
}

#[derive(Debug)]
struct DNSRecord {
    name: String,
    type_: u16,
    class: u16,
    ttl: u32,
    data: DNSRecordData,
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

        let data = match type_ {
            TYPE_NS => {
                let (length, name) = decode_name(buf, cursor);
                cursor += length;
                DNSRecordData::Name(name)
            }
            TYPE_A => {
                let ip = Ipv4Addr::new(
                    buf[cursor],
                    buf[cursor + 1],
                    buf[cursor + 2],
                    buf[cursor + 3],
                );
                cursor += 4;
                DNSRecordData::Ipv4Addr(ip)
            }
            _ => {
                let data = (&buf[cursor..cursor + data_length]).to_vec();
                cursor += data_length;
                DNSRecordData::Data(data)
            }
        };

        (
            cursor - cursor_start,
            DNSRecord {
                name,
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
    let mut cursor: usize = cursor_start;
    let mut labels: Vec<String> = Vec::new();
    let mut length: usize = buf[cursor].into();

    while length != 0 {
        if ((length as u8) & 0b11000000) != 0 {
            labels.push(decode_compressed_name(buf, cursor));
            cursor += 2;
            return (cursor - cursor_start, labels.join("."));
        } else {
            // Ignore length value in `start`.
            let (start, end) = (cursor + 1, cursor + length + 1);
            labels.push(String::from_utf8((&buf[start..end]).to_vec()).unwrap());
            cursor += length + 1;
            length = buf[cursor].into();
        }
    }
    cursor += 1; // For the 0 at the end.

    (cursor - cursor_start, labels.join("."))
}

fn decode_compressed_name(buf: &[u8], cursor_start: usize) -> String {
    let cursor =
        u16::from_be_bytes([(buf[cursor_start] & 0b00111111), buf[cursor_start + 1]]) as usize;
    decode_name(buf, cursor).1
}

fn send_query(ip_address: Ipv4Addr, domain_name: &str, record_type: u16) -> DNSPacket {
    let query = build_query(domain_name, record_type);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();

    socket
        .send_to(&query, (ip_address, 53))
        .expect("couldn't send data");

    let mut buf = [0; 1024];
    socket.recv_from(&mut buf).unwrap();

    DNSPacket::parse(&buf[..])
}

fn get_answer(packet: &DNSPacket) -> Option<&DNSRecordData> {
    match packet.answers.iter().find(|p| p.type_ == TYPE_A) {
        Some(answer) => Some(&answer.data),
        _ => None,
    }
}

fn get_nameserver(packet: &DNSPacket) -> &str {
    match packet.authorities.iter().find(|p| p.type_ == TYPE_NS) {
        Some(record) => match &record.data {
            DNSRecordData::Name(name) => name,
            _ => panic!("get_nameserver: no data"),
        },
        None => panic!("get_nameserver: no TYPE_NS authority"),
    }
}

fn get_nameserver_ip(packet: &DNSPacket) -> Option<Ipv4Addr> {
    match packet.additionals.iter().find(|p| p.type_ == TYPE_A) {
        Some(additional) => match additional.data {
            DNSRecordData::Ipv4Addr(ip) => Some(ip),
            _ => panic!("get_nameserver_ip: no Ipv4Addr"),
        },
        _ => None,
    }
}

fn resolve(domain_name: &str, record_type: u16) -> Ipv4Addr {
    let mut nameserver = Ipv4Addr::new(198, 41, 0, 4); // IP address for a.root-servers.net

    loop {
        println!("Querying {nameserver} for {domain_name}");
        let response = send_query(nameserver, domain_name, record_type);
        let answer = get_answer(&response);

        match answer {
            Some(data) => {
                return match data {
                    DNSRecordData::Ipv4Addr(ip) => *ip,
                    _ => panic!("resolve: something went wrong"),
                };
            }
            None => {
                let ns_ip = get_nameserver_ip(&response);
                match ns_ip {
                    Some(ip) => nameserver = ip,
                    None => {
                        let ns_domain = get_nameserver(&response);
                        nameserver = resolve(ns_domain, TYPE_A)
                    }
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    // let ip = lookup_domain("www.example.com".to_string());
    // println!("ip = {}", ip);

    // let ip = lookup_domain("recurse.com".to_string());
    // println!("ip = {}", ip);
    // let ip = lookup_domain("stace.dev".to_string());
    // println!("ip = {}", ip);
    // const TYPE_TXT: u16 = 16;
    // println!(
    //     "main: {:#?}",
    //     send_query("8.8.8.8", "example.com", TYPE_TXT).answers
    // );

    // println!(
    //     "main: {:#?}",
    //     send_query("198.41.0.4", "google.com", TYPE_A)
    // );

    let ip = resolve("google.com", TYPE_A);
    println!("ip = {ip}");

    let ip = resolve("facebook.com", TYPE_A);
    println!("ip = {ip}");

    let ip = resolve("twitter.com", TYPE_A);
    println!("ip = {ip}");

    let ip = resolve("stace.dev", TYPE_A);
    println!("ip = {ip}");
    Ok(())
}
