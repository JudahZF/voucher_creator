# Cross-Compilation Guide: Building pfSense Portal Generator on macOS for Ubuntu Server

This guide explains how to cross-compile the pfSense Portal Generator from macOS and deploy the binary to an Ubuntu server.

## Prerequisites on macOS

- macOS 10.15 (Catalina) or newer
- Xcode Command Line Tools
- Homebrew package manager
- Rust and Cargo installed

## Step 1: Install Required Tools on macOS

First, ensure you have the Xcode Command Line Tools:

```bash
xcode-select --install
```

Install Homebrew if you don't have it already:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Install Rust and Cargo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Step 2: Install Cross-Compilation Tools

Install the cross-compilation target for Linux:

```bash
rustup target add x86_64-unknown-linux-gnu
```

Install additional tools needed for cross-compilation:

```bash
brew install FiloSottile/musl-cross/musl-cross
```

## Step 3: Configure Cargo for Cross-Compilation

Create or edit `~/.cargo/config.toml` to specify the linker for the Linux target:

```bash
mkdir -p ~/.cargo
touch ~/.cargo/config.toml
```

Add the following content to `~/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-musl-gcc"
```

## Step 4: Clone the Repository

```bash
git clone https://github.com/yourusername/pfsense_portal_generator.git
cd pfsense_portal_generator
```

## Step 5: Cross-Compile the Application

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

This will create a binary in `target/x86_64-unknown-linux-gnu/release/pfsense_portal_generator`.

## Step 6: Prepare the Deployment Package

Create a deployment package containing the binary and required files:

```bash
mkdir -p deployment/templates
cp target/x86_64-unknown-linux-gnu/release/pfsense_portal_generator deployment/
cp -r templates/* deployment/templates/
cp config.toml deployment/
```

Create a basic configuration file for the server environment:

```bash
cat > deployment/config.production.toml << EOF
# WiFi Voucher Generator Configuration

# Path to the templates directory (relative to executable location)
templates_dir = "templates"

# Path for the database file
database_path = "vouchers.db"

# Server configuration
[server]
default_host = "0.0.0.0"
default_port = 8080
EOF
```

Compress the deployment package:

```bash
tar -czf pfsense-portal-deployment.tar.gz -C deployment .
```

## Step 7: Transfer to Ubuntu Server

Use `scp` to transfer the deployment package to your Ubuntu server:

```bash
scp pfsense-portal-deployment.tar.gz username@your-server-ip:/path/to/destination/
```

## Step 8: Server Setup

SSH into your server:

```bash
ssh username@your-server-ip
```

Create a directory for the application:

```bash
sudo mkdir -p /opt/pfsense-portal
cd /opt/pfsense-portal
```

Extract the deployment package:

```bash
sudo tar -xzf /path/to/destination/pfsense-portal-deployment.tar.gz -C /opt/pfsense-portal
```

Install required runtime dependencies:

```bash
sudo apt update
sudo apt install -y libssl-dev sqlite3 libsqlite3-dev
```

Set appropriate permissions:

```bash
sudo chown -R www-data:www-data /opt/pfsense-portal
sudo chmod +x /opt/pfsense-portal/pfsense_portal_generator
```

## Step 9: Create a Systemd Service

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/pfsense-portal.service
```

Add the following content:

```
[Unit]
Description=pfSense Portal Generator
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/pfsense-portal
ExecStart=/opt/pfsense-portal/pfsense_portal_generator --port 8080 --host 0.0.0.0
Restart=on-failure
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl enable pfsense-portal.service
sudo systemctl start pfsense-portal.service
```

## Step 10: Set Up a Reverse Proxy (Optional but Recommended)

Install and configure Nginx:

```bash
sudo apt install -y nginx

sudo nano /etc/nginx/sites-available/pfsense-portal
```

Add the following configuration:

```
server {
    listen 80;
    server_name your-domain-or-ip;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Enable the site and restart Nginx:

```bash
sudo ln -s /etc/nginx/sites-available/pfsense-portal /etc/nginx/sites-enabled/
sudo systemctl restart nginx
```

## Step 11: Set Up SSL with Let's Encrypt (Optional)

```bash
sudo apt install -y certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

## Troubleshooting Common Cross-Compilation Issues

### SQLite Issues

If you encounter SQLite-related errors, you may need to statically link SQLite:

1. Add the following to your `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"], default-features = false }
```

2. Rebuild with the static SQLite feature:

```bash
cargo build --release --target x86_64-unknown-linux-gnu --features sqlx/sqlite-static
```

### OpenSSL Issues

If you encounter OpenSSL-related errors, try using the `openssl` crate with vendored features:

1. Add the following to your `Cargo.toml`:

```toml
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

2. Rebuild the application.

### Library Mismatch Issues

If the deployed binary fails to run due to library mismatches, you can try building a fully static binary:

```bash
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu
```

## Updating the Application

For future updates, repeat steps 5-7 to cross-compile and deploy new versions.

To update on the server:

```bash
sudo systemctl stop pfsense-portal.service
sudo tar -xzf /path/to/new/pfsense-portal-deployment.tar.gz -C /opt/pfsense-portal
sudo chown -R www-data:www-data /opt/pfsense-portal
sudo chmod +x /opt/pfsense-portal/pfsense_portal_generator
sudo systemctl start pfsense-portal.service
```

## Backup Considerations

The application's database file (`vouchers.db`) contains all your data. Set up regular backups:

```bash
sudo mkdir -p /var/backups/pfsense-portal
sudo cp /opt/pfsense-portal/vouchers.db /var/backups/pfsense-portal/vouchers.db.$(date +%Y%m%d)
```

Consider setting up a cron job for automated backups:

```bash
sudo crontab -e
```

Add:

```
0 3 * * * cp /opt/pfsense-portal/vouchers.db /var/backups/pfsense-portal/vouchers.db.$(date +\%Y\%m\%d)
```
