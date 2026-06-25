// author https://github.com/MIrrox27/rkn-simulator
// src/main.rs

use tokio::net::{TcpListener, TcpStream};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}};
use std::{io};
use std::ffi::CString;
use std::os::raw::c_char;
use tokio::io::AsyncBufReadExt;

extern "C" {
    fn append_to_blacklist(domain: *const std::os::raw::c_char);
    fn search_domen(domain: *const std::os::raw::c_char) -> std::os::raw::c_int;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    add_domain_to_blacklist("example.com"); // тестовый 

    println!("\n\n\n -- Please enter proxy addres [127.0.0.1:8000]>");
    let mut addres = String::new();
    io::stdin().read_line(&mut addres)
        .expect("Error, can't read your addres");

    let addr = addres.trim();    
    let addres = if addr.is_empty() {"127.0.0.1:8000".to_string()} else {addr.to_string()};

    println!("Addres: http://{}", addres);

    // Задача для чтения команд из консоли
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
                    if domain.is_empty() { continue; }
                    if domain == "exit" { break; }
                    add_domain_to_blacklist(domain);
                }
                Err(_) => break,
            }
        }
    }); 
    
    let listener = TcpListener::bind(addres).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        println!("\n\nNew connection");
        tokio::spawn(handle(stream));
    }
}

async fn handle(mut stream: TcpStream) {   
    let mut buf = [0; 4096];
    let n = match stream.read(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            println!("Ошибка чтения: {}", e);
            return;
        }
    };
    if n == 0 { return; }

    let request = String::from_utf8_lossy(&buf[..n]).to_string(); // <-- преобразуем в String
    println!("Request: {}", request);

    let first_line = request.lines().next().unwrap_or("");

    if first_line.starts_with("CONNECT") {
        let domain = extract_domain_connect(first_line);
        handle_connect(stream, domain, request).await;
    } else {
        let domain = extract_domain_http(first_line); // <-- передаём first_line
        handle_http(stream, domain, request).await;
    }
}

async fn handle_http(mut client_stream: TcpStream, domain: String, request: String) {
    if is_domain_blocked(&domain) {
        let response = "HTTP/1.1 403 Forbidden\r\n\r\nBlocked";
        client_stream.write_all(response.as_bytes()).await.unwrap();
        return;
    }

    let first_line = request.lines().next().unwrap_or("");
    let path = extract_path(first_line);
    let server_request = transform_request_for_server(&request, &domain, &path);

    let addr = format!("{}:80", domain);
    let mut server_stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => {
            let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
            client_stream.write_all(response.as_bytes()).await.unwrap();
            return;
        }
    };
    
    server_stream.write_all(server_request.as_bytes()).await.unwrap();
    
    let mut response_buf = [0; 4096];
    loop {
        match server_stream.read(&mut response_buf).await {
            Ok(0) => break,
            Ok(n) => {
                if client_stream.write_all(&response_buf[..n]).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

async fn handle_connect(mut client_stream: TcpStream, domain: String, _request: String) {
    if is_domain_blocked(&domain) {
        let response = "HTTP/1.1 403 Forbidden\r\n\r\nBlocked";
        client_stream.write_all(response.as_bytes()).await.unwrap();
        return;
    }

    let addr = format!("{}:443", domain); // <-- исправлено 433 -> 443
    let mut server_stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => {
            let response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
            client_stream.write_all(response.as_bytes()).await.unwrap();
            return;
        }
    };

    let response = "HTTP/1.1 200 Connection established\r\n\r\n";
    client_stream.write_all(response.as_bytes()).await.unwrap();

    match tokio::io::copy_bidirectional(&mut client_stream, &mut server_stream).await {
        Ok(_) => println!("Tunnel closed for {}", domain),
        Err(e) => println!("Tunnel error: {}", e),
    }
}

fn transform_request_for_server(request: &str, domain: &str, path: &str) -> String {
    let mut lines: Vec<String> = request.lines().map(|s| s.to_string()).collect();
    
    let first_line = &lines[0];
    let method = first_line.split_whitespace().next().unwrap_or("GET");
    lines[0] = format!("{} {} HTTP/1.1", method, path);
    
    let mut has_host = false;
    for line in &mut lines {
        if line.to_lowercase().starts_with("host:") {
            *line = format!("Host: {}", domain);
            has_host = true;
            break;
        }
    }
    if !has_host {
        lines.push(format!("Host: {}", domain));
    }
    
    lines.join("\r\n") + "\r\n\r\n"
}

fn extract_domain_connect(first_line: &str) -> String {
    let rest = first_line.replacen("CONNECT ", "", 1);
    let host_port = rest.split_whitespace().next().unwrap_or("");
    let domain = host_port.split(':').next().unwrap_or("");
    domain.to_string()
}

fn extract_domain_http(first_line: &str) -> String {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 { return "error".to_string(); }
    
    let url = parts[1];
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let domain = url.split('/').next().unwrap_or("");
    let domain = domain.split(':').next().unwrap_or("");
    domain.to_string()
}

fn extract_path(first_line: &str) -> String {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 { return "/".to_string(); }
    
    let url = parts[1];
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let path = url.split('/').nth(1).unwrap_or("");
    format!("/{}", path)
}

fn is_domain_blocked(domain: &str) -> bool {
    let c_domain = CString::new(domain).unwrap();
    let c_ptr = c_domain.as_ptr();
    let result = unsafe { search_domen(c_ptr) };
    result == 1
}

fn add_domain_to_blacklist(domain: &str) {
    let c_domain = CString::new(domain).unwrap();
    let c_ptr = c_domain.as_ptr();
    unsafe { append_to_blacklist(c_ptr) };
    println!("Domain {} added to blacklist", domain)
}