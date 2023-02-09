use actix_cors::Cors;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder, Result};
use zkdoc_sdk::services::services::{
    generate_proof, get_file_commitment_and_selected_row, get_selected_row, verify_correct_selector,
};
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
    commitment: String,
}

#[derive(Serialize)]
struct ProofVerificationResponse {
    valid: bool,
}

/// This is for health check
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Rusty is fine!")
}

#[post("/generate-commitment")]
async fn generate_commitment_handler(
    req: web::Json<GenerateCommitmentAndProofRequest>,
) -> Result<impl Responder> {
    let commitment = get_file_commitment_and_selected_row(
        req.row_titles.to_owned(),
        req.row_contents.to_owned(),
        req.row_selectors.to_owned(),
    );

    Ok(web::Json(GenerateCommitmentResponse { commitment }))
}

#[post("/generate-proof")]
async fn generate_proof_handler(
    req: web::Json<GenerateCommitmentAndProofRequest>,
) -> Result<impl Responder> {
    // FYI this runs for 30+ seconds
    let proof = generate_proof(
        req.row_titles.to_owned(),
        req.row_contents.to_owned(),
        req.row_selectors.to_owned(),
    );

    Ok(web::Json(GenerateProofResponse { proof }))
}

#[post("/verify-proof")]
async fn verify_proof_handler(req: web::Json<ProofVerificationRequest>) -> Result<impl Responder> {
    let row_accumulator = get_selected_row(req.row_title.to_owned(), req.row_content.to_owned());
    let is_valid = verify_correct_selector(
        req.commitment.to_owned(),
        row_accumulator,
        req.proof.to_owned(),
    );

    Ok(web::Json(ProofVerificationResponse { valid: is_valid }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running on 8080");
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("[{method} {uri} {status} {response_time}ms {response_length}b]\n{request}\n{response}"))
            .service(hello)
            .service(generate_commitment_handler)
            .service(generate_proof_handler)
            .service(verify_proof_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
