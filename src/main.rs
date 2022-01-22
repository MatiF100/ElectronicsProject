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
    let test: db::ProjectInfo = db::ProjectInfo {
        id: 0,
        author: "Fesz".to_owned(),
        academic_year: 2021,
        category: String::from("arduino"),
        is_diploma: false,
        title: "Test".to_owned(),
        files_link: None,
        internal_filename: None,
    };

    let mut app = db::MyApp::new();
    app.init();
    //app.add_project(&test);
    //app.add_project(&test);
    //app.overrite_save_database();

    //let mut app2 = App::new();
    //app2.load_projects();
    //dbg!(&app2);
    //let mut file = std::fs::File::create("test.json").unwrap();
    //file.write_all(test.to_json().as_bytes()).unwrap();
    //let json = std::fs::read_to_string("test.json").unwrap();
    //let read = ProjectInfo::from_json(&json);
    //dbg!(&read);

    //println!("{:?}",std::fs::create_dir("./test"));

    let actix_db = web::Data::new(api::APIContainer {
        db: Mutex::new(app),
    });

    println!("Running...");

    //    let up_form = Form::new()
    //        .field("file", Field::file(|_, _, mut stream| async move {
    //            while let Some(res) = stream.next().await {
    //                res?;
    //            }
    //            Ok(()) as Result<_, FormError>
    //        }))
    //        .field("json", Field::text());

    HttpServer::new(move || {
        //let auth = HttpAuthentication::basic(api::basic_auth_validator);
        let auth = HttpAuthentication::basic(api::extract);
        App::new()
            .wrap(Cors::permissive())
            .wrap(auth)
            .app_data(actix_db.clone())
            .service(echo)
            .service(api::save_file)
            .route("/hey", web::get().to(manual_hello))
            .route("/API", web::get().to(send_db))
            .service(api::get_whole_db)
            .service(api::add_entry)
            .service(api::add_entry_multipart)
            .service(api::get_file_for_project)
            .service(api::get_categories)
            .service(api::add_category)
            .service(actix_files::Files::new("/", "./utils").index_file("index.html"))
    })
    .bind(("0.0.0.0", *port))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
