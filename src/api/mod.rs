use actix_web::dev::ServiceRequest;
use actix_web::{delete, get, post, web, Error, HttpResponse, Responder};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_grants::proc_macro::has_permissions;
use actix_web_httpauth::extractors::basic::BasicAuth;
use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use std::io::Write;
use std::sync::Mutex;

use crate::db::{self, ProjectInfo};

pub struct APIContainer<'a> {
    pub db: Mutex<db::MyApp<'a>>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthData {
    login: String,
    password: String,
}

#[post("/reauth")]
#[has_permissions("AUTH_ADMIN")]
pub async fn change_creds(item: web::Json<AuthData>) -> Result<HttpResponse, Error> {
    let creds = item.into_inner();
    let data = serde_json::to_string(&creds)?;
    let bytes = data.as_bytes();
    let mut file = std::fs::File::create("auth.json")?;
    file.write_all(bytes)?;

    Ok(HttpResponse::Accepted().finish())
}

#[get("/check")]
#[has_permissions("AUTH_ADMIN")]
pub async fn verify() -> impl Responder {
    Result::<_, actix_web::Error>::Ok("")
}

#[delete("/delete/{id}")]
#[has_permissions("AUTH_ADMIN")]
pub async fn delete_entry(
    database: web::Data<APIContainer<'_>>,
    res: web::Path<std::path::PathBuf>,
) -> impl Responder {
    let mut handle = database.db.lock().unwrap();
    let id = res.into_inner();
    let id = id
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    if let Some(project) = handle.get_project_by_id(id) {
        if let Some(_) = project.internal_filename {
            if let Err(_) =
                std::fs::remove_dir_all(handle.db_path.join("files").join(format!("{}", id)))
            {
                println!("File was already removed! Check for unauthorized database access");
            }
        }
    } else {
        return Err(actix_web::Error::from(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));
    }
    handle.remove_by_id(id);

    handle.overrite_save_database();

    Result::<_, actix_web::Error>::Ok("")
}

