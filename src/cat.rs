use clap::Parser;
use std::fs;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();
    args.paths
        .iter()
        .for_each(|p| println!("{}", fs::read_to_string(p).unwrap()));
}
