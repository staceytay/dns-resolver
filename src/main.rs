// TODO: brief explainer on how DNS works and links to resources and talks and
// references. Also perhaps the new to Rust and just translated this from
// Julia's python code caveat should be in this comment too.

use dns::{resolve, Config};
use std::env;
use std::process;

fn main() -> std::io::Result<()> {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let ip = resolve(&config.domain_name, config.record_type).unwrap_or_else(|err| {
        eprintln!("Unexpected error: {err}");
        process::exit(1);
    });
    println!("ip = {ip}");

    Ok(())
}
