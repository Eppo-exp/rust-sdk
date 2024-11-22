mod handlers;

use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
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
