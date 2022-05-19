
use super::rocket;
use base64::encode;
use rocket::http::ContentType;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
extern crate base64;

#[test]
fn test_index() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), "litebin.");
}

#[test]
fn test_auth() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    // Send unauthorised request
    let init_response = client.get(uri!(super::test_auth)).dispatch();
    assert_eq!(init_response.status(), Status::Unauthorized);

    // Send authorised request
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
fn test_upload_and_retrieve() {
    let body_content = "hello world";
    let host = "test";

    // Send authorised request with text body content
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let authorisation = format!("Basic {}", encode("hello:world"));
    let response = client
        .post(uri!(super::upload))
        .header(Header::new("Host", host))
        .body(body_content)
        .header(Header::new("Authorization", authorisation.clone()))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Get returned end of url and host
    let mut returned_url = response.into_string().unwrap();
    returned_url.pop();
    let url_split: Vec<&str> = returned_url.as_str().split('/').collect();
    assert_eq!(url_split[0], host);

    // Get uploaded content
    let response = client
        .get(format!("/{}", url_split[1]))
        .header(Header::new("Authorization", authorisation))
        .dispatch();

    assert_eq!(
        response.headers().iter().next(),
        Some(ContentType::Plain.into())
    );
    assert_eq!(response.status().clone(), Status::Ok);
    assert_eq!(response.into_string().unwrap(), body_content);
}

#[test]
fn test_unauthorised() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");

    // No SiteURL / Host header
    let response = client
        .post(uri!(super::upload))
        .body("hello world")
        .header(Header::new(
            "Authorization",
            format!("Basic {}", encode("hello:world")),
        ))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    // Incorrect username and password
    let response = client
        .get(uri!(super::test_auth))
        .header(Header::new(
            "Authorization",
            format!("Basic {}", encode("test:test")),
        ))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}
