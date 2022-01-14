use std::sync::Mutex;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;

mod db;
mod api;


async fn send_db(data: web::Data<api::APIContainer<'_>>) ->String{
    let db_str = data.db.lock().unwrap().get_database_as_string();
    db_str
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    //println!("Hello, world!");
    let test: db::ProjectInfo = db::ProjectInfo{
        id: 0,
        author: "Fesz".to_owned(),
        academic_year: 2021,
        date: 2019,
        category: String::from("arduino"),
        is_diploma: false,
        title: "Test".to_owned(),
        files_names: String::from("")
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

    let actix_db = web::Data::new(api::APIContainer{
        db: Mutex::new(app)
    });
    
    println!("Dupa");
    HttpServer::new(move ||{
        App::new().wrap(
            Cors::default().allow_any_origin().allow_any_header().allow_any_method()
        )
        .app_data(actix_db.clone())
        .service(echo)
        .service(api::route_function_example)
        .route("/hey", web::get().to(manual_hello))
        .route("/API", web::get().to(send_db))
        .service(api::get_whole_db)
        .service(api::add_entry)
        .service(actix_files::Files::new("/", "./utils").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
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