#![allow(unused)]

use clap::{Arg, ArgAction, ArgGroup, Command, Parser};
use colored::*;
use core_lib::services::services::{
    generate_proof, get_file_commitment_and_selected_row, get_selected_row, verify_correct_selector,
};
use std::fs;
use std::process;

use serde::{Deserialize, Serialize};

const ROW: usize = 10;

#[derive(Deserialize)]
struct GenerateCommitmentAndProofRequest {
    row_titles: [String; ROW],
    row_contents: [String; ROW],
    row_selectors: [u64; ROW],
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    // pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main() {
    let cmd = clap::Command::new("medi0-cli")
        .about("Medi0 CLI")
        .version("0.1.0")
        .author("Medi0 Team")
        .subcommand(
            Command::new("gen-commitment")
                .about("Generates a commitment for a given file")
                .arg(
                    Arg::new("input-file")
                        .short('i')
                        .long("input-file")
                        .required(true)
                        .help("Input file needs to be a valid JSON file"),
                ),
        )
        .subcommand(
            Command::new("gen-proof")
                .about("Generates a proof for a given file")
                .arg(
                    Arg::new("input-file")
                        .short('i')
                        .long("input-file")
                        .required(true)
                        .help("Input file needs to be a valid JSON file"),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Verifies a given commitment")
                .arg(
                    Arg::new("input-file")
                        .short('i')
                        .long("input-file")
                        .required(true)
                        .help("Input file needs to be a valid JSON file"),
                ),
        );
    let args = Cli::parse();

    let matches = cmd.get_matches();
    /// Matches subcommand and runs the corresponding functions
    // let matches = match matches.subcommand() {
    match matches.subcommand() {
        Some(("gen-commitment", matches)) => {
            println!("gen-commitment ran successfully");
            /// get contents of a file
            let contents =
                fs::read_to_string(args.path).expect("Something went wrong reading the file");

            // get contents of a file
            // let json_contents =
            //     serde_json::from_str::<GenerateCommitmentAndProofRequest>(&contents)
            //         .expect("Failed to deserialize JSON");
            // println!("JSON contents:\n{:#?}", json_contents.row_selector_u64);

            // let commitment = get_file_commitment_and_selected_row(
            //     req.row_titles.to_owned(),
            //     req.row_contents.to_owned(),
            //     req.row_selectors.to_owned(),
            // );
            // println!("input-file: {}", matches.value_of("input-file").unwrap());
        }
        Some(("gen-proof", matches)) => {
            println!("gen-proof ran successfully");
        }
        Some(("verify", matches)) => {
            println!("verify ran successfully");
        }
        None => {
            println!("No subcommand was used");
            process::exit(1);
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
