use actix_multipart::{
    form::{
        bytes::Bytes,
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use ifdohtem::xml_parser::*;
use ifdohtem::*;
use std::io::Read;
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

    let new_path = format!(
        "{}/tmp/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        uuid::Uuid::new_v4(),
    );
    info!("saving file to {}", new_path);
    form.file.file.persist(new_path.clone()).unwrap();
    info!("saved file to {}", new_path);

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

    // parse xml
    let a = parse_xml(&buf).unwrap();

    match payouts_call(a.row).await {
        Ok(Reports(a, b, c)) => {
            let id = uuid::Uuid::new_v4();
            let a_path = format!(
                "{}/tmp/{}_a.csv",
                std::env::current_dir().unwrap().to_str().unwrap(),
                id,
            );

            let b_path = format!(
                "{}/tmp/{}_b.csv",
                std::env::current_dir().unwrap().to_str().unwrap(),
                id,
            );

            let c_path = format!(
                "{}/tmp/{}_c.csv",
                std::env::current_dir().unwrap().to_str().unwrap(),
                id,
            );

            save_btreemap_to_csv(&a_path, &a).unwrap();
            save_btreemap_to_csv(&b_path, &b).unwrap();
            save_vec_to_csv(&c_path, &c).unwrap();

            HttpResponse::Ok().content_type("text/html").body(format!(
                r#"
        <html>
            <body>
                <button onclick="window.location.href='/download/{id}_a.csv';">Download Report1</button>
                <button onclick="window.location.href='/download/{id}_b.csv';">Download Report2</button>
                <button onclick="window.location.href='/download/{id}_c.csv';">Download Report3</button>
            </body>
        </html>
        "#,
            ))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/payouts/cancel_payment")]
async fn cancel_payment(form: web::Form<ConfirmForm>) -> impl Responder {
    let tmpfile_path = &form.tmpfile_path;
    std::fs::remove_file(&tmpfile_path).unwrap();
    info!("deleted file to {}", tmpfile_path);
    HttpResponse::Ok().body("Payment cancelled")
}

#[get("/download/{filename}")]
async fn download(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let file_path = format!("/tmp/{}", filename);

    match std::fs::read(&file_path) {
        Ok(data) => HttpResponse::Ok()
            .content_type("text/csv")
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            ))
            .body(data),
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
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
            .service(download)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
