mod handlers;

use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};

#[cfg(test)]
const TEST_HOST: &str = "test-host";

fn main() -> Result<(), Error> {
    let ds_req = Request::from_client();
    let us_resp = handler(ds_req)?;
    us_resp.send_to_client();
    Ok(())
}

fn handler(req: Request) -> Result<Response, Error> {
    // Handle CORS preflight requests
    if req.get_method() == Method::OPTIONS {
        return Ok(Response::from_status(StatusCode::NO_CONTENT)
            .with_header("Access-Control-Allow-Origin", "*")
            .with_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .with_header("Access-Control-Allow-Headers", "Content-Type")
            .with_header("Access-Control-Max-Age", "86400"));
    }

    // Handle regular requests and add CORS headers to the response
    let response = match (req.get_method(), req.get_path()) {
        (&Method::POST, "/assignments") => handlers::handle_assignments(req),
        (&Method::GET, "/health") => handlers::handle_health(req),
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND).with_body_text_plain("Not Found")),
    }?;

    // Add CORS headers to all responses
    Ok(response
        .with_header("Access-Control-Allow-Origin", "*")
        .with_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS"))
}

#[test]
fn test_health() {
    let req = fastly::Request::get(&format!("https://{}/health", TEST_HOST));
    let resp = handler(req).expect("request succeeds");
    assert_eq!(resp.get_status(), StatusCode::OK);
    assert_eq!(resp.into_body_str(), "OK");
}

#[test]
fn test_cors_headers() {
    let req = Request::get(&format!("https://{}/health", TEST_HOST));
    let resp = handler(req).expect("request succeeds");

    assert_eq!(resp.get_header("Access-Control-Allow-Origin").unwrap(), "*");
    assert_eq!(
        resp.get_header("Access-Control-Allow-Methods").unwrap(),
        "GET, POST, OPTIONS"
    );
}

#[test]
fn test_options_request() {
    let req = Request::new(
        Method::OPTIONS,
        &format!("https://{}/assignments", TEST_HOST),
    );
    let resp = handler(req).expect("request succeeds");

    assert_eq!(resp.get_status(), StatusCode::NO_CONTENT);
    assert_eq!(resp.get_header("Access-Control-Allow-Origin").unwrap(), "*");
    assert_eq!(
        resp.get_header("Access-Control-Allow-Methods").unwrap(),
        "GET, POST, OPTIONS"
    );
    assert_eq!(
        resp.get_header("Access-Control-Allow-Headers").unwrap(),
        "Content-Type"
    );
    assert_eq!(resp.get_header("Access-Control-Max-Age").unwrap(), "86400");
}
