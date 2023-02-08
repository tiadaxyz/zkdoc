#![allow(unused)]

use std::process;
use clap::{Arg, ArgAction, ArgGroup, Command};
use colored::*;

fn main(){
    let cmd = clap::Command::new("medi0-cli")
        .about("Medi0 CLI")
        .version("0.1.0")
        .author("Medi0 Team")
        .subcommand(
            Command::new("gen-commitment")
                .about("Generates a commitment for a given file")
                .arg(Arg::new("input-file")
                    .short('i')
                    .long("input-file")
                    .required(true)
                    .help("Input file needs to be a valid JSON file")))
        .subcommand(
            Command::new("gen-proof")
                .about("Generates a proof for a given file")
                .arg(Arg::new("input-file")
                    .short('i')
                    .long("input-file")
                    .required(true)
                    .help("Input file needs to be a valid JSON file"))
        )
        .subcommand(
            Command::new("verify")
                .about("Verifies a given commitment")
                .arg(Arg::new("input-file")
                    .short('i')
                    .long("input-file")
                    .required(true)
                    .help("Input file needs to be a valid JSON file"))
        );

    let matches = cmd.get_matches();
    /// Matches subcommand and runs the corresponding functions
    let matches = match matches.subcommand() {
        Some(("gen-commitment", matches)) => {
            println!("gen-commitment ran successfully");
            // println!("input-file: {}", matches.value_of("input-file").unwrap());
        },
        Some(("gen-proof", matches)) => {
            println!("gen-proof ran successfully");
        },
        Some(("verify", matches)) => {
            println!("verify ran successfully");
        },
        _ => unreachable!("clap should ensure we don't get here"),
    };
}