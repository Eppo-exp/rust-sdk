use fastly::http::StatusCode;
use fastly::{Error, Request, Response};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Create an HTTP OK response
    let response = Response::from_status(StatusCode::OK)
        .with_body_text_plain("Request processed successfully");

    Ok(response)
}
