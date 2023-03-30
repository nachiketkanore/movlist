use std::collections::{hash_map, HashMap};
use std::sync::Mutex;

use actix_web::dev::Payload;
use actix_web::error::ErrorBadRequest;
use actix_web::web::Data;
use actix_web::Error;
use actix_web::{
    cookie::Cookie, get, http::StatusCode, post, web, App, FromRequest, HttpRequest, HttpResponse,
    HttpServer, Responder, ResponseError,
};
use dotenv::dotenv;
use futures_util::future::{err, Future, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqlitePool, SqliteRow},
    FromRow,
};
use std::pin::Pin;
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

impl FromRequest for User {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    // Define the extractor method to retrieve the user from the auth_token cookie
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        dbg!("i was here...");
        println!("from_request called");
        dbg!(req);
        let app_state = req.app_data::<web::Data<AppState>>().unwrap().clone();
        let req = req.clone();
        Box::pin(async move {
            let auth_token = req
                .cookie("auth_token")
                .map(|cookie| cookie.value().clone().to_string());
            if auth_token.is_none() {
                // return futures_util::future::ready(Err(Error::from("nachiket")));
                return Err(actix_web::error::ErrorForbidden(format!(
                    "Authentication Cookie Not Found"
                )));
            }
            let auth_token = auth_token.unwrap();
            match get_user_from_token(&auth_token, &app_state).await {
                Ok(user) => Ok(user),
                Err(_) => Err(actix_web::error::ErrorForbidden(format!(
                    "Authentication Cookie Not Found"
                ))),
            }
        })
        /* let app_state = req
            .app_data::<Data<Mutex<AppState>>>()
            .expect("database connection unavailable")
            .unwrap();
        // Retrieve the auth_token cookie from the request
        let auth_token = req
            .cookie("auth_token")
            .map(|cookie| cookie.value().to_string());

        // If the auth_token cookie is not present, return an error
        if auth_token.is_none() {
            // return futures_util::future::ready(Err(Error::from("nachiket")));
            return err(ErrorBadRequest("unauthorized"));
        }
        let auth_token = auth_token.unwrap();

        // Use the auth_token and database connection to retrieve the user information
        match get_user_from_token(&auth_token, &app_state).await {
            Ok(user) => futures_util::future::ready(Ok(user)),
            Err(_) => {
                return err(ErrorBadRequest("unauthorized"));
            }
        } */
    }
}

