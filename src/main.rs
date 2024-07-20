use std::io::Read;

use actix_multipart::{
    form::{
        bytes::Bytes,
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use ifdohtem::yml_parser::*;
use tracing::{debug, info};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    file: TempFile,
}

#[post("/payouts")]
async fn payouts(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    let mut buf = String::new();
    form.file.file.into_file().read_to_string(&mut buf).unwrap();
    let a = parse_xml(&buf).unwrap();
    HttpResponse::Ok().body(a.to_string().unwrap())
}

#[get("/payouts")]
async fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // tracing
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    HttpServer::new(move || App::new().service(payouts).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
