#[macro_use]
extern crate clap;

use clap::App;
use kvs::{KvStore, KvStoreError, Result};
use std::{env, process};

fn main() -> Result<()> {
    let curr_dir = env::current_dir().unwrap();
    let mut store = KvStore::open(curr_dir.as_path())?;
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    app.name(env!("CARGO_PKG_NAME"));

    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE").unwrap();
            store.set(key.to_owned(), value.to_owned())
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let value = store.get(key.to_owned())?;
            match value {
                Some(v) => println!("{}", v),
                None => println!("Key not found"),
            };
            Ok(())
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            match store.remove(key.to_owned()) {
                Ok(_) => Ok(()),
                Err(KvStoreError::KeyNotFoundError {}) => {
                    println!("Key not found");
                    process::exit(1)
                }
                Err(e) => Err(e),
            }
        }
        _ => unreachable!(),
    }
}