async fn get_user_from_token(auth_token: &str, app_state: &AppState) -> Result<User, String> {
    let check = sqlx::query!(
        "SELECT email as user FROM sessions WHERE auth_token=?",
        auth_token
    )
    .fetch_one(&app_state.pool)
    .await;

    if let Ok(result) = check {
        return Ok(User {
            email: result.user,
            password: "ignore".to_string(),
        });
    }
    return Err("not found".to_string());
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

#[derive(Debug, Deserialize, FromRow, Serialize)]
struct List {
    name: String,
    description: String,
    movie_ids: Vec<u32>,
}

#[post("/test")]
async fn test(user: Option<User>, _app_state: web::Data<AppState>) -> impl Responder {
    if user.is_some() {
        return HttpResponse::Ok().json(user);
    }
    return HttpResponse::Unauthorized().finish();
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct ListItem {
    list_id: String,
    title: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, Clone)]
struct MyList {
    id: String,
    name: String,
    titles: Vec<String>,
}

// get all the lists for current user
#[get("my_lists")]
async fn get_lists(user: User, app_state: web::Data<AppState>) -> impl Responder {
    // TODO
    let list_movies: Vec<ListItem> = sqlx::query_as!(
        ListItem,
        r#"
SELECT
	lm.list_id , l.name , m.title
from
	list_movies lm,
	movies m,
	lists l
where
	lm.movie_id = m.id and l.id = lm.list_id
	and lm.email = ?
order by
    lm.list_id "#,
        user.email
    )
    .fetch_all(&app_state.pool)
    .await
    .unwrap();

    let mut my_lists: Vec<MyList> = Vec::new();
    let mut add = MyList::default();

    for item in list_movies {
        if add.titles.is_empty() {
            add.name = item.name;
            add.titles.push(item.title);
            add.id = item.list_id;
        } else if add.id == item.list_id {
            add.titles.push(item.title);
        } else {
            my_lists.push(add.clone());
            add.name = item.name;
            add.titles.clear();
            add.titles.push(item.title);
            add.id = item.list_id;
        }
    }
    if !add.titles.is_empty() {
        my_lists.push(add.clone());
    }

    return HttpResponse::Ok().json(my_lists);
}

// create a new list for current user with given movies
#[post("/create_list")]
async fn create_list(
    user: User,
    app_state: web::Data<AppState>,
    list: web::Json<List>,
) -> impl Responder {
    let result = sqlx::query!(
        "INSERT INTO lists (email, name, description) VALUES (?, ?, ?) RETURNING id",
        user.email,
        list.name,
        list.description
    )
    .fetch_one(&app_state.pool)
    .await
    .unwrap();

    let list_id = result.id;
    // dbg!(&list_id);

    let mut failed = false;
    for movie_id in &list.movie_ids {
        let result = sqlx::query!(
            " INSERT INTO list_movies (list_id, email, movie_id) VALUES ( ?, ?, ? )",
            list_id,
            user.email,
            movie_id
        )
        .execute(&app_state.pool)
        .await;
        // dbg!("added ", movie_id, &result);

        failed |= result.is_err();
    }

    match failed {
        false => format!("Created the list"),
        true => format!("Could not create the list"),
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
    initialization(&app_state).await;
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
            .service(create_list)
            .service(test)
            .service(get_lists)
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

async fn initialization(app_state: &AppState) {
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

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS lists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT
        )
        "#,
    )
    .execute(&app_state.pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS list_movies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_id TEXT NOT NULL,
            email TEXT NOT NULL,
            movie_id TEXT NOT NULL
        )
        "#,
    )
    .execute(&app_state.pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS movies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            release_year INTEGER,
            genre TEXT,
            image_url TEXT
        )
        "#,
    )
    .execute(&app_state.pool)
    .await
    .unwrap();

    // adding some random movies
    /* sqlx::query!(
        r#"
    INSERT INTO movies (title, description, release_year, genre, image_url)
    VALUES
        ('The Shawshank Redemption', 'Prison Drama', 1994, 'Drama', 'https://www.example.com/shawshank.jpg'),
        ('The Godfather', 'Mafia Epic', 1972, 'Crime', 'https://www.example.com/godfather.jpg'),
        ('The Dark Knight', 'Gotham Chaos', 2008, 'Action', 'https://www.example.com/darkknight.jpg'),
        ('Pulp Fiction', 'Violent Tales', 1994, 'Crime', 'https://www.example.com/pulpfiction.jpg'),
        ('The Lord of the Rings: The Return of the King', 'Epic Fantasy', 2003, 'Fantasy', 'https://www.example.com/returnoftheking.jpg'),
        ('Forrest Gump', 'Life Journey', 1994, 'Drama', 'https://www.example.com/forrestgump.jpg'),
        ('Inception', 'Mind Heist', 2010, 'Sci-Fi', 'https://www.example.com/inception.jpg'),
        ('The Matrix', 'Virtual Reality', 1999, 'Sci-Fi', 'https://www.example.com/matrix.jpg'),
        ('Goodfellas', 'Mob Life', 1990, 'Crime', 'https://www.example.com/goodfellas.jpg'),
        ('Schindlers List', 'Holocaust Drama', 1993, 'Drama', 'https://www.example.com/schindlerslist.jpg');
    "#,
    ).execute(&app_state.pool)
        .await
    .unwrap(); */
}
