use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir};

mod qr_generator;
mod voucher;
mod templates;
mod wifi_network;

use qr_generator::QrGenerator;
use voucher::{Voucher, VoucherManager};
use wifi_network::{WiFiNetwork, WiFiNetworkManager};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WiFi SSID
    #[arg(short, long)]
    ssid: String,

    /// WiFi Password
    #[arg(short, long)]
    password: String,

    /// Port to run the web server on
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Host to bind the web server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Clone)]
struct AppState {
    wifi_ssid: String,
    wifi_password: String,
    voucher_manager: Arc<RwLock<VoucherManager>>,
    wifi_network_manager: Arc<RwLock<WiFiNetworkManager>>,
    qr_generator: QrGenerator,
}

#[derive(Deserialize)]
struct GenerateQuery {
    format: Option<String>, // "html" or "pdf"
    network_id: Option<String>, // specific network ID
}

#[derive(Deserialize)]
struct NetworkForm {
    ssid: String,
    password: String,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct VoucherUploadForm {
    network_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Starting WiFi Voucher Generator");
    println!("SSID: {}", args.ssid);
    println!("Server: http://{}:{}", args.host, args.port);

    let mut network_manager = WiFiNetworkManager::new();
    
    // Create default network from command line args
    let default_network = WiFiNetwork::new(
        "Default Network".to_string(),
        args.ssid.clone(),
        args.password.clone(),
        Some("Network created from command line arguments".to_string()),
    );
    network_manager.add_network(default_network);

    let state = AppState {
        wifi_ssid: args.ssid,
        wifi_password: args.password,
        voucher_manager: Arc::new(RwLock::new(VoucherManager::new())),
        wifi_network_manager: Arc::new(RwLock::new(network_manager)),
        qr_generator: QrGenerator::new(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload_csv))
        .route("/generate", get(generate_vouchers))
        .route("/vouchers", get(list_vouchers))
        .route("/admin", get(admin_page))
        .route("/admin/networks", post(create_network))
        .route("/admin/networks/:id/delete", post(delete_network))
        .route("/admin/upload", post(admin_upload_csv))
        .route("/admin/networks/:id/vouchers", get(network_vouchers))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", args.host, args.port)
        .parse::<SocketAddr>()
        .expect("Invalid address");

    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csv_processing_with_comments() {
        let csv_data = b"voucher_code\n# This is a comment\nVOUCHER001\n# Another comment\nVOUCHER002\n\n# Final comment\nVOUCHER003";
        
        let result = process_csv_data(csv_data).await;
        assert!(result.is_ok());
        
        let vouchers = result.unwrap();
        assert_eq!(vouchers.len(), 3);
        assert_eq!(vouchers[0].code, "VOUCHER001");
        assert_eq!(vouchers[1].code, "VOUCHER002");
        assert_eq!(vouchers[2].code, "VOUCHER003");
    }

    #[tokio::test]
    async fn test_csv_processing_ignores_empty_lines() {
        let csv_data = b"VOUCHER001\nVOUCHER002\nVOUCHER003";
        
        let result = process_csv_data(csv_data).await;
        assert!(result.is_ok());
        
        let vouchers = result.unwrap();
        // CSV reader treats first line as header and skips it
        assert_eq!(vouchers.len(), 2);
        assert_eq!(vouchers[0].code, "VOUCHER002");
        assert_eq!(vouchers[1].code, "VOUCHER003");
    }

    #[tokio::test]
    async fn test_csv_processing_only_comments() {
        let csv_data = b"# Comment 1\n# Comment 2\n# Comment 3";
        
        let result = process_csv_data(csv_data).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No valid voucher codes found"));
    }

    #[tokio::test]
    async fn test_csv_processing_mixed_content() {
        let csv_data = b"# WiFi Codes\nvoucher_code\n# Hotel codes\nHOTEL-001\nHOTEL-002\n# Guest codes\nGUEST-001\n# End";
        
        let result = process_csv_data(csv_data).await;
        assert!(result.is_ok());
        
        let vouchers = result.unwrap();
        assert_eq!(vouchers.len(), 4); // voucher_code header is treated as a voucher code
        assert_eq!(vouchers[0].code, "voucher_code");
        assert_eq!(vouchers[1].code, "HOTEL-001");
        assert_eq!(vouchers[2].code, "HOTEL-002");
        assert_eq!(vouchers[3].code, "GUEST-001");
    }
}

// New admin functions
async fn admin_page(State(state): State<AppState>) -> Html<String> {
    let network_manager = state.wifi_network_manager.read().await;
    let voucher_manager = state.voucher_manager.read().await;
    
    let networks = network_manager.get_all_networks();
    
    Html(templates::admin_template(&networks, &voucher_manager))
}

async fn create_network(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut form_data: HashMap<String, String> = HashMap::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if let Some(name) = field.name() {
            let name = name.to_string();
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            form_data.insert(name, value);
        }
    }
    
