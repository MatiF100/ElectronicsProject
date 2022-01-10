use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

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