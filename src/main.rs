use std::time::Duration;

// main.rs
use actix_web::{get, patch, post, web, App, HttpResponse, HttpServer, Responder};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use redis::AsyncCommands;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;

#[derive(Serialize, Deserialize)]
struct InferenceRequest {
    data: String,
    model: String,
}

#[derive(Serialize, Deserialize)]
struct InferenceResponse {
    uuid: String,
    data: String,
    model: String,
}

#[post("/inference")]
async fn post_inference(
    req_body: web::Json<InferenceRequest>,
    redis: web::Data<redis::Client>,
    kafka_producer: web::Data<FutureProducer>,
) -> impl Responder {
    let uuid = Uuid::new_v4().to_string();
    let inference = InferenceResponse {
        uuid: uuid.clone(),
        data: req_body.data.clone(),
        model: req_body.model.clone(),
    };

    let mut con = redis.get_multiplexed_async_connection().await.unwrap();
    let _: () = con.set(&uuid, serde_json::to_string(&inference).unwrap()).await.unwrap();

    let payload = serde_json::to_string(&inference).unwrap();
    let record = FutureRecord::to(&req_body.model)
        .payload(&payload)
        .key(&uuid);

    kafka_producer.send(record, Timeout::After(Duration::from_secs(5))).await.unwrap();

    HttpResponse::Ok().json(inference)
}

#[get("/inference/{uuid}")]
async fn get_inference(
    path: web::Path<String>,
    redis: web::Data<redis::Client>,
) -> impl Responder {
    let uuid = path.into_inner();
    let mut con = redis.get_multiplexed_async_connection().await.unwrap();
    let result: String = con.get(&uuid).await.unwrap();

    HttpResponse::Ok().body(result)
}

#[patch("/inference/{uuid}")]
async fn patch_inference(
    path: web::Path<String>,
    req_body: web::Json<InferenceRequest>,
    redis: web::Data<redis::Client>,
) -> impl Responder {
    let uuid = path.into_inner();
    let inference = InferenceResponse {
        uuid: uuid.clone(),
        data: req_body.data.clone(),
        model: req_body.model.clone(),
    };

    let mut con = redis.get_multiplexed_async_connection().await.unwrap();
    let _: () = con.set(&uuid, serde_json::to_string(&inference).unwrap()).await.unwrap();

    HttpResponse::Ok().json(inference)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let kafka_producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(kafka_producer.clone()))
            .service(post_inference)
            .service(get_inference)
            .service(patch_inference)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}