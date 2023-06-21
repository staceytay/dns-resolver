# Rust DNS Resolver

Toy DNS resolver written in Rust, inspired by Julia Evans' [Implement DNS in a weekend](https://implement-dns.wizardzines.com).

## Features
- [x] Return IP address for A record queries
- [x] Show trace of DNS nameservers queried to get answer
- [ ] Support querying other record types, `AAAA`, `MX`, and `NS`
- [ ] Show more details from answer section not just IP
- [ ] Show authorities and additionals section

## Usage

```bash
$ cargo run -- google.com
Querying a.root-servers.net (198.41.0.4) for google.com
Querying e.gtld-servers.net (192.12.94.30) for google.com
Querying ns2.google.com (216.239.34.10) for google.com
ip = 142.250.80.110

$ cargo run -- stace.dev
Querying a.root-servers.net (198.41.0.4) for stace.dev
Querying ns-tld5.charlestonroadregistry.com (216.239.60.105) for stace.dev
Querying a.root-servers.net (198.41.0.4) for marjory.ns.cloudflare.com
Querying e.gtld-servers.net (192.12.94.30) for marjory.ns.cloudflare.com
Querying ns3.cloudflare.com (162.159.0.33) for marjory.ns.cloudflare.com
Querying marjory.ns.cloudflare.com (173.245.58.193) for stace.dev
ip = 172.67.141.136
```
