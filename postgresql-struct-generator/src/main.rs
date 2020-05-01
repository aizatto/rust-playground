use native_tls::{TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use heck::CamelCase;

use postgres::{Client, Error};
use std::env;
// use std::fs;

fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    // let cert = fs::read("ca-certificate.crt").unwrap();
    // let cert2 = cert.clone();
    // println!("{}", String::from_utf8(cert).unwrap());
    // let cert = Certificate::from_pem(&cert).unwrap();

    let connector = TlsConnector::builder()
        // .add_root_certificate(cert)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
        // .build() {
        //     Ok(conn) => conn,
        //     Err(e) => {
        //         panic!(e);
        //     }
        // };
    let connector = MakeTlsConnector::new(connector);


    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let mut client = Client::connect(&database_url, connector)?;
    let tables = client.query(
        "SELECT tablename FROM pg_tables WHERE schemaname='public' ORDER BY tablename ASC;",
        &[],
    )?;

    let statement = client.prepare("SELECT column_name, data_type FROM information_schema.columns WHERE table_name = $1")?;

    let mut output = String::new();

    // println!("row {}", &tables);
    // println!("table length: {}", tables.len());
    for table in tables {
        let tablename: String = table.get(0);
        let columns = client.query(&statement, &[&tablename])?;

        // let table_name: str = str::from_utf8(tablename);
        // https://stackoverflow.com/questions/23975391/how-to-convert-a-string-into-a-static-str
        let camel_name: &str = &format!("struct {} {{\n", &tablename[..].to_camel_case()).to_string();
        output.push_str(camel_name);

        for column in columns {
           let column_name: String = column.get("column_name");
           let column_type: String = column.get("data_type");
           output.push_str(&print_column(column_name, column_type));
        }
        output.push_str("}\n\n");
    }
    println!("{}", output);

    Ok(())
}

// https://github.com/sfackler/rust-postgres/blob/master/postgres-types/src/lib.rs#L372
// https://github.com/sfackler/rust-postgres/blob/master/postgres-types/src/uuid_08.rs
fn print_column(name: String, type_: String) -> String {
    // https://stackoverflow.com/questions/25383488/how-to-match-a-string-against-string-literals-in-rust
    let rs_type = match &type_[..] {
        "BOOL" => "bool",
        "char" => "i8",
        "SMALLINT" => "i16",
        "SMALLSERIAL" => "i16",
        "INT" => "i32",
        "SERIAL" => "i32",
        "OID" => "u32",
        "BIGINT" => "i64",
        "BIGSERIAL" => "i64",
        "REAL" => "f32",
        "DOUBLE PRECISION" => "f64",
        "character varying" => "String",
        "integer" => "i32",
        "json" => "serde_json::Value",
        "jsonb" => "serde_json::Value",
        "text" => "String",
        "timestamp with time zone" => "chrono::DateTime<Utc>",
        "timestamp without time zone" => "chrono::DateTime<Local>",
        "uuid" => "uuid:Uuid",
        _ => "whatever",
    };

    format!("    {}: {} // {}\n", name, rs_type, type_)
}