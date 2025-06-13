# Quick Start Guide

Get the WiFi Voucher Generator running in minutes!

## 🚀 Quick Start

### 1. Build the Application

```bash
git clone <repository-url>
cd pfsense_portal_generator
cargo build --release
```

### 2. Choose Your Setup

#### Option A: Admin Panel Only (Recommended)
Start without any default network - manage everything through the web interface:

```bash
./target/release/pfsense_portal_generator
```

Then open: http://localhost:3000/admin

#### Option B: With Default Network
Start with a default WiFi network:

```bash
./target/release/pfsense_portal_generator --ssid "Guest-WiFi" --password "welcome123"
```

Then open: http://localhost:3000

### 3. Create Your First Network

1. Go to the Admin Panel: http://localhost:3000/admin
2. Fill out the "Create New WiFi Network" form:
   - **Network Name**: "Hotel Guest WiFi" (descriptive name)
   - **SSID**: "Hotel-Guest" (actual WiFi network name)
   - **Password**: "your-wifi-password"
   - **Description**: "WiFi for hotel guests" (optional)
3. Click "Create Network"

### 4. Upload Voucher Codes

1. Create a CSV file with voucher codes:
   ```csv
   GUEST-001
   GUEST-002
   GUEST-003
   HOTEL-VIP-001
   HOTEL-VIP-002
   ```

2. In the Admin Panel:
   - Select your network from the dropdown
   - Choose your CSV file
   - Click "Upload Vouchers"

### 5. Generate Vouchers

1. Click "Generate" next to your network in the admin panel
2. Print the voucher cards (optimized for 8+ vouchers per page)
3. Each voucher includes:
   - QR code for instant WiFi connection
   - Individual voucher code for authentication
   - Network information

## 📱 How It Works

1. **Guest scans QR code** → Automatically connects to WiFi
2. **Guest enters voucher code** → Gets internet access
3. **Voucher is tracked** → Prevents reuse (if implemented in your system)

## 🔧 Common Commands

```bash
# Basic usage (admin panel only)
./pfsense_portal_generator

# With default network
./pfsense_portal_generator --ssid "WiFi-Name" --password "wifi-password"

# Custom port
./pfsense_portal_generator --port 8080

# Server deployment (accessible from network)
./pfsense_portal_generator --host "0.0.0.0" --port 3000

# Help
./pfsense_portal_generator --help
```

## 📁 Example CSV Files

Check the `examples/` directory for sample CSV files:
- `vouchers-simple.csv` - Basic voucher codes
- `hotel-vouchers.csv` - Hotel-specific codes with comments

## 🌐 URLs

- **Main Page**: http://localhost:3000
- **Admin Panel**: http://localhost:3000/admin
- **Generate Vouchers**: http://localhost:3000/generate

## ⚡ Advanced Setup

### Multiple Networks Example

1. **Guest Network**
   - Name: "Guest WiFi"
   - SSID: "Hotel-Guest"
   - Codes: GUEST-001, GUEST-002, etc.

2. **VIP Network**
   - Name: "VIP WiFi"
   - SSID: "Hotel-VIP"
   - Codes: VIP-001, VIP-002, etc.

3. **Conference Network**
   - Name: "Conference WiFi"
   - SSID: "Conference-2024"
   - Codes: CONF-001, CONF-002, etc.

### CSV with Comments

```csv
# Hotel WiFi Codes - January 2024
# Guest Network Vouchers
GUEST-JAN-001
GUEST-JAN-002
# VIP Network Vouchers
VIP-JAN-001
VIP-JAN-002
```

## 🔒 Production Deployment

For server deployment, see:
- **Full Guide**: [SERVER_DEPLOYMENT.md](SERVER_DEPLOYMENT.md)
- **Examples**: [examples/README.md](examples/README.md)

## ❓ Need Help?

1. **Check logs**: Application prints status to console
2. **Verify network**: Ensure WiFi credentials are correct
3. **Test QR codes**: Use multiple QR scanner apps
4. **Check CSV format**: First column should contain voucher codes

## 🎯 Next Steps

1. **Customize**: Modify templates for your branding
2. **Scale**: Set up reverse proxy for production
3. **Secure**: Configure SSL certificates
4. **Monitor**: Set up logging and health checks

That's it! You're ready to generate professional WiFi vouchers with QR codes.