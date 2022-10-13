use actix_web::{
    delete, get, post,
    web::{Json, Path},
    App, HttpResponse, HttpServer,
};
use ffly_rs::{FireflyResult, FireflyStream};
use serde::{Deserialize, Serialize};

static PORT: u16 = 46_601;

#[derive(Serialize)]
struct Status {
    status: String,
}

#[derive(Serialize)]
struct FullValueResponse {
    value: String,
    ttl: usize,
}

#[derive(Serialize)]
struct ValueResponse {
    value: String,
}

#[derive(Serialize)]
struct TtlResponse {
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

fn check_res<T>(res: FireflyResult<T>) -> HttpResponse {
    parse(res, |_| Status {
        status: "ok".to_string(),
    })
}

#[get("/{key}")]
async fn get_all(key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.get(&key.to_string()).await;

    parse(res, |(value, ttl)| FullValueResponse { value, ttl })
}

#[get("/{key}/ttl")]
async fn get_ttl(key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.get_ttl(&key.to_string()).await;

    parse(res, |ttl| TtlResponse { ttl })
}

#[get("/{key}/value")]
async fn get_value(key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.get_value(&key.to_string()).await;

    parse(res, |value| ValueResponse { value })
}

#[post("/{key}")]
async fn create(data: Json<FullValue>, key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;

    let ttl = match data.ttl {
        Some(ttl) => ttl,
        None => 0,
    };

    let res = firefly.new_with_ttl(&key, &data.value, ttl).await;
    check_res(res)
}

#[delete("/{key}")]
async fn delete(key: Path<String>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.drop(&key).await;
    check_res(res)
}

#[delete("/")]
async fn delete_by_value(data: Json<FullValue>) -> HttpResponse {
    let firefly = get_firefly().await;
    let res = firefly.drop_values(&data.value).await;
    check_res(res)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at port {}", PORT);
    HttpServer::new(|| {
        App::new()
            .service(get_all)
            .service(get_ttl)
            .service(get_value)
            .service(create)
            .service(delete)
            .service(delete_by_value)
    })
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}