    let ssid = form_data.get("ssid").ok_or(StatusCode::BAD_REQUEST)?.clone();
    let password = form_data.get("password").ok_or(StatusCode::BAD_REQUEST)?.clone();
    let name = form_data.get("name").ok_or(StatusCode::BAD_REQUEST)?.clone();
    let description = form_data.get("description").cloned();
    
    let network = WiFiNetwork::new(name, ssid, password, description);
    
    let mut network_manager = state.wifi_network_manager.write().await;
    network_manager.add_network(network);
    
    Ok(axum::response::Redirect::to("/admin").into_response())
}

async fn delete_network(
    State(state): State<AppState>,
    Path(network_id): Path<String>,
) -> impl IntoResponse {
    let mut network_manager = state.wifi_network_manager.write().await;
    let mut voucher_manager = state.voucher_manager.write().await;
    
    network_manager.remove_network(&network_id);
    voucher_manager.remove_vouchers_for_network(&network_id);
    
    axum::response::Redirect::to("/admin")
}

async fn admin_upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut network_id = String::new();
    let mut csv_data = Vec::new();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        match field.name() {
            Some("network_id") => {
                network_id = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            Some("csv_file") => {
                csv_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            }
            _ => {}
        }
    }
    
    if network_id.is_empty() || csv_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match process_csv_data(&csv_data).await {
        Ok(mut vouchers) => {
            // Assign network ID to all vouchers
            for voucher in &mut vouchers {
                voucher.network_id = Some(network_id.clone());
            }
            
            let mut manager = state.voucher_manager.write().await;
            manager.add_vouchers(vouchers.clone());
            
            Ok(axum::response::Redirect::to("/admin").into_response())
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn network_vouchers(
    State(state): State<AppState>,
    Path(network_id): Path<String>,
) -> Html<String> {
    let voucher_manager = state.voucher_manager.read().await;
    let network_manager = state.wifi_network_manager.read().await;
    
    let network = network_manager.get_network(&network_id);
    let vouchers = voucher_manager.get_vouchers_for_network(&network_id);
    
    Html(templates::network_vouchers_template(network, &vouchers, &network_id))
}

async fn index() -> Html<String> {
    Html(templates::index_template())
}

async fn upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if field.name() == Some("csv_file") {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            match process_csv_data(&data).await {
                Ok(vouchers) => {
                    let mut manager = state.voucher_manager.write().await;
                    manager.add_vouchers(vouchers);
                    
                    return Ok((
                        StatusCode::OK,
                        Html(format!(
                            r#"
                            <div class="alert alert-success">
                                <h4>Success!</h4>
                                <p>CSV uploaded successfully. {} vouchers loaded.</p>
                                <a href="/vouchers" class="btn btn-primary">View Vouchers</a>
                                <a href="/generate" class="btn btn-success">Generate QR Codes</a>
                                <a href="/admin" class="btn btn-info">Admin Panel</a>
                            </div>
                            "#,
                            manager.voucher_count()
                        ))
                    ));
                }
                Err(e) => {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Html(format!(
                            r#"
                            <div class="alert alert-danger">
                                <h4>Error!</h4>
                                <p>Failed to process CSV: {}</p>
                                <a href="/" class="btn btn-secondary">Go Back</a>
                            </div>
                            "#,
                            e
                        ))
                    ));
                }
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

async fn process_csv_data(data: &[u8]) -> anyhow::Result<Vec<Voucher>> {
    let csv_content = String::from_utf8(data.to_vec())?;
    let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
    let mut vouchers = Vec::new();

    for result in reader.records() {
        let record = result?;
        if let Some(code) = record.get(0) {
            let trimmed_code = code.trim();
            // Skip empty lines and comment lines starting with #
            if !trimmed_code.is_empty() && !trimmed_code.starts_with('#') {
                vouchers.push(Voucher::new(trimmed_code.to_string()));
            }
        }
    }

    if vouchers.is_empty() {
        anyhow::bail!("No valid voucher codes found in CSV");
    }

    Ok(vouchers)
}

async fn list_vouchers(State(state): State<AppState>) -> Html<String> {
    let manager = state.voucher_manager.read().await;
    let vouchers = manager.get_all_vouchers();
    
    if vouchers.is_empty() {
        return Html(templates::no_vouchers_template());
    }

    let voucher_list = vouchers
        .iter()
        .enumerate()
        .map(|(i, v)| format!(
            r#"<tr><td>{}</td><td><code>{}</code></td></tr>"#,
            i + 1,
            v.code
        ))
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Voucher List</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet">
        </head>
        <body>
            <div class="container mt-4">
                <h1>Loaded Vouchers</h1>
                <p class="text-muted">Total: {} vouchers</p>
                
                <div class="mb-3">
                    <a href="/" class="btn btn-secondary">Back to Upload</a>
                    <a href="/generate" class="btn btn-success">Generate QR Codes</a>
                </div>
                
                <div class="table-responsive">
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th>#</th>
                                <th>Voucher Code</th>
                            </tr>
                        </thead>
                        <tbody>
                            {}
                        </tbody>
                    </table>
                </div>
            </div>
        </body>
        </html>
        "#,
        vouchers.len(),
        voucher_list
    ))
}

