// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs

use axum::http::response;
//use axum::{Router, routing::get, extract::Path};
//use tower_http::services::ServeDir;
use tokio::net::{TcpListener, TcpStream};
use tokio::{io::{ AsyncReadExt, AsyncWriteExt}, stream};
use std::io;


extern "C" {
    fn delete_from_blacklist(domain: *const std::os::raw::c_char);
    fn add_to_blacklist(domain: *const std::os::raw::c_char);
    fn search_domen(domain: *const std::os::raw::c_char) -> std::os::raw::c_int;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" -- Please enter proxy addres [127.0.0.1:8000]>"); // Просим ввести адрес для прокси
    let mut addres = String::new();
    io::stdin().read_line(&mut addres)
        .expect("Error, can't read your addres");

    let addr = addres.trim();    
    let addres = if addr.is_empty() {"127.0.0.1:8000".to_string()} else {addr.to_string()};

    println!("Addres: http://{}", addres);

    let listner = TcpListener::bind(addres).await.unwrap();


    loop {
        let (mut stream, _) =  listner.accept().await.unwrap();
        println!("New connect {}", addr);
        
        tokio::spawn(handle(stream)); 
    }
}

async fn handle (mut stream: TcpStream) {   
    let mut buf = [0; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    if n == 0 {return};

    let request = String::from_utf8_lossy(&buf[..n]);
    println!("Requets: {}", request);

    let responce = "HTTP/1.1 200 OK\r\n\r\nShalom from your mother!";
    stream.write_all(responce.as_bytes()).await.unwrap();

}