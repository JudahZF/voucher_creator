# Installation Guide for pfSense Portal Generator on Ubuntu Server

This guide explains how to install and configure the pfSense Portal Generator application on an Ubuntu server.

## Prerequisites

- Ubuntu 20.04 LTS or newer
- A user with sudo privileges
- Internet connectivity

## Step 1: Update System Packages

```bash
sudo apt update
sudo apt upgrade -y
```

## Step 2: Install Required Dependencies

```bash
sudo apt install -y build-essential pkg-config libssl-dev sqlite3 libsqlite3-dev curl
```

## Step 3: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen prompts to complete the installation. After installation, load Rust in your current shell:

```bash
source "$HOME/.cargo/env"
```

## Step 4: Clone the Repository

```bash
git clone https://github.com/yourusername/pfsense_portal_generator.git
cd pfsense_portal_generator
```

## Step 5: Build the Application

```bash
cargo build --release
```

This will create an optimized binary in the `target/release` directory.

## Step 6: Configure the Application

Create or edit the configuration file:

```bash
cp config.toml config.production.toml
nano config.production.toml
```

Modify the configuration as needed. At minimum, you should update the host to `0.0.0.0` to allow connections from other computers:

```toml
# WiFi Voucher Generator Configuration

# Path to the templates directory (relative to project root)
templates_dir = "templates"

# Path for the database file (absolute path recommended for production)
database_path = "/path/to/your/data/vouchers.db"

# Server configuration
[server]
default_host = "0.0.0.0"
default_port = 8080
```

## Step 7: Create a Systemd Service (Optional)

Create a systemd service file to run your application as a service:

```bash
sudo nano /etc/systemd/system/pfsense-portal.service
```

Add the following content (adjust the paths as needed):

```
[Unit]
Description=pfSense Portal Generator
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/path/to/pfsense_portal_generator
ExecStart=/path/to/pfsense_portal_generator/target/release/pfsense_portal_generator --port 8080 --host 0.0.0.0
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

## Step 8: Set Up a Reverse Proxy (Recommended)

For production use, setting up Nginx as a reverse proxy is recommended:

### Install Nginx

```bash
sudo apt install -y nginx
```

### Configure Nginx

Create a new site configuration:

```bash
sudo nano /etc/nginx/sites-available/pfsense-portal
```

Add the following configuration:

```
server {
    listen 80;
    server_name your-domain.com;

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

## Step 9: Set Up SSL with Let's Encrypt (Recommended)

```bash
sudo apt install -y certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

Follow the prompts to complete the SSL certificate installation.

## Step 10: Verify Installation

Open a web browser and navigate to your server's IP address or domain name:

```
http://your-server-ip:8080
```

or if using Nginx:

```
https://your-domain.com
```

## Troubleshooting

### Check Service Status

```bash
sudo systemctl status pfsense-portal.service
```

### View Application Logs

```bash
journalctl -u pfsense-portal.service
```

### Check Nginx Logs

```bash
sudo tail -f /var/log/nginx/error.log
sudo tail -f /var/log/nginx/access.log
```

## Security Recommendations

1. **Firewall Configuration**: Set up `ufw` to only allow necessary ports:
   ```bash
   sudo ufw allow 22/tcp
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   sudo ufw enable
   ```

2. **Database Permissions**: Ensure the database directory has appropriate permissions:
   ```bash
   sudo chown www-data:www-data /path/to/your/data/
   sudo chmod 700 /path/to/your/data/
   ```

3. **Regular Updates**: Keep your system and the application updated:
   ```bash
   sudo apt update && sudo apt upgrade -y
   git pull
   cargo build --release
   sudo systemctl restart pfsense-portal.service
   ```

## Backup Strategy

Regularly back up your database file:

```bash
sudo cp /path/to/your/data/vouchers.db /path/to/backup/vouchers.db.$(date +%Y%m%d)
```

Consider setting up a cron job for automated backups:

```bash
sudo crontab -e
```

Add:

```
0 3 * * * cp /path/to/your/data/vouchers.db /path/to/backup/vouchers.db.$(date +\%Y\%m\%d)
```
