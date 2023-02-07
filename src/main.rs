use serde::{Serialize, Deserialize};
use medi_0_core::services::services::{get_file_commitment_and_selected_row};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};

const ROW: usize = 10;

#[derive(Deserialize)]
struct GenerateCommitmentRequest {
    row_title_u32: [[u32; 8]; ROW],
    row_content_u32: [[u32; 8]; ROW],
    row_selector_u64: [u64; ROW],
}


#[derive(Serialize)]
struct GenerateCommitmentResponse {
    commitment: String,
}

#[post("/generate-commitment")]
async fn generate_commitment(req: web::Json<GenerateCommitmentRequest>) -> Result<impl Responder> {
    let commitment =  get_file_commitment_and_selected_row(req.row_title_u32, req.row_content_u32, req.row_selector_u64);

    Ok(web::Json(GenerateCommitmentResponse {
        commitment: commitment
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(generate_commitment)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}