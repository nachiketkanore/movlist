use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[derive(Debug, Deserialize, FromRow, Serialize)]
struct User {
    email: String,
    password: String,
}

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[get("/users")]
async fn users(app_state: web::Data<AppState>) -> impl Responder {
    let users: Vec<User> = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&app_state.pool)
        .await
        .unwrap();
    return HttpResponse::Ok().json(users);
}

#[post("/signup")]
async fn signup(user: web::Json<User>, app_state: web::Data<AppState>) -> impl Responder {
    let created = sqlx::query!(
        "INSERT INTO users values ( ?, ? )",
        user.email,
        user.password
    )
    .execute(&app_state.pool)
    .await;
    match created {
        Ok(_) => format!("Created user with email: {}", user.email),
        Err(_) => format!("Could not create user"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // connect to SQLite DB
    // let db_url = String::from("sqlite://movlist.db");
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env file");
    let pool = SqlitePool::connect(&db_url).await.unwrap();
    let app_state = AppState { pool };

    // Database initialization -----------------------------------------------------------------
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS users ( email TEXT PRIMARY KEY NOT NULL, password TEXT NOT NULL )",
    )
    .execute(&app_state.pool)
    .await
    .unwrap();
    // Database initialization -----------------------------------------------------------------

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(index)
            .service(greet)
            .service(signup)
            .service(users)
    })
    .bind(("127.0.0.1", 8080))?
    .workers(2)
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    "server is up and running.."
}
