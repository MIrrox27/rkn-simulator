// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs


//use axum::{Router, routing::get, extract::Path};
//use tower_http::services::ServeDir;
use tokio::net::{TcpListener, TcpStream};
use tokio::{io::{ AsyncReadExt, AsyncWriteExt}};
use std::{io};
use std::ffi::CString;
use std::os::raw::c_char;


extern "C" {
    //fn delete_from_blacklist(domain: *const std::os::raw::c_char);
    //fn add_to_blacklist(domain: *const std::os::raw::c_char);
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
    let addr_clone = addres.clone();

    loop {
        let (mut stream, _) =  listner.accept().await.unwrap();
        println!("New connect {}", addr);
        
        tokio::spawn(handle(stream, addr_clone)); 
    }
}

async fn handle (mut stream: TcpStream, addres:&str) {   
    let mut buf = [0; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    if n == 0 {return};

    let request = String::from_utf8_lossy(&buf[..n]);
    println!("Requets: {}", request);

    
    
    let response;

    let first_line = if let Some(first_line) = request.lines().next(){first_line}
    else {""};

    let response_body = process_domain(first_line).await;
    if first_line != "Addres: ".to_string() + addres{
       let response = response_body;
    }
    else {
        response = "Initial connect";
    }  
    stream.write_all(response.as_bytes()).await.unwrap();

}


async fn process_domain(first_line: &str) -> String {
    let sec_domain = if let Some(domain_) = first_line.split(' ').nth(1){
        domain_}
    else {
        "error"
    };


    let domain: &str =
    if let Some(domain_) = sec_domain.rsplit_once(':'){domain_.0}
    else {" "};

    let c_domain = CString::new(domain).unwrap();
    let c_ptr: *const c_char = c_domain.as_ptr();
    let result = unsafe {search_domen(c_ptr)};
    println!("Domain {} has status {}", domain, result);

    let responce;
    if result == 1 {
        responce = "HTTP/1.1 200 OK\r\n\r\nДля тебя сайт заблокирован";
    }
    else {
        responce = "HTTP/1.1 200 OK\r\n\r\nShalom from your mother!";
    }

    return responce.to_string();

}