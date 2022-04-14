use super::rocket;
use base64::encode;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
extern crate base64;

#[test]
fn index() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "litebin.");
}

#[test]
fn test_auth() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let init_response = client.get("/test_auth").dispatch();
    assert_eq!(init_response.status(), Status::Unauthorized);
    let default_credentials_base64 = encode("hello:world");
    let authorisation = format!("Basic {}", default_credentials_base64);
    let response = client
        .get("/test_auth")
        .header(Header::new("Authorization", authorisation))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Test Authentication.");
}
