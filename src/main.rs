use actix_files as fs;
use actix_web::{App, HttpServer};
use clap::Parser;
use local_ip_address::local_ip;
use mimalloc_rust::GlobalMiMalloc;
use qrcode::QrCode;

#[global_allocator]
static GLOBAL_MIMALLOC: GlobalMiMalloc = GlobalMiMalloc;

#[derive(Clone, Parser)]
struct Config {
    /// [optional] Port number exposed by file service (uses 8080 by default)
    #[arg(short, long)]
    port: Option<u16>,
    /// [optional] URL path to the mount file service (uses `/static` by default)
    #[arg(short, long)]
    mount_path: Option<String>,
    /// [optional] Relative path that specifies the location of the local file service, must be a
    /// path that specifies a folder (uses `.` by default)
    #[arg(short, long)]
    serve_from: Option<String>,
}

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_MOUNT_PATH: &str = "/static";
const DEFAULT_RELATIVE_PATH: &str = ".";

fn get_local_addr() -> String {
    local_ip().expect("get local ip never fails").to_string()
}

impl Config {
    fn get_bind_addr(&self) -> String {
        let port = match self.port {
            Some(p) => p,
            None => DEFAULT_PORT,
        };
        let local_addr = get_local_addr();
        format!("{local_addr}:{port}")
    }

    fn get_mount_path_or_default(&self) -> &str {
        match self.mount_path.as_ref() {
            Some(p) => p.as_str(),
            None => DEFAULT_MOUNT_PATH,
        }
    }

    fn get_relative_path_or_default(&self) -> &str {
        match self.serve_from.as_ref() {
            Some(p) => p.as_str(),
            None => DEFAULT_RELATIVE_PATH,
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let bind_addr = config.get_bind_addr();
    println!(
        "The file service is about to listen to `http://{bind_addr}{}`",
        config.get_mount_path_or_default()
    );
    // show QR code
    let code = QrCode::new(bind_addr.as_str()).unwrap();
    let connect_code = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{connect_code}");

    // start server
    HttpServer::new(move || {
        App::new().service(
            fs::Files::new(
                config.get_mount_path_or_default(),
                config.get_relative_path_or_default(),
            )
            .show_files_listing()
            .prefer_utf8(true),
        )
    })
    .bind(bind_addr.as_str())?
    .run()
    .await
}
