use clap::Parser;
use std::fs;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    list: bool,

    #[clap(short, long)]
    all: bool,

    paths: Vec<String>,
}

fn main() {
    let args = Args::parse();
    //println!("Hello {:?}!", args.paths)

    let mut paths = args.paths;
    if paths.is_empty() {
        paths.push(String::from("."));
    }

    let seperator = if args.list { "\n" } else { " " };

    let files: Vec<String> = paths
        .iter()
        .flat_map(|p| {
            fs::read_dir(p)
                .unwrap()
                .map(|f| f.unwrap().path().display().to_string())
                .collect::<Vec<String>>()
        })
        .collect();

    println!("{}", files.join(seperator));
}
