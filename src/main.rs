// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs


use axum::http::{request, response};
//use axum::{Router, routing::get, extract::Path};
//use tower_http::services::ServeDir;
use tokio::net::{TcpListener, TcpStream};
use tokio::{io::{ AsyncReadExt, AsyncWriteExt}};
use std::intrinsics::exact_div;
use std::{io};
use std::ffi::CString;
use std::os::raw::c_char;
use tokio::io::AsyncBufReadExt;
use reqwest;

extern "C" {
    //fn delete_from_blacklist(domain: *const std::os::raw::c_char);
    fn append_to_blacklist(domain: *const std::os::raw::c_char);
    fn search_domen(domain: *const std::os::raw::c_char) -> std::os::raw::c_int;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    add_domain_to_blacklist("example.com"); // тестовый 

    println!("\n\n\n -- Please enter proxy addres [127.0.0.1:8000]>"); // Просим ввести адрес для прокси
    let mut addres = String::new();
    io::stdin().read_line(&mut addres)
        .expect("Error, can't read your addres");

    let addr = addres.trim();    
    let addres = if addr.is_empty() {"127.0.0.1:8000".to_string()} else {addr.to_string()};

    println!("Addres: http://{}", addres);

    
    //let addr_clone = addres.clone();

    tokio::spawn(async {
            let stdin = tokio::io::stdin();
            let mut reader = tokio::io::BufReader::new(stdin);
            let mut line = String::new();
            
            println!("Enter domain you want to block (or 'exit', to quit the program): ");
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {

                        let domain = line.trim();
                        if domain.is_empty() {continue;}
                        if domain == "exit" {break;}
                        add_domain_to_blacklist(domain);

                    }
                    Err(_) => break,
                }
            }
        }); 
    
    let listner = TcpListener::bind(addres).await?;
    loop {
        let (stream, _) =  listner.accept().await?;
        println!("\n\nNew connect {}", addr);
        
        tokio::spawn(handle(stream));
    }
}



async fn handle (mut stream: TcpStream) {   
    let mut buf = [0; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    if n == 0 { return; }

    let request = String::from_utf8_lossy(&buf[..n]);
    println!("Requets: {}", request);

    let first_line = request.lines().next().unwrap_or("");


    if first_line.starts_with("CONNECT") {
        let domain = extract_domain_connect(first_line);
        handle_connect(stream, domain, request).await;
    }
    else {
        let domain = extract_domain_http().await;
        handle_http(stream, domain, request).await;
    }


    let response = search_domain_rst(&(process_domain(first_line).await));
    stream.write_all(response.as_bytes()).await.unwrap();

}



async fn handle_http(mut client_stream: TcpStream, domain: String, request: String){  
    if is_domain_blocked(&domain) {
        let response = "HTTP/1.1 403 Forbidden\r\n\r\nBlocked";
        client_stream.write_all(response.as_bytes()).await.unwrap();
        return;
    }

    let first_line = request.lines().next().unwrap_or("");
    let path = exact_path(first_line);

}



async fn handle_connect(mut client_stream: TcpStream, domain: String, request: String){
        // Проверка домена 
    if is_domain_blocked(&domain) { // проверка домена 
        let response = "HTTP/1.1 403 Forbidden\r\n\r\nBlocked";
        client_stream.write_all(response.as_bytes()).await.unwrap();
        return;
    }

        // Подключение 
    let addr = format!("{}:433", domain);
    let mut server_stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => {
            let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
            client_stream.write_all(response.as_bytes()).await.unwrap();
            return ;

        }
    };


        // отправляем браузеру инфу, что все хорошо 
    let response = "HTTP/1.1 200 Connection established\r\n\r\n";
    client_stream.write_all(response.as_bytes()).await.unwrap();


        // Делаем туннель
    match tokio::io::copy_bidirectional(&mut client_stream, &mut server_stream).await {
        Ok(_) => println!("Tunnel closed for {}", domain),
        Err(e) => println!("Tunnel error: {}", e)
    }
}



fn extract_domain_connect(first_line: &str) -> String{
    let res = first_line.replacen("CONNECT", "", 1);
    let host_port = res.split_whitespace().next().unwrap_or("");
    let domain = host_port.split(':').next().unwrap_or("");
    return domain.to_string();


}


fn extract_path(first_line: &str) -> String {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 { return "/".to_string(); }
    
    let url = parts[1];
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let path = url.split('/').nth(1).unwrap_or("");
    format!("/{}", path)
}


async fn process_domain(first_line: &str) -> String {
    if first_line.starts_with("CONNECT"){
        let domain_and_http = first_line.replacen("CONNECT ", "", 1);
        let domain_and_http_vec: Vec<&str> = domain_and_http.split(":").collect();
        let domain = domain_and_http_vec[0];
        return domain.to_string();
    }
    else {  
        let url_and_metgod: Vec<&str> = first_line.split(" ").collect();
        if url_and_metgod.len() < 2 {
            return "error".to_string(); // или обработка ошибки
        }

        let url = url_and_metgod[1];
        
        let url = if url.starts_with("http://"){
            url.replacen("http://", "", 1).to_string()
        }
        else if url.starts_with("https://"){
            url.replacen("https://", "", 1).to_string()
        }
        else {url.to_string()};

        let url_and_port: Vec<&str> = url.split("/").collect();
        let domain = if url_and_port[0].contains(":") {
            let parts: Vec<&str> = (url_and_port[0].split(":")).collect();
            parts[0]
        } else { url_and_port[0] };

        return domain.to_string();
    }
     
}   


fn search_domain_rst(domain: &str) -> String{
    let c_domain = CString::new(domain).unwrap();
    let c_ptr: *const c_char = c_domain.as_ptr();
    let result = unsafe {search_domen(c_ptr)};
    
    let responce = 
    if result == 1 { "HTTP/1.1 200 OK\r\n\r\nHAHAHAHAHAH Website was blocked for yoy"}
    else { "HTTP/1.1 200 Connection established\r\n\r\n"};
    return responce.to_string();
}

fn add_domain_to_blacklist(domain: &str){
    let c_domain = CString::new(domain).unwrap();
    let c_ptr: *const c_char = c_domain.as_ptr();
    unsafe {append_to_blacklist(c_ptr)};
    println!("Domain {} added to blacklist", domain)

}



async fn extract_domain_http(){}

fn is_domain_blocked(domain: &str) -> bool{
    let c_domain = CString::new(domain).unwrap();
    let c_ptr: *const c_char = c_domain.as_ptr();
    let result = unsafe {search_domen(c_ptr)};
    return result == 1;
}