use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use core_lib::services::services::get_file_commitment_and_selected_row;
use serde::{Deserialize, Serialize};

const ROW: usize = 10;

#[derive(Deserialize)]
struct GenerateCommitmentRequest {
    row_titles: [String; ROW],
    row_contents: [String; ROW],
    row_selectors: [u64; ROW],
}

#[derive(Serialize)]
struct GenerateCommitmentResponse {
    commitment: String,
}

#[post("/generate-commitment")]
async fn generate_commitment(req: web::Json<GenerateCommitmentRequest>) -> Result<impl Responder> {
    let commitment = get_file_commitment_and_selected_row(
        req.row_titles.to_owned(),
        req.row_contents.to_owned(),
        req.row_selectors.to_owned(),
    );

    Ok(web::Json(GenerateCommitmentResponse {
        commitment: commitment,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Running on 8080");
    HttpServer::new(|| App::new().service(generate_commitment))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
