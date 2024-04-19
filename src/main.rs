extern crate csv;
use std::net::SocketAddr;
use std::{fs, vec};
use std::{fs::File, io::Read};

use axum::http::header;
use axum::response::IntoResponse;
use axum::{routing::get, Router};

use chrono::{DateTime, Utc};
use sqlx::{FromRow, MySqlPool};
use tokio::net::TcpListener;

#[derive(Debug, Clone, FromRow)]
struct OpeningParcel {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub whatsapp: String,
    pub code_booklet: i64,
    pub quota: i64,
    pub payment_booklet_id: Option<String>,
}

impl OpeningParcel {
    fn to_csv(&self) -> [String; 7] {
        let code_booklet_str = self.code_booklet.to_string();
        let quota_str = self.quota.to_string();
        let payment_booklet_id_str = self.payment_booklet_id.to_owned().unwrap_or("".to_string());

        [
            self.id.clone(),
            self.name.clone(),
            self.phone.clone(),
            self.whatsapp.clone(),
            code_booklet_str.clone(),
            quota_str.clone(),
            payment_booklet_id_str.clone(),
        ]
    }
}

async fn get_all_parcel(connection_pool: &MySqlPool) -> anyhow::Result<Vec<OpeningParcel>> {
    Ok(sqlx::query_as::<_, OpeningParcel>(
        r#"
            SELECT b.id, a.name, a.cpf, a.phone, a.whatsapp, b.codeBooklet as code_booklet, b.quota, b.paymentBookId as payment_booklet_id
            FROM Booklet b 
            INNER JOIN Acquirer a ON a.id = b.acquirerId 
            ORDER by b.codeBooklet asc, b.quota asc;
        "#,
    )
    .fetch_all(connection_pool)
    .await?)
}

async fn write_csv() -> Result<Vec<u8>, String> {

    // Obtener la fecha y hora actual en UTC
    let utc: DateTime<Utc> = Utc::now();

    // Convertir la marca de tiempo a una cadena
    let timestamp_str = utc.to_rfc3339();

    println!("Starting processing: {}", timestamp_str);

    // Create a new CSV writer.
    let file = File::create("users.csv").expect("Couldn't create users.csv");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = match MySqlPool::connect(&database_url).await {
        Ok(connection_pool) => connection_pool,
        Err(err) => {
            println!("Error (MySqlPool::connect): {:?}", err);
            std::process::exit(1)
        }
    };

    let data = match get_all_parcel(&connection).await {
        Ok(connection_pool) => connection_pool,
        Err(err) => {
            println!("Error (get_all_parcel): {:?}", err);
            std::process::exit(1)
        }
    };

    {

        let mut writer = csv::Writer::from_writer(file);

        // Write the header.
        writer
            .write_record(&[
                "ID",
                "Nome",
                "Telefone Celular",
                "Whatsapp",
                "Numero do Carnê",
                "Parcela",
                "Foi Pago",
            ])
            .expect("Error writing header");

        // then write the records by looping over the vec
        for parcel in &data {
            
            writer
                .write_record(&parcel.to_csv())
                .expect("Error writing header");
        }
    }

    let mut f = File::open("users.csv").expect("msg");
    let mut data = vec![];
    let _ = f
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())
        .expect("");

    fs::remove_file("users.csv").expect("msg");


    // Obtener la fecha y hora actual en UTC
    let utc: DateTime<Utc> = Utc::now();

    // Convertir la marca de tiempo a una cadena
    let timestamp_str = utc.to_rfc3339();

    println!("Finish processing: {}", timestamp_str);

    Ok(data)
}

async fn excel_handler() -> impl IntoResponse {
    match write_csv().await {
        Ok(buffer) => (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"data.csv\"",
                ),
            ],
            buffer,
        ),
        Err(_) => (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"data.csv\"",
                ),
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
    let listener = TcpListener::bind(addr).await?;

    println!("Running generate csv");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
