// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs

use axum::{Router, routing::get, response::Html};
use reqwest::header::AUTHORIZATION;
use std::net::SocketAddr;
use tower_http::services::ServeDir;



#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/api/*path", get(proxy_handler))
        .nest_service("/", ServeDir::new("./static"));
    let port = "127.0.0.1:1488";
    let listner = tokio::net::TcpListener::bind(port).await.unwrap();

    axum::serve(listner, app).await.unwrap();
}


async fn proxy_handler(
    axum::extract::Path(path): axum::extract::Path<String>
) -> impl axum::response::IntoResponse {
        // Строю URL
    let target_url = format!("https://jsonplaceholder.typicode.com/{}", path);

        // Отправляю запрос
    let response = reqwest::get(&target_url).await.unwrap();    

        // Читаю тело
    let body = response.text().await.unwrap();

        // возврат ответа
    (response.status(), body)

}
        
