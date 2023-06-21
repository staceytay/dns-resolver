// TODO: brief explainer on how DNS works and links to resources and talks and
// references. Also perhaps the new to Rust and just translated this from
// Julia's python code caveat should be in this comment too.

use dns::{resolve, TYPE_A};

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
