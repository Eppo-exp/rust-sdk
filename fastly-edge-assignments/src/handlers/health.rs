use fastly::{http::StatusCode, Error, Request, Response};

pub fn handle_health(_req: Request) -> Result<Response, Error> {
    Ok(Response::from_status(StatusCode::OK).with_body_text_plain("OK"))
}
