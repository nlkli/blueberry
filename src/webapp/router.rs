use crate::error::Result;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{self, Bytes},
};
use std::path::Path;

type ResponseT = Response<BoxBody<Bytes, hyper::Error>>;

pub async fn handler(req: Request<body::Incoming>) -> Result<ResponseT> {
    let uri = req.uri();

    match (req.method(), uri.path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::builder()
            .header("Content-Type", TEXT_HTML_UTF_8)
            .body(empty())
            .unwrap()),
        (&Method::GET, "/api/templates") => match template_list().await {
            Ok(template_list) => Ok(Response::builder()
                .header("Content-Type", APPLICATION_JSON)
                .body(full(serde_json::to_vec(&template_list).unwrap()))
                .unwrap()),
            Err(_) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(empty())
                .unwrap()),
        },
        (&Method::GET, p) if p.starts_with("/api/read/template/") => {
            let path =
                Path::new(TEMPLATES_PATH_DIR).join(p.trim_start_matches("/api/read/template/"));
            match tokio::fs::read(&path).await {
                Ok(template) => Ok(Response::builder()
                    .header("Content-Type", TEXT_PLAIN_UTF_8)
                    .body(full(template))
                    .unwrap()),
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap()),
            }
        }
        (&Method::POST, p) if p.starts_with("/api/write/template/") => {
            let path =
                Path::new(TEMPLATES_PATH_DIR).join(p.trim_start_matches("/api/write/template/"));
            let content = req.into_body().collect().await.unwrap().to_bytes();
            match tokio::fs::write(&path, content).await {
                Ok(_) => Ok(Response::builder().body(empty()).unwrap()),
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap()),
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", TEXT_HTML_UTF_8)
            .body(full(NOT_FOUND_PAGE))
            .unwrap()),
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn template_list() -> Result<Vec<String>> {
    let mut res = Vec::new();

    if !Path::new(TEMPLATES_PATH_DIR).exists() {
        tokio::fs::create_dir(TEMPLATES_PATH_DIR).await?;
        return Ok(res);
    }

    let mut read_dir = tokio::fs::read_dir(TEMPLATES_PATH_DIR).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if let Some(path) = entry.path().into_iter().last().and_then(|v| v.to_str()) {
            res.push(path.to_owned());
        }
    }

    Ok(res)
}

const TEMPLATES_PATH_DIR: &str = "templates";

const TEXT_HTML_UTF_8: &[u8] = b"text/html; charset=utf-8";

const APPLICATION_JSON: &[u8] = b"application/json";

const TEXT_PLAIN_UTF_8: &[u8] = b"text/plain; charset=utf-8";

const NOT_FOUND_PAGE: &str = r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8">
  <title>404 Not Found</title>
</head>
<body style="color:#fff;background:#000;text-align:center;">
  <h1>404 Not Found</h1>
  <p>Страница не найдена</p>
  <a href="/" style="color:aqua;" >На главную</a>
</body>
</html>
"#;

#[tokio::test]
async fn templates_list_test() {
    println!("{:?}", template_list().await.unwrap());
}
