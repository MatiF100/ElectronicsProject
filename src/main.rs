use std::sync::Mutex;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Date{
    year: i32,
    month: i32,
    day: i32
}

impl Date{
    fn from_values(day: i32, month: i32, year: i32) -> Self{
        Self{
            day,
            month,
            year
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectInfo{
    id: usize,
    title: String,
    author: String,
    date: i32,
    academic_year: i32,
    is_diploma: bool,
    category: String,
    files_names: String,
}

impl ProjectInfo{
    fn to_json(&self) -> String{
        String::from(serde_json::to_string(&self).unwrap())
    }

    fn from_json(json: &str) ->Self{
        serde_json::from_str(json).unwrap()
    }

    fn create_file_entry(){

    }
}


#[derive(Debug)]
struct MyApp<'a>{
    projects: Vec<ProjectInfo>,
    db_path: &'a Path
}

impl<'a> MyApp<'a>{
    fn new() ->Self{
        Self{
            projects: Vec::new(),
            db_path: Path::new("./database")
        }
    }

    fn init(&mut self){
        self.prepare_database();
    }

    fn prepare_database(&self){
        match std::fs::read_dir(self.db_path){
            Ok(db) => println!("Database found: {:?}", db),
            Err(e) => match e.kind(){
                std::io::ErrorKind::NotFound => match std::fs::create_dir(self.db_path){
                    Ok(_) => println!("Database created!"), 
                    Err(e) => panic!("Failed to create database! Error: {}", e)
                },
                _ => panic!("Failed to open database! Error: {}", e),
            }
        };
    }

    fn load_projects(&mut self){
        let db = std::fs::read_to_string("./database/db.json").unwrap();
        self.projects = serde_json::from_str(&db).unwrap();

    }

    fn add_project(&mut self, project: &ProjectInfo){
        self.projects.push(project.clone());
    }

    fn save_database(&self){
        let x = serde_json::to_string(&self.projects).unwrap();
        let mut db = std::fs::File::create(self.db_path.join(Path::new("db.json"))).unwrap();
        db.write_all(x.as_bytes()).unwrap();
    }

    fn get_database_as_string(&self) -> String{
        serde_json::to_string(&self.projects).unwrap()
    }

}

struct APIContainer<'a>{
    db: Mutex<MyApp<'a>>
}

async fn send_db(data: web::Data<APIContainer<'_>>) ->String{
    let db_str = data.db.lock().unwrap().get_database_as_string();
    db_str
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    //println!("Hello, world!");
    let test: ProjectInfo = ProjectInfo{
        id: 0,
        author: "Fesz".to_owned(),
        academic_year: 2021,
        date: 2019,
        category: String::from("arduino"),
        is_diploma: false,
        title: "Test".to_owned(),
        files_names: String::from("")
    };

    let mut app = MyApp::new();
    app.add_project(&test);
    app.add_project(&test);
    //app.save_database();

    //let mut app2 = App::new();
    //app2.load_projects();
    //dbg!(&app2);
    //let mut file = std::fs::File::create("test.json").unwrap();
    //file.write_all(test.to_json().as_bytes()).unwrap();
    //let json = std::fs::read_to_string("test.json").unwrap();
    //let read = ProjectInfo::from_json(&json);
    //dbg!(&read);

    //println!("{:?}",std::fs::create_dir("./test"));

    let actix_db = web::Data::new(APIContainer{
        db: Mutex::new(app)
    });
    
    println!("Dupa");
    HttpServer::new(move ||{
        App::new()
        .app_data(actix_db.clone())
        .service(hello)
        .service(echo)
        .route("/hey", web::get().to(manual_hello))
        .route("/API", web::get().to(send_db))
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