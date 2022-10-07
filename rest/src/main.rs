use actix_web::{
    get, post,
    web::{Json, Path},
    App, HttpResponse, HttpServer,
};
use ffly_rs::{FireflyResult, FireflyStream};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Status {
    status: String,
}

#[derive(Serialize)]
struct FullValueResponse {
    value: String,
    ttl: usize,
}

#[derive(Deserialize)]
struct FullValue {
    value: String,
    ttl: Option<usize>,
}

static FIREFLY_ADDR: &'static str = "127.0.0.1:46600";

async fn get_firefly() -> FireflyStream {
    return FireflyStream::connect(FIREFLY_ADDR).await.unwrap();
}

fn parse<T, U>(res: FireflyResult<T>, parser: fn(T) -> U) -> HttpResponse
where
    U: Serialize,
{
    if res.is_err() {
        return HttpResponse::NotFound().finish();
    }

    let value = res.unwrap();
    let parsed = parser(value);
    HttpResponse::Ok().json(Json(parsed))
}

#[get("/{key}")]
async fn get_all(key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.get(&key.to_string()).await;

    parse(res, |(value, ttl)| FullValueResponse {
        value,
        ttl: ttl.parse().unwrap(),
    })
}

#[post("/{key}")]
async fn create(data: Json<FullValue>, key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;

    let ttl = match data.ttl {
        Some(ttl) => ttl,
        None => 0,
    };

    let res = firefly.new_with_ttl(&key, &data.value, ttl).await;

    parse(res, |_| Status {
        status: "ok".to_string(),
    })
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_all).service(create))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
