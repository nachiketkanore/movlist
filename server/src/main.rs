use actix_web::{
    cookie::Cookie, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow};
use uuid::Uuid;

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

async fn auth_token() -> String {
    return Uuid::new_v4().to_string();
}

#[derive(Serialize, Deserialize)]
struct LoginSuccessResponse {
    email: String,
}

#[get("/login")]
async fn login(login_form: web::Json<User>, app_state: web::Data<AppState>) -> impl Responder {
    let user_check = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email=? AND password=?",
        login_form.email,
        login_form.password
    )
    .fetch_one(&app_state.pool)
    .await;
    if let Ok(user) = user_check {
        let auth_token = auth_token().await;
        // storing the auth_token and corresponding user in database

        let created = sqlx::query!(
            "INSERT INTO sessions values ( ?, ? )",
            user.email,
            auth_token
        )
        .execute(&app_state.pool)
        .await;

        if let Err(_) = created {
            return HttpResponse::Unauthorized().json("Failed to persist user session data");
        }

        return HttpResponse::Ok()
            .cookie(
                Cookie::build("auth_token", &auth_token)
                    .secure(true)
                    .http_only(true)
                    .finish(),
            )
            .json(LoginSuccessResponse { email: user.email });
    } else {
        return HttpResponse::Unauthorized().json("Invalid credentials");
    }
}

// Returns the email of logged in user or invalid
// based on auth_token cookie
#[get("/whoami")]
async fn whoami(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    let check_cookie = req.cookie("auth_token");
    if let Some(cookie) = check_cookie {
        let cookie = cookie.value();
        let check = sqlx::query!(
            "SELECT email as user FROM sessions WHERE auth_token=?",
            cookie
        )
        .fetch_one(&app_state.pool)
        .await;

        if let Ok(result) = check {
            return HttpResponse::Ok().json(result.user);
        } else {
            return HttpResponse::Unauthorized().json("user not identified");
        }
    }
    return HttpResponse::Unauthorized().json("user not identified");
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
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS sessions ( email TEXT NOT NULL, auth_token TEXT NOT NULL )",
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
            .service(login)
            .service(whoami)
    })
    .bind(("127.0.0.1", 8080))?
    .workers(1)
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    "server is up and running.."
}
