use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, middleware};
use core_lib::services::services::{get_file_commitment_and_selected_row, generate_proof, verify_correct_selector, get_selected_row};
use serde::{Deserialize, Serialize};

const ROW: usize = 10;

#[derive(Deserialize)]
struct GenerateCommitmentAndProofRequest {
    row_titles: [String; ROW],
    row_contents: [String; ROW],
    row_selectors: [u64; ROW],
}

#[derive(Serialize)]
struct GenerateCommitmentResponse {
    commitment: String,
}

#[derive(Serialize)]
struct GenerateProofResponse {
    proof: Vec<u8>,
}

#[derive(Deserialize)]
struct ProofVerificationRequest {
    proof: Vec<u8>,
    row_title: String,
    row_content: String,
    commitment: String
}

#[derive(Serialize)]
struct ProofVerificationResponse {
    valid: bool
}

/// This is for health check
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Rusty is fine!")
}

#[post("/generate-commitment")]
async fn generate_commitment_handler(req: web::Json<GenerateCommitmentAndProofRequest>) -> Result<impl Responder> {
    let commitment = get_file_commitment_and_selected_row(
        req.row_titles.to_owned(),
        req.row_contents.to_owned(),
        req.row_selectors.to_owned(),
    );

    Ok(web::Json(GenerateCommitmentResponse {
        commitment,
    }))
}

#[post("/generate-proof")]
async fn generate_proof_handler(req: web::Json<GenerateCommitmentAndProofRequest>) -> Result<impl Responder> {
    // FYI this runs for 30+ seconds
    let proof = generate_proof(
        req.row_titles.to_owned(),
        req.row_contents.to_owned(),
        req.row_selectors.to_owned(),
    );

    Ok(web::Json(GenerateProofResponse {
        proof,
    }))
}

#[post("/verify-proof")]
async fn verify_proof_handler(req: web::Json<ProofVerificationRequest>) -> Result<impl Responder> {
    let row_accumulator = get_selected_row(req.row_title.to_owned(), req.row_content.to_owned());
    let is_valid = verify_correct_selector(req.commitment.to_owned(), row_accumulator, req.proof.to_owned());

    Ok(web::Json(ProofVerificationResponse { valid: is_valid}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running on 8080");
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("[{method} {uri} {status} {response_time}ms {response_length}b]\n{request}\n{response}"))
            .service(hello)
            .service(generate_commitment_handler)
            .service(generate_proof_handler)
            .service(verify_proof_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
