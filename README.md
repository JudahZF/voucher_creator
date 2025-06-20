# WiFi Voucher Generator

A Rust-based web application that generates printable WiFi vouchers with QR codes. Create and manage multiple WiFi networks, upload voucher codes, track voucher usage, and create beautiful, print-ready voucher cards.

## Features

- 🎯 **Easy CSV Upload**: Simple drag-and-drop interface for uploading voucher codes
- 🔗 **WiFi QR Codes**: Automatically generates QR codes for instant WiFi connection
- 🎨 **Print-Ready Design**: Professional voucher cards optimized for printing (8+ per page)
- 🚀 **Fast & Lightweight**: Built with Rust for maximum performance
- 📱 **Responsive Web Interface**: Works on desktop and mobile devices
- 🏢 **Multi-Network**: Create and manage different WiFi networks for different purposes
- 🎛️ **Admin Dashboard**: Comprehensive management interface with statistics

## Installation

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)

### Build from Source

1. Clone the repository:
```bash
git clone <repository-url>
cd pfsense_portal_generator
```

2. Build the application:
```bash
cargo build --release
```

3. The compiled binary will be available at `target/release/pfsense_portal_generator`

## Usage

### Command Line Options

- `--port`: Port to run the web server on (default: 3000)
- `--host`: Host to bind the web server to (default: 127.0.0.1)

### Examples

```bash
# Run on default port 3000
./pfsense_portal_generator

# Run on custom port
./pfsense_portal_generator --port 8080

# Server deployment (accessible from network)
./pfsense_portal_generator --host "0.0.0.0" --port 3000
```

### Web Interface

Once running, access these URLs:

- **Admin Panel**: `http://localhost:3000/admin` - Main management interface
- **All Vouchers**: `http://localhost:3000/vouchers` - View all vouchers across networks
- **Network Vouchers**: `http://localhost:3000/admin/networks/{id}/vouchers` - View vouchers for specific network
- **Generate Cards**: `http://localhost:3000/generate?network_id={id}` - Print voucher cards

## CSV Format

Your CSV file should contain voucher codes in the first column. The application will automatically detect and skip headers if present.

### Example CSV:
```csv
# WiFi Voucher Codes for Hotel Network
# Lines starting with # are ignored
voucher_code_a
voucher_code_b
voucher_code_c
```

## QR Code Details

The generated QR codes contain WiFi connection information in the standard format:
```
WIFI:T:WPA;S:YourSSID;P:YourPassword;H:false;;
```
