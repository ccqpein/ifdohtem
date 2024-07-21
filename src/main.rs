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
use ifdohtem::*;
use tracing::{debug, info};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    file: TempFile,
}

#[derive(serde::Deserialize)]
struct ConfirmForm {
    tmpfile_path: String,
}

#[post("/payouts")]
async fn payouts(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    let mut buf = String::new();
    form.file.file.as_file().read_to_string(&mut buf).unwrap();
    let a = parse_xml(&buf).unwrap();
    //let csv_data = pre_payouts(&a.row).await;

    let mut table_html = String::new();
    table_html.push_str("<table border=\"1\">");
    table_html.push_str(
        "<tr><td>payer id</td><td>pay to amount</td><td>first name</td><td>last name</td></tr>",
    );
    for row in a.row {
        table_html.push_str("<tr>");

        table_html.push_str(&format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            row.payor.dunkin_id.clone(),
            row.amount.clone(),
            row.employee.first_name.clone(),
            row.employee.last_name.clone(),
        ));
        table_html.push_str("</tr>");
    }
    table_html.push_str("</table>");

    //let tmpfile_path = form.file.file.path().as_os_str().to_str().unwrap();
    let new_path = format!(
        "{:?}/tmp/{:?}",
        std::env::current_dir().unwrap(),
        uuid::Uuid::new_v4(),
    );
    form.file.file.persist(new_path.clone()).unwrap();
    debug!("save file to {}", new_path);

    HttpResponse::Ok().content_type("text/html").body(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Payouts Form</title>
        </head>
        <body>
           <h2>Your CSV data</h2>
            <form action="/confirm_payment" method="post">
               {table_html}
                <input type="hidden" name="tmpfile_path" value="{new_path}">
                <br>
                <button type="submit" formaction="/payouts/confirm_payment">Confirm</button>
               <button type="submit" formaction="/payouts/cancel_payment">Cancel</button>
            </form>
        </body>
        </html>"#
    ))
}

#[get("/payouts")]
async fn index() -> impl Responder {
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

#[post("/payouts/confirm_payment")]
async fn confim_payment(form: web::Form<ConfirmForm>) -> impl Responder {
    let tmpfile_path = &form.tmpfile_path;
    let mut buf = String::new();
    std::fs::File::open(&tmpfile_path)
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();

    HttpResponse::Ok().body("hello")
}

#[post("/payouts/cancel_payment")]
async fn cancel_payment(form: web::Form<ConfirmForm>) -> impl Responder {
    let tmpfile_path = &form.tmpfile_path;
    std::fs::remove_file(&tmpfile_path).unwrap();
    HttpResponse::Ok().body("Payment cancelled")
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

    HttpServer::new(move || {
        App::new()
            .service(payouts)
            .service(index)
            .service(confim_payment)
            .service(cancel_payment)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
