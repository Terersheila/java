use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use mongodb::{Client, options::ClientOptions, Collection};
use mongodb::bson::{doc, Document, DateTime};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct PaymentRequest {
    item: String,
    amount: i32,
    phone: String,
}

#[derive(Debug, Serialize)]
struct PaymentResponse {
    message: String,
    status: String,
}

struct AppState {
    collection: Collection<Document>,
}

async fn init_db() -> mongodb::error::Result<Collection<Document>> {
    let mongo_uri = std::env::var("MONGO_URI").expect("MONGO_URI must be set");
    let client_options = ClientOptions::parse(&mongo_uri).await?;
    let client = Client::with_options(client_options)?;
    Ok(client.database("candylicious").collection("payments"))
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Backend server is running")
}

async fn pay(req: web::Json<PaymentRequest>, data: web::Data<Arc<AppState>>) -> Result<impl Responder> {
    let req_data = req.into_inner();

    let doc = doc! {
        "item_id": req_data.item,
        "amount": req_data.amount,
        "phone": req_data.phone,
        "status": "pending",
        "timestamp": DateTime::now(),
    };

    match data.collection.insert_one(doc, None).await {
        Ok(result) => {
            println!("Inserted payment with id: {:?}", result.inserted_id);
            Ok(HttpResponse::Ok().json(PaymentResponse {
                message: format!("Payment initiated for item {}", req_data.item),
                status: "success".to_string(),
            }))
        }
        Err(e) => {
            eprintln!("Failed to insert payment: {}", e);
            Ok(HttpResponse::InternalServerError().json(PaymentResponse {
                message: "Failed to process payment".to_string(),
                status: "error".to_string(),
            }))
        }
    }
}

async fn callback() -> impl Responder {
    HttpResponse::Ok().body("Callback received")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let collection = init_db().await.expect("Failed to initialize database");
    let app_state = Arc::new(AppState { collection });

    println!("Server running on http://127.0.0.1:5500");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(index))
            .route("/pay", web::post().to(pay))
            .route("/callback", web::post().to(callback))
    })
    .bind("127.0.0.1:5500")?
    .run()
    .await
}