async fn generate_vouchers(
    State(state): State<AppState>,
    Query(params): Query<GenerateQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let manager = state.voucher_manager.read().await;
    let network_manager = state.wifi_network_manager.read().await;
    
    // Determine which network to use
    let (network_ssid, network_password, network_name, vouchers) = if let Some(network_id) = &params.network_id {
        // Generate vouchers for specific network
        let network = network_manager.get_network(network_id)
            .ok_or(StatusCode::NOT_FOUND)?;
        let vouchers = manager.get_vouchers_for_network(network_id);
        (network.ssid.clone(), network.password.clone(), network.name.clone(), vouchers)
    } else {
        // Generate vouchers for default network (backward compatibility)
        let vouchers = manager.get_all_vouchers();
        (state.wifi_ssid.clone(), state.wifi_password.clone(), "Default Network".to_string(), vouchers)
    };
    
    if vouchers.is_empty() {
        return Ok(Html(templates::no_vouchers_template()).into_response());
    }

    // Generate WiFi QR code
    let wifi_qr_data = format!(
        "WIFI:T:WPA;S:{};P:{};H:false;;",
        network_ssid,
        network_password
    );

    let wifi_qr_base64 = match state.qr_generator.generate_qr_base64(&wifi_qr_data) {
        Ok(qr) => qr,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Generate voucher cards using template
    let voucher_cards = vouchers
        .iter()
        .map(|voucher| {
            templates::generate_voucher_card(
                &wifi_qr_base64,
                &network_ssid,
                &voucher.code
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html_content = templates::generate_vouchers_page(
        vouchers.len(),
        &network_name,
        &network_ssid,
        &voucher_cards
    );

    Ok(Html(html_content).into_response())
}