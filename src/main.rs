use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use actix_web_httpauth::middleware::HttpAuthentication;
use futures::stream::StreamExt;

mod api;
mod db;


async fn send_db(data: web::Data<api::APIContainer<'_>>) -> String {
    let db_str = data.db.lock().unwrap().get_database_as_string();
    db_str
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //println!("Hello, world!");
    let port = std::env::var("ACTIX_PORT");
    dbg!(&port);
    let port = &port.unwrap_or("8080".to_owned()).parse::<u16>().unwrap();
    dbg!(&port);

    let mut app = db::MyApp::new();
    app.init();

    let actix_db = web::Data::new(api::APIContainer {
        db: Mutex::new(app),
    });

    println!("Running...");

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(api::extract);
        App::new()
            .wrap(Cors::permissive())
            //.wrap(auth)
            .app_data(actix_db.clone())
            .service(
                web::scope("/admin/")
                    .wrap(auth)
                    .service(api::save_file)
                    .service(api::add_entry)
                    .service(api::add_entry_multipart)
                    .service(api::add_category)
                    .service(api::delete_entry),
            )
            .service(api::get_whole_db)
            .service(api::get_file_for_project)
            .service(api::get_categories)
            .service(actix_files::Files::new("/", "./utils").index_file("index.html"))
    })
    .bind(("0.0.0.0", *port))?
    .run()
    .await
}
