# Server Deployment Guide

This guide provides comprehensive instructions for deploying the WiFi Voucher Generator on a production server.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Configuration](#configuration)
- [Service Setup](#service-setup)
- [Reverse Proxy Configuration](#reverse-proxy-configuration)
- [Security Considerations](#security-considerations)
- [Firewall Configuration](#firewall-configuration)
- [SSL/TLS Setup](#ssltls-setup)
- [Monitoring and Logs](#monitoring-and-logs)
- [Backup and Recovery](#backup-and-recovery)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements
- **OS**: Ubuntu 20.04 LTS or CentOS 8+ (or compatible Linux distribution)
- **CPU**: 1 vCPU
- **RAM**: 512 MB
- **Storage**: 1 GB free space
- **Network**: Internet connection for dependency installation

### Recommended Requirements
- **OS**: Ubuntu 22.04 LTS
- **CPU**: 2 vCPUs
- **RAM**: 1 GB
- **Storage**: 5 GB free space
- **Network**: Dedicated IP address

## Installation

### Step 1: System Preparation

Update your system packages:

```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y

# CentOS/RHEL
sudo yum update -y
# or for newer versions:
sudo dnf update -y
```

Install required system packages:

```bash
# Ubuntu/Debian
sudo apt install -y curl wget git build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl wget git openssl-devel
# or for newer versions:
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl wget git openssl-devel
```

### Step 2: Install Rust

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

### Step 3: Create Application User

Create a dedicated user for the application:

```bash
sudo useradd --system --shell /bin/bash --home /opt/wifi-voucher --create-home wifi-voucher
sudo usermod -aG sudo wifi-voucher  # Optional: if you need sudo access
```

### Step 4: Download and Build Application

Switch to the application user:

```bash
sudo su - wifi-voucher
```

Clone and build the application:

```bash
# Clone the repository
git clone <repository-url> /opt/wifi-voucher/app
cd /opt/wifi-voucher/app

# Build the application
cargo build --release

# Create necessary directories
mkdir -p /opt/wifi-voucher/{logs,config,static}

# Copy the binary to a standard location
cp target/release/pfsense_portal_generator /opt/wifi-voucher/wifi-voucher-generator
```

### Step 5: Set Permissions

```bash
# Set ownership
sudo chown -R wifi-voucher:wifi-voucher /opt/wifi-voucher

# Set executable permissions
sudo chmod +x /opt/wifi-voucher/wifi-voucher-generator
```

## Configuration

### Application Configuration

Create a configuration script:

```bash
sudo nano /opt/wifi-voucher/config/start.sh
```

Add the following content:

```bash
#!/bin/bash

# WiFi Voucher Generator Startup Script
# Modify these variables as needed

# Server configuration
HOST="127.0.0.1"  # Use 0.0.0.0 to bind to all interfaces
PORT="3000"

# Optional: Default WiFi network (can be omitted)
# SSID="Your-Default-WiFi"
# PASSWORD="your-wifi-password"

# Application directory
APP_DIR="/opt/wifi-voucher"
BINARY="$APP_DIR/wifi-voucher-generator"
LOG_FILE="$APP_DIR/logs/wifi-voucher.log"

# Create log directory if it doesn't exist
mkdir -p "$APP_DIR/logs"

# Start the application
cd "$APP_DIR/app"

# Option 1: With default WiFi network
# exec "$BINARY" --ssid "$SSID" --password "$PASSWORD" --host "$HOST" --port "$PORT" >> "$LOG_FILE" 2>&1

# Option 2: Without default WiFi network (recommended for multi-network setups)
exec "$BINARY" --host "$HOST" --port "$PORT" >> "$LOG_FILE" 2>&1
```

Make the script executable:

```bash
sudo chmod +x /opt/wifi-voucher/config/start.sh
```

### Environment Variables (Alternative Configuration)

Create an environment file:

```bash
sudo nano /opt/wifi-voucher/config/wifi-voucher.env
```

```bash
# Server Configuration
WIFI_VOUCHER_HOST=127.0.0.1
WIFI_VOUCHER_PORT=3000

# Optional: Default Network
# WIFI_VOUCHER_SSID=Your-Default-WiFi
# WIFI_VOUCHER_PASSWORD=your-wifi-password

# Logging
RUST_LOG=info
```

## Service Setup

### SystemD Service Configuration

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/wifi-voucher.service
```

```ini
[Unit]
Description=WiFi Voucher Generator
After=network.target
Wants=network.target

[Service]
Type=exec
User=wifi-voucher
Group=wifi-voucher
WorkingDirectory=/opt/wifi-voucher/app
ExecStart=/opt/wifi-voucher/config/start.sh
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
PrivateDevices=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/wifi-voucher/logs

# Environment
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### Enable and Start the Service

```bash
# Reload systemd configuration
sudo systemctl daemon-reload

# Enable the service to start on boot
sudo systemctl enable wifi-voucher

# Start the service
sudo systemctl start wifi-voucher

# Check service status
sudo systemctl status wifi-voucher
```

### Service Management Commands

```bash
# Start the service
sudo systemctl start wifi-voucher

# Stop the service
sudo systemctl stop wifi-voucher

# Restart the service
sudo systemctl restart wifi-voucher

# Check service status
sudo systemctl status wifi-voucher

# View logs
sudo journalctl -u wifi-voucher -f

# View application logs
sudo tail -f /opt/wifi-voucher/logs/wifi-voucher.log
```

## Reverse Proxy Configuration

### Nginx Configuration

Install Nginx:

```bash
# Ubuntu/Debian
sudo apt install -y nginx

# CentOS/RHEL
sudo yum install -y nginx
# or
sudo dnf install -y nginx
```

Create Nginx configuration:

```bash
sudo nano /etc/nginx/sites-available/wifi-voucher
```

```nginx
server {
    listen 80;
    server_name your-domain.com www.your-domain.com;  # Replace with your domain

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=wifi_voucher:10m rate=10r/m;
    limit_req zone=wifi_voucher burst=20 nodelay;

    # File upload size (for CSV files)
    client_max_body_size 10M;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeout settings
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Static files (if any)
    location /static/ {
        alias /opt/wifi-voucher/app/static/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "OK";
        add_header Content-Type text/plain;
    }
}
```

Enable the site:

```bash
# Ubuntu/Debian
sudo ln -s /etc/nginx/sites-available/wifi-voucher /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default  # Remove default site

# CentOS/RHEL
sudo cp /etc/nginx/sites-available/wifi-voucher /etc/nginx/conf.d/wifi-voucher.conf
```

Test and reload Nginx:

```bash
sudo nginx -t
sudo systemctl enable nginx
sudo systemctl start nginx
sudo systemctl reload nginx
```

### Apache Configuration (Alternative)

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    ServerAlias www.your-domain.com

    ProxyPreserveHost On
    ProxyRequests Off
    ProxyPass / http://127.0.0.1:3000/
    ProxyPassReverse / http://127.0.0.1:3000/

    # Security headers
    Header always set X-Frame-Options "SAMEORIGIN"
    Header always set X-Content-Type-Options "nosniff"
    Header always set X-XSS-Protection "1; mode=block"

    # Logging
    ErrorLog ${APACHE_LOG_DIR}/wifi-voucher_error.log
    CustomLog ${APACHE_LOG_DIR}/wifi-voucher_access.log combined
</VirtualHost>
```

## Security Considerations

### Firewall Configuration

Configure UFW (Ubuntu) or firewalld (CentOS):

```bash
# Ubuntu (UFW)
sudo ufw enable
sudo ufw allow ssh
sudo ufw allow 'Nginx Full'
sudo ufw deny 3000  # Block direct access to application port

# CentOS (firewalld)
sudo systemctl enable firewalld
sudo systemctl start firewalld
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

### User Access Control

Restrict access to the admin panel by IP address (add to Nginx config):

```nginx
location /admin {
    # Restrict access to specific IP addresses
    allow 192.168.1.0/24;  # Local network
    allow 10.0.0.0/8;      # Private network
    deny all;
    
    proxy_pass http://127.0.0.1:3000;
    # ... other proxy settings
}
```

### File Permissions

Ensure proper file permissions:

```bash
sudo chmod 700 /opt/wifi-voucher/config
sudo chmod 600 /opt/wifi-voucher/config/wifi-voucher.env
sudo chmod 755 /opt/wifi-voucher/logs
```

## SSL/TLS Setup

### Using Let's Encrypt (Certbot)

Install Certbot:

```bash
# Ubuntu/Debian
sudo apt install -y certbot python3-certbot-nginx

# CentOS/RHEL
sudo yum install -y certbot python3-certbot-nginx
```

Obtain SSL certificate:

```bash
sudo certbot --nginx -d your-domain.com -d www.your-domain.com
```

Verify auto-renewal:

```bash
sudo certbot renew --dry-run
```

## Monitoring and Logs

### Log Rotation

Create log rotation configuration:

```bash
sudo nano /etc/logrotate.d/wifi-voucher
```

```
/opt/wifi-voucher/logs/*.log {
    weekly
    rotate 4
    compress
    delaycompress
    missingok
    notifempty
    create 644 wifi-voucher wifi-voucher
    postrotate
        systemctl restart wifi-voucher
    endscript
}
```

### Monitoring Script

Create a simple monitoring script:

```bash
sudo nano /opt/wifi-voucher/scripts/monitor.sh
```

```bash
#!/bin/bash

# WiFi Voucher Generator Monitoring Script

SERVICE="wifi-voucher"
URL="http://localhost:3000/"
LOG_FILE="/opt/wifi-voucher/logs/monitor.log"

# Check if service is running
if ! systemctl is-active --quiet $SERVICE; then
    echo "$(date): Service $SERVICE is not running, attempting restart" >> $LOG_FILE
    systemctl restart $SERVICE
    sleep 5
fi

# Check HTTP response
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" $URL)
if [ $HTTP_CODE -ne 200 ]; then
    echo "$(date): HTTP check failed with code $HTTP_CODE" >> $LOG_FILE
    systemctl restart $SERVICE
fi
```

Add to crontab:

```bash
sudo crontab -e
```

```
# Monitor WiFi Voucher Generator every 5 minutes
*/5 * * * * /opt/wifi-voucher/scripts/monitor.sh
```

## Backup and Recovery

### Backup Script

Since the application doesn't persist data between restarts, focus on backing up configuration:

```bash
sudo nano /opt/wifi-voucher/scripts/backup.sh
```

```bash
#!/bin/bash

BACKUP_DIR="/opt/wifi-voucher/backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="wifi-voucher-config-$DATE.tar.gz"

mkdir -p $BACKUP_DIR

# Backup configuration files
tar -czf $BACKUP_DIR/$BACKUP_FILE \
    /opt/wifi-voucher/config/ \
    /etc/systemd/system/wifi-voucher.service \
    /etc/nginx/sites-available/wifi-voucher \
    /opt/wifi-voucher/scripts/

# Keep only last 7 backups
find $BACKUP_DIR -name "wifi-voucher-config-*.tar.gz" -mtime +7 -delete

echo "Backup created: $BACKUP_FILE"
```

### Recovery Process

1. Restore configuration files from backup
2. Restart services:

```bash
sudo systemctl daemon-reload
sudo systemctl restart wifi-voucher
sudo systemctl restart nginx
```

## Troubleshooting

### Common Issues

#### Service Won't Start

Check logs:

```bash
sudo journalctl -u wifi-voucher -n 50
```

Check file permissions:

```bash
ls -la /opt/wifi-voucher/
```

#### High Memory Usage

Monitor memory usage:

```bash
ps aux | grep wifi-voucher
```

#### Port Already in Use

Check what's using the port:

```bash
sudo netstat -tlnp | grep :3000
```

#### SSL Certificate Issues

Test SSL configuration:

```bash
sudo certbot certificates
sudo nginx -t
```

### Log Locations

- Application logs: `/opt/wifi-voucher/logs/wifi-voucher.log`
- System logs: `sudo journalctl -u wifi-voucher`
- Nginx logs: `/var/log/nginx/access.log` and `/var/log/nginx/error.log`

### Performance Optimization

#### For High Traffic

1. Increase worker processes in Nginx:

```nginx
worker_processes auto;
worker_connections 1024;
```

2. Enable gzip compression:

```nginx
gzip on;
gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
```

3. Add caching for static assets:

```nginx
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

## Updates and Maintenance

### Updating the Application

1. Stop the service:
```bash
sudo systemctl stop wifi-voucher
```

2. Backup current version:
```bash
cp /opt/wifi-voucher/wifi-voucher-generator /opt/wifi-voucher/wifi-voucher-generator.backup
```

3. Update code and rebuild:
```bash
sudo su - wifi-voucher
cd /opt/wifi-voucher/app
git pull
cargo build --release
cp target/release/pfsense_portal_generator /opt/wifi-voucher/wifi-voucher-generator
exit
```

4. Restart service:
```bash
sudo systemctl start wifi-voucher
```

### Regular Maintenance

- Update system packages monthly
- Renew SSL certificates (automated with certbot)
- Review logs weekly
- Test backups monthly
- Monitor disk space usage

## Support

For additional support:

1. Check the application logs
2. Review system logs
3. Verify network connectivity
4. Check file permissions
5. Ensure all dependencies are installed

Remember to replace placeholder values (like `your-domain.com`, IP addresses, etc.) with your actual configuration values.