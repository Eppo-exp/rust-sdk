mod handlers;

use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    match (req.get_method(), req.get_path()) {
        (&Method::POST, "/assignments") => handlers::handle_assignments(req),
        (&Method::GET, "/health") => handlers::handle_health(req),
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND).with_body_text_plain("Not Found")),
    }
}
