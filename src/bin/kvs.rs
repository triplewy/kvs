#[macro_use]
extern crate clap;

use clap::App;
use kvs::KvStore;
use std::{env, process};

fn main() {
    // let store = KvStore::new();
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    app.name(env!("CARGO_PKG_NAME"));

    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        ("get", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        ("rm", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        _ => unreachable!(),
    }
}
