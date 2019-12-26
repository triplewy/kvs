#[macro_use]
extern crate clap;

use clap::App;
use kvs::{KvsClient, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{env, process};

fn main() -> Result<()> {
    let yaml = load_yaml!("client.yml");
    let app = App::from_yaml(yaml);
    app.name(env!("CARGO_PKG_NAME"));

    let matches = App::from_yaml(yaml).get_matches();

    let socket = match matches.value_of("addr") {
        Some(v) => v.parse()?,
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
    };

    let mut client = KvsClient::new(socket)?;

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE").unwrap();
            client.set(key.to_owned(), value.to_owned())?;
            Ok(())
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let result = client.get(key.to_owned())?;
            if let Some(v) = result {
                println!("{}", v);
            } else {
                println!("Key not found");
            }
            Ok(())
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            client.remove(key.to_owned())?;
            Ok(())
        }
        _ => unreachable!(),
    }
}
