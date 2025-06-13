# WiFi Voucher Generator

A Rust-based web application that generates printable WiFi vouchers with QR codes. Create and manage multiple WiFi networks, upload CSV files with voucher codes, and generate beautiful, print-ready voucher cards that include:

- WiFi connection QR codes for instant network access
- Individual voucher codes for authentication
- Professional styling optimized for printing
- Multi-network support with admin panel
- 8+ vouchers per page for cost efficiency

Perfect for hotels, cafes, conferences, events, and any business that provides temporary WiFi access to multiple networks.

## Features

- 🎯 **Easy CSV Upload**: Simple drag-and-drop interface for uploading voucher codes
- 🔗 **WiFi QR Codes**: Automatically generates QR codes for instant WiFi connection
- 🎨 **Print-Ready Design**: Professional voucher cards optimized for printing (8+ per page)
- 🚀 **Fast & Lightweight**: Built with Rust for maximum performance
- 📱 **Responsive Web Interface**: Works on desktop and mobile devices
- 🔒 **Secure**: No data stored permanently, everything processed in memory
- 🛠️ **Admin Panel**: Manage multiple WiFi networks and organize vouchers
- 📝 **CSV Comments**: Support for comments in CSV files (lines starting with #)
- 🏢 **Multi-Network**: Create and manage different WiFi networks for different purposes
- 📊 **Network Organization**: Link vouchers to specific networks and print by network

## Installation

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Git

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

```bash
# With default WiFi network
./pfsense_portal_generator --ssid "YourWiFiNetwork" --password "YourWiFiPassword"

# Without default WiFi network (admin panel only)
./pfsense_portal_generator
```

#### Available Options:

- `--ssid, -s`: WiFi network name (SSID) - **Optional** (creates default network if provided)
- `--password, -p`: WiFi network password - **Required if SSID is provided** (for default network)
- `--port`: Port to run the web server on (default: 3000)
- `--host`: Host to bind the web server to (default: 127.0.0.1)

**Note**: The command line SSID/password creates a "Default Network" that appears in the admin panel. You can create additional networks through the web interface. If no default network is provided, you must use the admin panel to create all networks.

### Example

```bash
# Run with custom WiFi credentials (creates default network)
./pfsense_portal_generator --ssid "Hotel-Guest-WiFi" --password "welcome123"

# Run without default network (admin panel only)
./pfsense_portal_generator --port 3000

# Run on a different port with default network
./pfsense_portal_generator --ssid "Cafe-WiFi" --password "coffee2024" --port 8080

# Run accessible from other devices on the network
./pfsense_portal_generator --ssid "Event-WiFi" --password "conference2024" --host "0.0.0.0" --port 3000

# Server deployment (no default network, accessible from network)
./pfsense_portal_generator --host "0.0.0.0" --port 3000
```

## CSV Format

Your CSV file should contain voucher codes in the first column. The application will automatically detect and skip headers if present.

### Example CSV:
```csv
# WiFi Voucher Codes for Hotel Network
# Lines starting with # are ignored
voucher_code
# Room access codes
ROOM-101-WIFI
ROOM-102-WIFI
ROOM-103-WIFI
# Guest access codes
GUEST-123-XYZ
DAILY-PASS-001
```

### Simple Format (no headers):
```csv
# Comments are supported for organization
WIFI001-ABC
WIFI002-DEF
WIFI003-GHI
# More voucher codes below
GUEST-123-XYZ
DAILY-PASS-001
```

### Network-Specific Organization:
You can use comments to organize vouchers by purpose, then upload them to the appropriate network in the admin panel:
```csv
# Conference WiFi Codes - Speaker Access
SPEAKER-001-CONF
SPEAKER-002-CONF
# Conference WiFi Codes - Attendee Access  
ATTEND-001-CONF
ATTEND-002-CONF
```

## How It Works

### Basic Workflow
1. **Start the Application**: Run the executable with your WiFi credentials
2. **Open Web Interface**: Navigate to `http://localhost:3000` (or your configured host/port)
3. **Upload CSV**: Use the drag-and-drop interface to upload your voucher codes
4. **Generate Vouchers**: Click "Generate QR Codes" to create printable vouchers
5. **Print**: Use your browser's print function to print the voucher cards

### Admin Panel Workflow
1. **Access Admin Panel**: Click "Admin Panel" from the main page or go to `/admin`
2. **Create Networks**: Add new WiFi networks with different SSIDs and passwords
3. **Upload Vouchers**: Upload CSV files and assign them to specific networks
4. **Manage Networks**: View, edit, and delete networks and their associated vouchers
5. **Generate by Network**: Print vouchers for specific networks only

## Web Interface

The application provides a clean, modern web interface with:

- **Main Page**: Drag-and-drop CSV file upload with validation
- **Admin Panel**: Complete network and voucher management interface
- **Network Management**: Create, edit, and delete WiFi networks
- **Voucher Organization**: Upload and assign vouchers to specific networks
- **Network Voucher View**: Review vouchers for individual networks
- **Generated Vouchers**: Print-ready voucher cards with QR codes (8+ per page)
- **Print Optimization**: CSS optimized for clean, high-density printing

## QR Code Details

The generated QR codes contain WiFi connection information in the standard format:
```
WIFI:T:WPA;S:YourSSID;P:YourPassword;H:false;;
```

When scanned with a smartphone:
- **Android**: Automatically prompts to connect to the WiFi network
- **iOS**: Opens WiFi settings with network pre-filled

## Development

### Running in Development Mode

```bash
cargo run -- --ssid "TestNetwork" --password "testpassword"
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs           # Main application and web server
├── qr_generator.rs   # QR code generation logic
├── voucher.rs        # Voucher management and CSV processing
├── wifi_network.rs   # WiFi network management
└── templates.rs      # HTML templates for the web interface
```

## Dependencies

- **axum**: Web framework
- **tokio**: Async runtime
- **qrcode**: QR code generation
- **image**: Image processing
- **csv**: CSV file parsing
- **serde**: Serialization/deserialization
- **clap**: Command line argument parsing
- **chrono**: Date/time handling
- **base64**: Base64 encoding for embedded images

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Use a different port
   ./pfsense_portal_generator --ssid "WiFi" --password "pass" --port 8080
   ```

2. **CSV Not Processing**
   - Ensure your CSV has voucher codes in the first column
   - Check that the file is properly formatted UTF-8 text
   - Verify there are no empty rows at the beginning

3. **QR Codes Not Working**
   - Verify your WiFi credentials are correct
   - Ensure the QR code is large enough to scan (150x150px minimum)
   - Test with different QR code scanner apps

### Performance Tips

- For best performance, keep CSV files under 10,000 vouchers per network
- The application processes everything in memory for security
- Use a modern browser for the best web interface experience
- Organize vouchers by network for better management and faster printing
- The compact 8+ vouchers per page design reduces printing time and paper usage

## Security Considerations

- Voucher codes are processed in memory only
- No permanent storage of sensitive information
- WiFi credentials are only used for QR code generation
- Run on localhost by default for security
- Admin panel allows network management but doesn't persist data between restarts
- Each network's credentials are isolated and only used for their specific QR codes

## Server Deployment

For production server deployment, including reverse proxy setup, SSL configuration, and systemd service configuration, see the [Server Deployment Guide](SERVER_DEPLOYMENT.md).

Key features for server deployment:
- **No default network required**: Run without command line WiFi parameters
- **Admin panel management**: Create and manage all networks through the web interface
- **Reverse proxy compatible**: Works with Nginx, Apache, and other reverse proxies
- **Systemd service**: Includes complete service configuration
- **Security hardening**: Comprehensive security recommendations

## License

This project is open source. Please check the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Support

For issues and questions, please create an issue in the project repository.