#[post("/upload/{filename}")]
#[has_permissions("AUTH_ADMIN")]
pub async fn save_file(
    filename: web::Path<std::path::PathBuf>,
    payload: web::Payload,
    database: web::Data<APIContainer<'_>>,
) -> impl Responder {
    let mut handle = database.db.lock().unwrap();

    let filename = filename.into_inner();
    let fileparts = filename.to_str().unwrap().split_once("%2F").unwrap();
    let mut payload = payload.into_inner();

    let dirpath = handle.db_path.join("files").join(fileparts.0);
    let filepath = dirpath.join(fileparts.1);

    match std::fs::create_dir(&dirpath) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => return Result::<_, actix_web::Error>::Err(actix_web::Error::from(e)),
        },
    }

    let mut file = std::fs::File::create(&filepath)?;
    while let Some(chunk) = payload.next().await {
        file.write(chunk?.as_ref())?;
    }

    let id = dirpath
        .file_name()
        .ok_or(actix_web::Error::from(
            actix_web::error::InternalError::new(
                "Invalid filename",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ))?
        .to_str()
        .ok_or(actix_web::Error::from(
            actix_web::error::InternalError::new(
                "Invalid filename",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ))?
        .parse::<i32>()
        .map_err(|_| actix_web::Error::from(actix_web::error::ParseError::Method))?;
    handle.attatch_file(id as usize, None);

    Result::<_, actix_web::Error>::Ok("")
}

#[get("/file/{id}")]
pub async fn get_file_for_project(
    res: web::Path<std::path::PathBuf>,
    database: web::Data<APIContainer<'_>>,
) -> Option<HttpResponse> {
    //TODO: Send file with correct filename
    let handle = database.db.lock().unwrap();

    let res = res.into_inner();
    let id = res.file_name()?.to_str()?.parse::<usize>().unwrap();
    let tmp = handle.get_file_path(id)?;
    let filepath = *tmp;
    let file = match std::fs::read(&filepath) {
        Ok(f) => f,
        Err(e) => return Some(HttpResponse::from_error(actix_web::Error::from(e))),
    };

    let mut resp = HttpResponse::Ok();
    let mime_guess = mime_guess::from_path(&filepath);
    if let Some(mime_guess) = mime_guess.first() {
        resp.content_type(mime_guess.as_ref());
    }
    let filename = format!("filename=\"{}\"", filepath.file_name()?.to_str()?);
    resp.header("Content-Disposition", filename);
    Some(resp.body(file))
}

#[get("/projects")]
pub async fn get_whole_db(database: web::Data<APIContainer<'_>>) -> impl Responder {
    HttpResponse::Ok().json(database.db.lock().unwrap().get_database_as_ref())
}

#[get("/categories")]
pub async fn get_categories(database: web::Data<APIContainer<'_>>) -> impl Responder {
    let cats = database.db.lock().unwrap();
    let mut resp = String::new();
    for cat in cats.get_categories().iter() {
        resp += cat;
    }
    //serde_json::to_string(cats.get_categories())
    HttpResponse::Ok().json(cats.get_categories())
}

#[post("/add_category/{cat}")]
#[has_permissions("AUTH_ADMIN")]
pub async fn add_category(
    database: web::Data<APIContainer<'static>>,
    res: web::Path<std::path::PathBuf>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || add_single_category(database, res))
        .await
        .map(|user| HttpResponse::Created().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

fn add_single_category(
    database: web::Data<APIContainer>,
    res: web::Path<std::path::PathBuf>,
) -> Result<()> {
    let mut handle = database.db.lock().unwrap();
    let category = res.into_inner();
    let category = category.to_str().unwrap_or("default");

    handle.add_category(category);
    handle.overrite_save_database();
    Ok(())
}

#[post("/add_mp")]
#[has_permissions("AUTH_ADMIN")]
pub async fn add_entry_multipart(
    database: web::Data<APIContainer<'static>>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, Error> {
    let mut filepath_copy = String::from("");
    let mut file_field: Vec<u8> = Vec::new();
    let mut data_field: Vec<u8> = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        match field.content_disposition() {
            Some(content_type) => match content_type.get_filename() {
                Some(filename) => {
                    filepath_copy = format!("{}", &filename);

                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        file_field.append(&mut data.to_vec());
                    }
                }
                None => {
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        data_field.append(&mut data.to_vec());
                    }
                }
            },
            None => (),
        }
    }

    let mut pr_info = ProjectInfo::from_str(&String::from_utf8(data_field).unwrap()).unwrap();
    let mut handle = database.db.lock().unwrap();
    if pr_info.files_link == None {
        pr_info.internal_filename = Some(filepath_copy.to_owned());
    } else {
        pr_info.internal_filename = None;
    }
    let id = handle.add_project(&pr_info).unwrap();

    if let Some(_) = pr_info.internal_filename {
        let dirpath = handle.db_path.join("files").join(format!("{}", id));
        match std::fs::create_dir(&dirpath) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => return Result::<_, actix_web::Error>::Err(actix_web::Error::from(e)),
            },
        }
        let filepath = dirpath.join(format!("{}", filepath_copy));

        let mut file = std::fs::File::create(&filepath)?;
        file.write(&file_field)?;
    }

    handle.overrite_save_database();

    Ok(HttpResponse::Ok().finish())
}

#[post("/add")]
#[has_permissions("AUTH_ADMIN")]
pub async fn add_entry(
    database: web::Data<APIContainer<'static>>,
    item: web::Json<db::ProjectInfo>,
) -> Result<HttpResponse, Error> {
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
        category: item.category.to_owned(),
        is_diploma: item.is_diploma,
        title: item.title.to_owned(),
        files_link: item.files_link.to_owned(),
        internal_filename: None,
    };
    handle.add_project(&new_user.clone())?;
    handle.overrite_save_database();
    Ok(())
}

async fn validate_creds(
    user: &str,
    pass: &Option<&std::borrow::Cow<'static, str>>,
) -> Result<bool, std::io::Error> {
    let auth = std::fs::read_to_string("auth.json")?;
    let auth: AuthData = serde_json::from_str(&auth).unwrap();
    match pass {
        Some(pass) => {
            if user.eq(&auth.login) && pass.trim().eq(&auth.password) {
                Ok(true)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid username and/or password",
                ))
            }
        }
        None => Ok(false),
    }
}

pub async fn extract(req: ServiceRequest, creds: BasicAuth) -> Result<ServiceRequest, Error> {
    let mut grants = vec!["AUTH_USER".to_string()];

    match validate_creds(creds.user_id(), &creds.password()).await {
        Ok(true) => grants.push("AUTH_ADMIN".to_string()),
        Ok(false) => (),
        Err(_) => (),
    }
    req.attach(grants);
    Ok(req)
}
