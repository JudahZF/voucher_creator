use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde::Deserialize;
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, services::ServeDir};

mod database;
mod qr_generator;
mod templates;
mod voucher;
mod wifi_network;

use database::Database;
use qr_generator::QrGenerator;
use voucher::Voucher;
use wifi_network::WiFiNetwork;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "3000")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Clone)]
struct AppState {
    database: Arc<Database>,
    qr_generator: QrGenerator,
}

#[derive(Deserialize)]
struct GenerateQuery {
    network_id: Option<String>, // specific network ID
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Starting WiFi Voucher Generator");
    println!("Server: http://{}:{}", args.host, args.port);

    // Initialize database
    let db_path = env::current_dir()?.join("vouchers.db");
    let database_url = format!("sqlite:{}", db_path.display());
    let database = Arc::new(Database::new(&database_url).await?);
    println!("Database initialized at: {}", db_path.display());

    // Initialize application state
    let state = AppState {
        database,
        qr_generator: QrGenerator::new(),
    };

    let app = Router::new()
        .route(
            "/",
            get(|| async { axum::response::Redirect::permanent("/admin") }),
        )
        .route("/upload", post(upload_csv))
        .route("/generate", get(generate_vouchers))
        .route("/print", post(print_vouchers))
        .route("/vouchers", get(list_vouchers))
        .route("/admin", get(admin_page))
        .route("/admin/networks", post(create_network))
        .route("/admin/networks/:id/delete", post(delete_network))
        .route("/admin/upload", post(admin_upload_csv))
        .route("/admin/networks/:id/vouchers", get(network_vouchers))
        .route("/vouchers/:id/use", post(mark_voucher_used))
        .route("/vouchers/:id/unuse", post(mark_voucher_unused))
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
        let csv_data = "voucher_code\n# This is a comment\nVOUCHER001\n# Another comment\nVOUCHER002\n\n# Final comment\nVOUCHER003";

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
        let csv_data = "VOUCHER001\nVOUCHER002\nVOUCHER003";

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
        let csv_data = "# Comment 1\n# Comment 2\n# Comment 3";

        let result = process_csv_data(csv_data).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid voucher codes found"));
    }

    #[tokio::test]
    async fn test_csv_processing_mixed_content() {
        let csv_data = "# WiFi Codes\nvoucher_code\n# Hotel codes\nHOTEL-001\nHOTEL-002\n# Guest codes\nGUEST-001\n# End";

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
    let networks = state.database.get_all_networks().await.unwrap_or_default();

    // Get voucher counts for each network
    let mut network_counts = Vec::new();
    for network in &networks {
        let counts = state
            .database
            .get_voucher_counts(&network.id)
            .await
            .unwrap_or(database::VoucherCounts {
                total: 0,
                used: 0,
                unused: 0,
                printed: 0,
                unprinted: 0,
            });
        network_counts.push(counts);
    }

    Html(templates::admin_template(&networks, &network_counts))
}

async fn create_network(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut form_data: HashMap<String, String> = HashMap::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if let Some(name) = field.name() {
            let name = name.to_string();
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            form_data.insert(name, value);
        }
    }

    let ssid = form_data
        .get("ssid")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    let password = form_data
        .get("password")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    let name = form_data
        .get("name")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();
    let description = form_data.get("description").cloned();

    let network = WiFiNetwork::new(name, ssid, password, description);

    if let Err(_) = state.database.create_network(&network).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::response::Redirect::to("/admin").into_response())
}

async fn delete_network(
    State(state): State<AppState>,
    Path(network_id): Path<String>,
) -> impl IntoResponse {
    // Delete the network (which will cascade delete vouchers due to foreign key)
    let _ = state.database.delete_network(&network_id).await;

    axum::response::Redirect::to("/admin")
}

