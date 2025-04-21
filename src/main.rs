use rocket::{launch, routes};
use rocket_db_pools::{mongodb::Client, Database};
use mongodb::bson::oid::ObjectId;
use rocket::serde::{Deserialize, Serialize};
use mongodb::bson::doc;
use rocket::{
    get, http::Status, post, response::status,
    serde::json::Json,
};
use rocket_db_pools::Connection;
use serde_json::{json, Value};
use rocket::response::Redirect;
use rocket::fs::{FileServer, relative};
use rocket::fs::NamedFile;
use std::path::{PathBuf, Path};
 
#[derive(Database)]
#[database("db")]
pub struct MainDatabase(Client);

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Url {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub site: String,
    pub short_key: String,
}

#[get("/serve_home_page/<path..>")]
   pub async fn serve_home_page(path: PathBuf) -> Option<NamedFile> {
       let mut path = Path::new(relative!("static")).join(path);
       if path.is_dir() {
           path.push("index.html");
       }
       NamedFile::open(path).await.ok()
}

#[post("/", data = "<data>", format = "json")]
pub async fn create_url(
    db: Connection<MainDatabase>,
    data: Json<Url>,
) -> status::Custom<Json<Value>> {
    if let Ok(res) = db
        .database("UrlsDB")
        .collection::<Url>("UrlsCollection")
        .insert_one(data.into_inner(), None)
        .await
    {
        if let Some(id) = res.inserted_id.as_object_id() {
            return status::Custom(
                Status::Created,
                Json(
                    json!({"status": "success", "message": format!("Url ({}) created successfully", id.to_string())}),
                ),
            );
        }
    }

    status::Custom(
        Status::BadRequest,
        Json(json!({"status": "error", "message":"Url could not be created"})),
    )
}

#[get("/<short_key>", rank = 11)]
pub async fn get_url(db: Connection<MainDatabase>, short_key: &str) -> Redirect {	
	
    if let Ok(Some(url)) = db
        .database("UrlsDB")
        .collection::<Url>("UrlsCollection")
        .find_one(doc! {"short_key": short_key}, None)
        .await
		{
		return Redirect::to(url.site);
		};
	return Redirect::to("/");   
} 
 
#[launch]
fn rocket() -> _ {
    rocket::build().attach(MainDatabase::init())
		.mount("/", routes![serve_home_page])
		.mount("/", routes![get_url])
		.mount("/", routes![create_url])
		.mount("/", FileServer::from(relative!("static")))
}