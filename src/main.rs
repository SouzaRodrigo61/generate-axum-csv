extern crate csv;
use std::fs;
use std::net::SocketAddr;
use std::{fs::File, io::Read};

use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    routing::get,
    Router,
};

use tokio::net::TcpListener;


fn write_csv() -> Result<Vec<u8>, String> {
    // Create a new CSV writer.
    let file = File::create("users.csv").expect("Couldn't create users.csv");

    let users: Vec<(&str, &str)> = vec![("Alice", "30"), ("Bob", "35")];

    {
        let mut writer = csv::Writer::from_writer(file);

        // Write the header.
        writer.write_record(&["Name", "Person Age"]).expect("Error writing header"); 

        // then write the records by looping over the vec
        for user in &users {
            let (name, age) = user;
            writer.write_record(&[name, age]).expect("Error writing record.");
        }
    }

    let mut f = File::open("users.csv").expect("msg");
    let mut data = vec![];
    let _ = f
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())
        .expect("");

    fs::remove_file("users.csv").expect("msg");

    Ok(data)
}


async fn excel_handler() -> impl IntoResponse {
    match write_csv() {
        Ok(buffer) => (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"data.csv\""),
            ],
            buffer,
        ),
        Err(_) => (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"data.csv\""),
            ],
            Vec::new(),
        ),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv::dotenv();
    let port = std::env::var("PORT").expect("PORT must be set");
    
    let app = Router::new().route("/", get(excel_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse::<u16>().unwrap()));
    // let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    println!("Running generate csv");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