async fn admin_upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut network_id = String::new();
    let mut csv_data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("network_id") => {
                network_id = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            Some("csv_file") => {
                csv_data = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .to_vec();
            }
            _ => {}
        }
    }

    if network_id.is_empty() || csv_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let csv_string = String::from_utf8(csv_data).map_err(|_| StatusCode::BAD_REQUEST)?;

    match process_csv_data(&csv_string).await {
        Ok(vouchers) => {
            let mut network_vouchers = vouchers;

            // Set network_id for all vouchers
            for voucher in &mut network_vouchers {
                voucher.network_id = Some(network_id.clone());
            }

            if let Err(_) = state.database.create_vouchers(&network_vouchers).await {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(axum::response::Redirect::to("/admin").into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn network_vouchers(
    State(state): State<AppState>,
    Path(network_id): Path<String>,
) -> Html<String> {
    let network = state
        .database
        .get_network(&network_id)
        .await
        .unwrap_or(None);
    let vouchers = state
        .database
        .get_vouchers_for_network(&network_id)
        .await
        .unwrap_or_default();
    let voucher_counts = state
        .database
        .get_voucher_counts(&network_id)
        .await
        .unwrap_or(database::VoucherCounts {
            total: 0,
            used: 0,
            unused: 0,
            printed: 0,
            unprinted: 0,
        });

    Html(templates::network_vouchers_template(
        network.as_ref(),
        &vouchers,
        &network_id,
        &voucher_counts,
    ))
}

async fn upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("csv_file") {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            let csv_content =
                String::from_utf8(data.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;

            match process_csv_data(&csv_content).await {
                Ok(vouchers) => {
                    if let Err(_) = state.database.create_vouchers(&vouchers).await {
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    let buttons = r#"
                        <a href="/vouchers" class="bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                            <i class="fas fa-list mr-2"></i>View Vouchers
                        </a>
                        <a href="/generate" class="bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-600 hover:to-teal-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                            <i class="fas fa-qrcode mr-2"></i>Generate QR Codes
                        </a>
                        <a href="/admin" class="bg-gradient-to-r from-purple-500 to-indigo-600 hover:from-purple-600 hover:to-indigo-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                            <i class="fas fa-cog mr-2"></i>Admin Panel
                        </a>
                        "#;

                    return Ok((
                        StatusCode::OK,
                        Html(templates::success_response(
                            "CSV Uploaded Successfully!",
                            &format!("Your CSV file has been processed and {} voucher codes have been loaded into the system. You can now generate QR code vouchers for printing.", vouchers.len()),
                            vouchers.len(),
                            buttons
                        ))
                    ));
                }
                Err(e) => {
                    let buttons = r#"
                        <a href="/" class="bg-gradient-to-r from-gray-500 to-gray-600 hover:from-gray-600 hover:to-gray-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                            <i class="fas fa-arrow-left mr-2"></i>Go Back
                        </a>
                        <a href="/admin" class="bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-6 py-3 rounded-xl font-semibold transition-all duration-200 transform hover:scale-105 shadow-lg hover:shadow-xl">
                            <i class="fas fa-cog mr-2"></i>Try Admin Panel
                        </a>
                    "#;

                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Html(templates::error_response(
                            "CSV Processing Failed",
                            &format!("We encountered an error while processing your CSV file: {}. Please check your file format and try again.", e),
                            buttons
                        ))
                    ));
                }
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

async fn process_csv_data(csv_content: &str) -> anyhow::Result<Vec<Voucher>> {
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
    let vouchers = state.database.get_all_vouchers().await.unwrap_or_default();

    if vouchers.is_empty() {
        return Html(templates::no_vouchers_template());
    }

    let voucher_list = vouchers
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let status = if v.is_used { "Used" } else { "Available" };
            let status_class = if v.is_used { "text-danger" } else { "text-success" };
            let used_at = match &v.used_at {
                Some(time) => time.format("%Y-%m-%d %H:%M").to_string(),
                None => "".to_string()
            };

            format!(
                r#"<tr>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td><span class="{}">{}</span></td>
                    <td>{}</td>
                    <td>
                        {}
                    </td>
                </tr>"#,
                i + 1,
                v.code,
                status_class,
                status,
                used_at,
                if v.is_used {
                    format!(r#"<button class="btn btn-sm btn-warning" onclick="markUnused('{}')">Mark Unused</button>"#, v.id)
                } else {
                    format!(r#"<button class="btn btn-sm btn-success" onclick="markUsed('{}')">Mark Used</button>"#, v.id)
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Voucher List</title>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet">
            <script>
                function markUsed(id) {{
                    fetch('/vouchers/' + id + '/use', {{ method: 'POST' }})
                        .then(() => location.reload());
                }}
                function markUnused(id) {{
                    fetch('/vouchers/' + id + '/unuse', {{ method: 'POST' }})
                        .then(() => location.reload());
                }}
            </script>
        </head>
        <body>
            <div class="container mt-4">
                <h1>All Vouchers</h1>
                <p class="text-muted">Total: {} vouchers</p>

                <div class="mb-3">
                    <a href="/admin" class="btn btn-secondary">Back to Admin</a>
                    <a href="/generate" class="btn btn-success">Generate QR Codes</a>
                </div>

                <div class="table-responsive">
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th>#</th>
                                <th>Voucher Code</th>
                                <th>Status</th>
                                <th>Used At</th>
                                <th>Actions</th>
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
    let network_id = match &params.network_id {
        Some(i) => i,
        None => return Err(StatusCode::PARTIAL_CONTENT),
    };

    // Get network from database
    let network = state
        .database
        .get_network(network_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get voucher counts for display
    let voucher_counts = state
        .database
        .get_voucher_counts(network_id)
        .await
        .unwrap_or(database::VoucherCounts {
            total: 0,
            used: 0,
            unused: 0,
            printed: 0,
            unprinted: 0,
        });

    if voucher_counts.total == 0 {
        return Ok(Html(templates::no_vouchers_template()).into_response());
    }

    // Return print selection page
    let html_content = templates::print_selection_page(&network, &voucher_counts);
    Ok(Html(html_content).into_response())
}

async fn print_vouchers(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let mut network_id = String::new();
    let mut count = 0usize;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("network_id") => {
                network_id = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            Some("count") => {
                let count_str = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                count = count_str.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            _ => {}
        }
    }

    if network_id.is_empty() || count == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get network from database
    let network = state
        .database
        .get_network(&network_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get unprinted vouchers up to the requested count
    let vouchers = state
        .database
        .get_unprinted_vouchers_for_network(&network_id, Some(count))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if vouchers.is_empty() {
        return Ok(Html(templates::no_unprinted_vouchers_template()).into_response());
    }

    // Mark these vouchers as printed
    let voucher_ids: Vec<String> = vouchers.iter().map(|v| v.id.clone()).collect();
    let _ = state.database.mark_vouchers_as_printed(&voucher_ids).await;

    // Generate WiFi QR code
    let wifi_qr_data = format!(
        "WIFI:T:WPA;S:{};P:{};H:false;;",
        network.ssid, network.password
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
                &network.ssid,
                &network.name,
                &voucher.code,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html_content = templates::generate_vouchers_page(
        vouchers.len(),
        &network.name,
        &network.ssid,
        &voucher_cards,
    );

    Ok(Html(html_content).into_response())
}

// Handler for marking voucher as used
async fn mark_voucher_used(
    State(state): State<AppState>,
    Path(voucher_id): Path<String>,
) -> impl IntoResponse {
    let _ = state.database.mark_voucher_as_used(&voucher_id).await;
    axum::response::Redirect::to("/vouchers")
}

// Handler for marking voucher as unused
async fn mark_voucher_unused(
    State(state): State<AppState>,
    Path(voucher_id): Path<String>,
) -> impl IntoResponse {
    let _ = state.database.mark_voucher_as_unused(&voucher_id).await;
    axum::response::Redirect::to("/vouchers")
}
