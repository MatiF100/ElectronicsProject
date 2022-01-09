use std::path::Path;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

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
pub struct ProjectInfo{
    pub id: usize,
    pub title: String,
    pub author: String,
    pub date: i32,
    pub academic_year: i32,
    pub is_diploma: bool,
    pub category: String,
    pub files_names: String,
}

impl ProjectInfo{
    pub fn to_json(&self) -> String{
        String::from(serde_json::to_string(&self).unwrap())
    }

    pub fn from_json(json: &str) ->Self{
        serde_json::from_str(json).unwrap()
    }

    pub fn create_file_entry(){

    }
}


#[derive(Debug)]
pub struct MyApp<'a>{
    projects: Vec<ProjectInfo>,
    db_path: &'a Path
}

impl<'a> MyApp<'a>{
    pub fn new() ->Self{
        Self{
            projects: Vec::new(),
            db_path: Path::new("./database")
        }
    }

    pub fn init(&mut self){
        self.prepare_database();
    }

    pub fn prepare_database(&self){
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

    pub fn load_projects(&mut self){
        let db = std::fs::read_to_string("./database/db.json").unwrap();
        self.projects = serde_json::from_str(&db).unwrap();

    }

    pub fn add_project(&mut self, project: &ProjectInfo){
        self.projects.push(project.clone());
    }

    pub fn save_database(&self){
        let x = serde_json::to_string(&self.projects).unwrap();
        let mut db = std::fs::File::create(self.db_path.join(Path::new("db.json"))).unwrap();
        db.write_all(x.as_bytes()).unwrap();
    }

    pub fn get_database_as_string(&self) -> String{
        serde_json::to_string(&self.projects).unwrap()
    }

}