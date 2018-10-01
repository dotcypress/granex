#![feature(test)]

use clap::{App, Arg};
use std::env::current_dir;
use std::path::PathBuf;
use std::fs::canonicalize;

mod generator;

fn main() {
  let args = App::new("granex")
    .version(env!("CARGO_PKG_VERSION"))
    .author("Vitaly Domnikov <oss@vitaly.codes>")
    .about("Tor v3 Vanity Address Generator")
    .arg(
      Arg::with_name("prefix")
        .short("p")
        .long("prefix")
        .help("Sets the address prefix")
        .takes_value(true)
        .required(true)
        .display_order(0)
        .index(1),
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output")
        .help("Output directory")
        .takes_value(true)
        .display_order(1)
        .index(2),
    )
    .get_matches();
  let prefix = args.value_of("prefix").unwrap();
  let output_dir = match args.value_of("output") {
    Some(val) => canonicalize(&PathBuf::from(val)).unwrap(),
    None => current_dir().unwrap(),
  };
  generator::start(prefix, output_dir).unwrap();
}
