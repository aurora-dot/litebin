use super::rocket;
use base64::encode;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::response;
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
    let init_response = client.get(uri!(super::test_auth)).dispatch();
    assert_eq!(init_response.status(), Status::Unauthorized);
    let default_credentials_base64 = encode("hello:world");
    let authorisation = format!("Basic {}", default_credentials_base64);
    let response = client
        .get(uri!(super::test_auth))
        .header(Header::new("Authorization", authorisation))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "Test Authentication.");
}

#[test]
fn test_upload() {
    let body_content = "hello world";
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let authorisation = format!("Basic {}", encode("hello:world"));
    let request = client
        .post(uri!(super::upload))
        .header(Header::new("Host", "test"))
        .body(body_content)
        .header(Header::new("Authorization", authorisation.clone()));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);

    let mut returned_url = response.into_string().unwrap();
    returned_url.pop();
    let url_split: Vec<&str> = returned_url.as_str().split('/').collect();
    let url_end: &str = url_split[1];

    println!("{}", url_end);

    let final_response = client
        .get(format!("/{}", url_end))
        .header(Header::new("Authorization", authorisation))
        .dispatch();
    assert_eq!(final_response.into_string().unwrap(), body_content);
}
