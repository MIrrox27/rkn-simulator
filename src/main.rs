// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs

//use axum::{Router, routing::get, extract::Path};
//use tower_http::services::ServeDir;
use tokio::{net::TcpListener, net::TcpStream, io::AsyncBufRead, spawn};
use std::io;




#[tokio::main]
async fn main() {
    println!("Please enter proxy addres [localhost:8000]"); // Просим ввести адресс для прокси
    let mut addr = String::new();
    io::stdin()
        .read_line(&mut addr)
        .expect("Error, can't read your addres");

    if addr.is_empty(){addr = "localhost:8000".to_string()}

    let listner = TcpListener::bind(addr);
    loop {
        

    }
    
}
