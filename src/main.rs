#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod paste_id;

use std::env;
use std::path::{Path, PathBuf};

use paste_id::PasteId;

use rocket::data::{Data, ToByteUnit};
use rocket::fs::NamedFile;
use rocket::http::{self, ContentType, Header};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::response::content;
use rocket::response::status::NotFound;
use rocket::response::{Responder, Response};
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncWriteExt;
use rocket_basicauth::BasicAuth;

struct Authenticated {}

struct SiteURL(String);

#[derive(Debug)]
enum ApiTokenError {
    Missing,
}

// Request guard to get Host from headers
#[rocket::async_trait]
impl<'r> FromRequest<'r> for SiteURL {
    type Error = ApiTokenError;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let site_url = request.headers().get_one("Host");
        match site_url {
            Some(site_url) => Outcome::Success(SiteURL(site_url.to_string())),
            None => Outcome::Failure((http::Status::BadRequest, ApiTokenError::Missing)),
        }
    }
}

// Request guard for basic auth check
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

// Catches 401s and responds with a request to the client for basic auth login
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
fn test_auth(_auth: Authenticated) -> &'static str {
    "Test Authentication."
}

#[get("/<id..>")]
async fn retrieve(id: PathBuf, _auth: Authenticated) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("upload/").join(id);
    NamedFile::open(&path)
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[post("/upload", data = "<paste>")]
async fn upload(
    _auth: Authenticated,
    site_url: SiteURL,
    paste: Data<'_>,
) -> std::io::Result<String> {
    let mb_limit: i16 = 512;
    let raw_bytes = paste.open(mb_limit.mebibytes()).into_bytes().await?;
    let kind = infer::get(&raw_bytes).expect("file type is known");

    let id = PasteId::new(16, kind.extension().to_string());
    let mut file = File::create(id.file_path()).await?;
    file.write_all(&raw_bytes).await?;

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
        .mount("/", routes![test_auth, index, upload, retrieve])
        .register("/", catchers![unauthorized_catcher,])
        .manage(BasicAuth { username, password })
}
