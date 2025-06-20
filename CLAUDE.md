# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WiFi Voucher Generator is a Rust web application that generates printable WiFi vouchers with QR codes. It manages multiple WiFi networks, tracks voucher usage, and creates print-ready voucher cards.

## Development Commands

### Build and Run
```bash
# Build the project
cargo build --release

# Run in development mode
cargo run

# Run with custom port and host
cargo run -- --port 8080 --host 0.0.0.0

# Run tests
cargo test --verbose

# Run linter (CI requirement)
cargo clippy -- -Dwarnings
```

### Database
- SQLite database is automatically created at `vouchers.db`
- Database migrations run automatically on startup
- Schema includes `wifi_networks` and `vouchers` tables with foreign key relationships

## Architecture

### Core Components

- **main.rs**: Axum web server with route handlers for admin, voucher management, and printing
- **database.rs**: SQLite database operations using sqlx with connection pooling
- **voucher.rs**: Voucher entity with UUID-based IDs and usage tracking
- **wifi_network.rs**: WiFi network entity with SSID, password, and metadata
- **qr_generator.rs**: QR code generation for WiFi connection strings
- **templates.rs**: HTML template loading and rendering system

### Key Routes
- `/admin` - Main management dashboard
- `/vouchers` - View all vouchers across networks
- `/generate?network_id={id}` - Print voucher selection
- `/admin/networks/{id}/vouchers` - Network-specific voucher management

### Data Flow
1. Networks are created via admin interface
2. CSV files with voucher codes are uploaded and associated with networks
3. Vouchers can be generated with QR codes for WiFi connection
4. Print status and usage is tracked per voucher

### Template System
- HTML templates stored in `templates/` directory
- Templates use `{{PLACEHOLDER}}` syntax for variable replacement
- Template loading happens at runtime via `fs::read_to_string`

### CSV Processing
- Supports comments (lines starting with `#`)
- Skips empty lines automatically
- First column contains voucher codes
- Headers are automatically detected and handled

## Testing

- Unit tests are embedded in each module using `#[cfg(test)]`
- Integration tests focus on CSV processing and voucher creation
- Test database operations use in-memory SQLite
- CI runs `cargo test --verbose` and `cargo clippy -- -Dwarnings`