// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs

//use axum::{Router, routing::get, extract::Path};
//use tower_http::services::ServeDir;
use tokio::{io::{ AsyncReadExt, AsyncWriteExt}, net::{TcpListener}};
use std::io;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" -- Please enter proxy addres [127.0.0.1:8000]>"); // Просим ввести адрес для прокси
    let mut addres = String::new();
    io::stdin().read_line(&mut addres)
        .expect("Error, can't read your addres");

    let addr = addres.trim();    
    let addres = if addr.is_empty() {"127.0.0.1:8000".to_string()} else {addr.to_string()};

    println!("Addres:{}", addres);

    let listner = TcpListener::bind(addres).await.unwrap();


    loop {
        let (mut socket, addr) =  listner.accept().await.unwrap();
        println!("New connect {}", addr);
        
        tokio::spawn( async move  {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return, 
                    Ok(n) => {if socket.write_all(&buf[..n]).await.is_err(){return ;}}
                    Err(_) => return,
                }
            }
        });
    } 
}