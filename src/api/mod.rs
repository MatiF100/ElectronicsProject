use actix_web::{get, post, web, App, HttpResponse,Error, HttpServer, Responder};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use anyhow::Result;

use std::sync::Mutex;
use std::io::Write;

use crate::db;

pub struct APIContainer<'a>{
    pub db: Mutex<db::MyApp<'a>>
}
async fn save_file(mut payload: Multipart, path: &str) -> Option<bool>{

	while let Ok(Some(mut field)) = payload.try_next().await{
		let content_type = field.content_disposition().unwrap();
		println!("{:?}", content_type);
		let mut f = web::block(|| std::fs::File::create("test.tst")).await.unwrap();

		while let Some(chunk) = field.next().await{
			let data = chunk.unwrap();
			f = web::block(move || f.write_all(&data).map(|_| f)).await.unwrap();
		}
	}

	Some(true)
}

#[get("/file")]
async fn route_function_example(
    payload: Multipart
) -> Result<HttpResponse, ()> {
  
    let upload_status = save_file(payload, "/path/filename.jpg").await;

    match upload_status {
        Some(true) => {

            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body("update_succeeded"))
        }
        _ => Ok(HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("update_failed")),
    }
}

#[get("/projects")]
pub async fn get_whole_db(database: web::Data<APIContainer<'_>>) -> impl Responder{
    database.db.lock().unwrap().get_database_as_string()
}

#[post("/add")]
pub async fn add_entry(database: web::Data<APIContainer<'static>>, item: web::Json<db::ProjectInfo>) -> Result<HttpResponse, Error>{
    Ok(web::block(move || add_single_entry(database, item))
    .await
    .map(|user| HttpResponse::Created().json(user))
    .map_err(|_| HttpResponse::InternalServerError())?)
}

fn add_single_entry(
    database: web::Data<APIContainer>,
    item: web::Json<db::ProjectInfo>,
) -> Result<()> {
    let mut handle = database.db.lock().unwrap();
    
    let new_user = db::ProjectInfo {
        id: 0,
        author: item.author.clone(),
        academic_year: item.academic_year,
        date: item.date,
        category: item.category.to_owned(),
        is_diploma: item.is_diploma,
        title: item.title.to_owned(),
        files_names: item.files_names.to_owned()
    };
    handle.add_project(&new_user.clone())?;
    handle.overrite_save_database();
    Ok(())
}