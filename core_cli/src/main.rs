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

#[derive(Deserialize)]
struct ProofVerificationRequest {
    proof: Vec<u8>,
    row_title: String,
    row_content: String,
    commitment: String,
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
            Command::new("verify-proof")
                .about("Verifies a given commitment")
                .arg(
                    Arg::new("input-file")
                        .short('i')
                        .long("input-file")
                        .required(true)
                        .help("Input file needs to be a valid JSON file"),
                ),
        );

    let matches = cmd.get_matches();
    /// Matches subcommand and runs the corresponding functions
    match matches.subcommand() {
        Some(("gen-commitment", matches)) => {
            println!("{}", "Generating commitment...".cyan().bold());
            /// Get the input file path
            let input_file = matches.get_one::<String>("input-file").unwrap();
            /// Parse the contents of the input file
            let contents = fs::read_to_string(input_file).unwrap_or_else(|_| {
                panic!(
                    "{}",
                    "Something went wrong reading the file".red().to_string()
                )
            });
            let json_contents =
                serde_json::from_str::<GenerateCommitmentAndProofRequest>(&contents)
                    .unwrap_or_else(|_| { panic!("{}", "Failed to deserialize JSON file.\nRefer to the sample 'gen-commitment.json' as reference".red().to_string()) });
            println!("{}: {}", "Input file path".blue().bold(), input_file.blue());

            /// Calling the function to generate the commitment
            let commitment: String = get_file_commitment_and_selected_row(
                json_contents.row_titles.to_owned(),
                json_contents.row_contents.to_owned(),
                json_contents.row_selectors.to_owned(),
            );
            println!("{}: {}", "Commitment".green().bold(), commitment.green());
        }
        Some(("gen-proof", matches)) => {
            println!("{}", "Generating proof...".cyan().bold());
            /// Get the input file path
            let input_file = matches.get_one::<String>("input-file").unwrap();
            /// Parse the contents of the input file
            let contents = fs::read_to_string(input_file).unwrap_or_else(|_| {
                panic!(
                    "{}",
                    "Something went wrong reading the file".red().to_string()
                )
            });
            let json_contents =
                serde_json::from_str::<GenerateCommitmentAndProofRequest>(&contents)
                    .unwrap_or_else(|_| { panic!("{}", "Failed to deserialize JSON file.\nRefer to the sample 'gen-proof.json' as reference".red().to_string()) });
            println!("{}: {}", "Input file path".blue().bold(), input_file.blue());

            /// Calling the function to generate the proof
            let proof = generate_proof(
                json_contents.row_titles.to_owned(),
                json_contents.row_contents.to_owned(),
                json_contents.row_selectors.to_owned(),
            );
            // let proof_string = String::from_utf8(proof).unwrap();

            println!("{}: {:?}", "Proof".green().bold(), proof);
        }
        Some(("verify-proof", matches)) => {
            println!("{}", "Verifying proof...".cyan().bold());
            /// Get the input file path
            let input_file = matches.get_one::<String>("input-file").unwrap();
            /// Parse the contents of the input file
            let contents = fs::read_to_string(input_file).unwrap_or_else(|_| {
                panic!(
                    "{}",
                    "Something went wrong reading the file".red().to_string()
                )
            });
            let json_contents =
                serde_json::from_str::<ProofVerificationRequest>(&contents)
                    .unwrap_or_else(|_| { panic!("{}", "Failed to deserialize JSON file.\nRefer to the sample 'gen-proof.json' as reference".red().to_string()) });
            println!("{}: {}", "Input file path".blue().bold(), input_file.blue());

            /// Calling the function to verify the proof
             let row_accumulator = get_selected_row(
                json_contents.row_title.to_owned(), 
                json_contents.row_content
            );
            let is_valid = verify_correct_selector(
                json_contents.commitment.to_owned(),
                row_accumulator,
                json_contents.proof
            );
            match is_valid {
                true => println!("{}: {}", "Proof verification".green().bold(), "true".green()),
                false => println!("{}: {}", "Proof verification".green().bold(), "false".red()),
            }
        }
        None => {
            println!("No subcommand was used");
            process::exit(1);
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
