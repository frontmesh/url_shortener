mod error;
mod utils;

use worker::{
    console_error, console_log, event, Date, Env, Request, Response,
    Result as WorkerResult, Router,
};
use rand::{ Rng };
use rand::distributions::Alphanumeric;
use serde::Deserialize;
use serde_json::json;
use crate::error::Result;


#[derive(Deserialize)]
struct ShortUrlRequest {
    url: String,
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

async fn handle_short_url_request(mut req: Request) -> Result<Response> {
    let ShortUrlRequest { url } = req.json().await.map_err(|e| {
        console_log!("Error decoding JSON: {:?}", e);
        error::Error::InternalError("Invalid JSON in the request".to_string())
    })?;

    let shortlink: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    console_log!("Received URL: {}, Received in URL: {}", url, req.url()?.host_str().unwrap());

    let short_url = format!("https://{}{}/{}", req.url()?.host_str().unwrap(), "/", shortlink);

    let resp = Response::from_json(&json!({
        "short_url": short_url
    }))?;

    Ok(resp)
}

fn result_to_response(result: Result<Response>) -> Response {
    result.unwrap_or_else(|e| {
        console_error!("error: {:?}", e);
        Response::from(e)
    })
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: worker::Context) -> WorkerResult<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    return router
        .get("/", |_, _| { Response::ok("Hello from workers!!") })
        .get("/:short_link", |_,_| {
            Response::ok("")
        })
        .post_async("/get_short_url",  |req, ctx| async move {
            let result = handle_short_url_request(req).await;
            Ok(result_to_response(result))
        })
        .get("/worker-version", |_, ctx|{
            Response::ok("0.0.18".to_string())
        })
        .run(req, env).await
}
