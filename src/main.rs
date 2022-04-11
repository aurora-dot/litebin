#[macro_use]
extern crate rocket;

mod paste_id;

use std::env;

use paste_id::PasteId;

use rocket::data::{Data, ToByteUnit};
use rocket::http::{self, ContentType, Header};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::response::content;
use rocket::response::{Responder, Response};
use rocket::tokio::fs::File;
use rocket_basicauth::BasicAuth;

struct Authenticated {}

struct SiteURL(String);

#[derive(Debug)]
enum ApiTokenError {
    Missing,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SiteURL {
    type Error = ApiTokenError;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let site_url = request.headers().get_one("Host");
        match site_url {
            Some(site_url) => Outcome::Success(SiteURL(site_url.to_string())),
            None => Outcome::Failure((http::Status::Unauthorized, ApiTokenError::Missing)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = &'static str;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_secrets = req.rocket().state::<BasicAuth>().unwrap();

        let auth = req.guard::<BasicAuth>().await;

        let basic_auth_username = match auth {
            request::Outcome::Success(ref a) => a.username.as_str(),
            _ => "",
        };

        let basic_auth_password = match auth {
            request::Outcome::Success(ref a) => a.password.as_str(),
            _ => "",
        };

        if (basic_auth_username == auth_secrets.username)
            & (basic_auth_password == auth_secrets.password)
        {
            return request::Outcome::Success(Authenticated {});
        } else {
            return request::Outcome::Failure((
                http::Status::Unauthorized,
                "Auth check failed. Please perform HTTP basic auth with the correct password.",
            ));
        }
    }
}

#[catch(401)]
fn unauthorized_catcher<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    struct Resp {}
    impl<'r, 'o: 'r> Responder<'r, 'o> for Resp {
        fn respond_to(
            self,
            _request: &Request,
        ) -> Result<rocket::Response<'o>, rocket::http::Status> {
            let mut res = Response::build();
            res.header(Header::new("WWW-Authenticate", "Basic"));
            res.status(http::Status::Unauthorized);
            Ok(res.finalize())
        }
    }
    Resp {}
}

#[get("/")]
fn index() -> &'static str {
    "litebin."
}

#[get("/test_auth")]
fn hello(_auth: Authenticated) -> &'static str {
    "Test Authentication."
}

#[get("/<id>")]
async fn retrieve(_auth: Authenticated, id: &str) -> content::Custom<Option<File>> {
    let split: Vec<&str> = id.split('.').collect();
    let id: &str = split[0];

    let content_type = if split.len() > 1 {
        let ext_string = split[1].to_string();
        let ext_upper = ext_string.to_uppercase();
        let ext: &str = ext_upper.as_str();

        match ext {
            "PNG" => ContentType::PNG,
            "JPG" | "JPEG" => ContentType::JPEG,
            _ => ContentType::Any,
        }
    } else {
        ContentType::Any
    };

    let filename = format!(
        "{}/{}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/", "upload"),
        id
    );
    content::Custom(content_type, File::open(&filename).await.ok())
}

#[post("/upload", data = "<paste>")]
async fn upload(
    _auth: Authenticated,
    paste: Data<'_>,
    site_url: SiteURL,
) -> std::io::Result<String> {
    let id = PasteId::new(8);
    let mb_limit: i16 = 512;
    paste
        .open(mb_limit.mebibytes())
        .into_file(id.file_path())
        .await?;

    Ok(format!("{}/{}\n", site_url.0, id.get_paste_id()))
}

#[launch]
fn rocket() -> _ {
    let username = match env::var("LITEBIN_USERNAME") {
        Ok(v) => v,
        Err(_e) => "hello".to_string(),
    };

    let password = match env::var("LITEBIN_PASSWORD") {
        Ok(v) => v,
        Err(_e) => "world".to_string(),
    };

    rocket::build()
        .mount("/", routes![hello, index, upload, retrieve])
        .register("/", catchers![unauthorized_catcher,])
        .manage(BasicAuth { username, password })
}
