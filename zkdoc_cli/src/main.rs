#![allow(unused)]

use clap::{Arg, ArgAction, ArgGroup, Command, Parser};
use colored::*;
use zkdoc_sdk::services::services::{
    generate_proof, get_file_commitment_and_selected_row, get_selected_row, verify_correct_selector,
};
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::{fs, fs::File, process, str, str::FromStr};
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

fn save_to_file(filename: &str, data: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn main() {
    let cmd = clap::Command::new("zkdoc-cli")
        .about("ZKDoc CLI")
        .version("0.1.0")
        .author("ZKDoc Team")
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
                .about("Verifies a proof against a given commitment")
                .arg(
                    Arg::new("input-file")
                        .short('i')
                        .long("input-file")
                        .required(true)
                        .help("Input file needs to be a valid JSON file"),
                ),
        );
    // Choose a spinner style
    let spinner_name = "Dots12".to_string();

    let matches = cmd.get_matches();
    /// Matches subcommand and runs the corresponding functions
    match matches.subcommand() {
        Some(("gen-commitment", matches)) => {
            println!("{}", "## Generate commitment ##".cyan().bold());
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
            println!("{}: {}", "Input file".blue().bold(), input_file.blue());
            // Start the spinner animation
            let mut sp = Spinner::with_timer(
                Spinners::from_str(&spinner_name).unwrap(),
                "ZK circuit is running...".into(),
            );
            /// Calling the function to generate the commitment
            let commitment: String = get_file_commitment_and_selected_row(
                json_contents.row_titles.to_owned(),
                json_contents.row_contents.to_owned(),
                json_contents.row_selectors.to_owned(),
            );
            sp.stop_with_newline();
            println!("{}: {}", "Commitment".green().bold(), commitment.green());
            /// Save value to file
            match save_to_file("commitment.txt", &commitment) {
                Ok(_) => println!(
                    "{}: {}",
                    "Output file".green().bold(),
                    "commitment.txt".green()
                ),
                Err(_) => println!(
                    "{}: {}",
                    "Failed to save commitment to file".red().bold(),
                    "commitment.txt".red()
                ),
            }
        }
        Some(("gen-proof", matches)) => {
            println!("{}", "## Generate proof ##".cyan().bold());
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
            println!("{}: {}", "Input file".blue().bold(), input_file.blue());
            // Start the spinner animation
            let mut sp = Spinner::with_timer(
                Spinners::from_str(&spinner_name).unwrap(),
                "ZK circuit is running...".into(),
            );
            /// Calling the function to generate the proof
            let proof = generate_proof(
                json_contents.row_titles.to_owned(),
                json_contents.row_contents.to_owned(),
                json_contents.row_selectors.to_owned(),
            );
            sp.stop_with_newline();
            /// Save value to file
            let proof_string = serde_json::to_string(&proof).unwrap();
            match save_to_file("proof.txt", &proof_string) {
                Ok(_) => println!(
                    "{}: {}",
                    "Output file".green().bold(),
                    "proof.txt".green()
                ),
                Err(_) => println!(
                    "{}: {}",
                    "Failed to save proof to file".red().bold(),
                    "proof.txt".red()
                ),
            }
        }
        Some(("verify-proof", matches)) => {
            println!("{}", "## Verify proof##".cyan().bold());
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
            println!("{}: {}", "Input file".blue().bold(), input_file.blue());
            // Start the spinner animation
            let mut sp = Spinner::with_timer(
                Spinners::from_str(&spinner_name).unwrap(),
                "ZK circuit is running...".into(),
            );
            /// Calling the function to verify the proof
            let row_accumulator = get_selected_row(
                json_contents.row_title.to_owned(),
                json_contents.row_content,
            );
            let is_valid = verify_correct_selector(
                json_contents.commitment.to_owned(),
                row_accumulator,
                json_contents.proof,
            );
            sp.stop_with_newline();
            match is_valid {
                true => println!(
                    "{}: {}",
                    "Proof verification result".green().bold(),
                    "true".green()
                ),
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
