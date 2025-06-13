#!/bin/bash

# WiFi Voucher Generator Automated Deployment Script
# This script automates the deployment of the WiFi Voucher Generator
# on a production server (Ubuntu/Debian or CentOS/RHEL).
#
# Usage:
#   1. Make the script executable: chmod +x deploy.sh
#   2. Run the script: sudo ./deploy.sh
#
# IMPORTANT:
#   - Run this script with root privileges (sudo).
#   - Ensure your system meets the minimum requirements.
#   - Replace 'your-domain.com' with your actual domain for Nginx and Certbot.
#   - Review and modify the configuration sections as needed before running.

set -e # Exit immediately if a command exits with a non-zero status

# --- Configuration Variables ---
APP_USER="wifi-voucher"
APP_GROUP="wifi-voucher"
APP_HOME="/opt/$APP_USER"
APP_DIR="$APP_HOME/app"
APP_BINARY_NAME="pfsense_portal_generator" # Name of the binary produced by cargo build
INSTALLED_BINARY_PATH="$APP_HOME/$APP_BINARY_NAME"
LOG_DIR="$APP_HOME/logs"
CONFIG_DIR="$APP_HOME/config"
STATIC_DIR="$APP_HOME/static"
SCRIPTS_DIR="$APP_HOME/scripts"
BACKUP_DIR="$APP_HOME/backups"

GIT_REPO_URL="https://github.com/your-username/pfsense_portal_generator.git" # <<< CHANGE THIS to your actual GitHub repository URL
DEFAULT_HOST="127.0.0.1" # Application host bind address (use 0.0.0.0 for direct access, but reverse proxy is recommended)
DEFAULT_PORT="3000"     # Application listening port

# Nginx/Apache configuration
USE_NGINX="true" # Set to "false" to use Apache or no reverse proxy
YOUR_DOMAIN="your-domain.com" # <<< CHANGE THIS to your actual domain name
ENABLE_SSL="true" # Set to "false" if you don't want to enable SSL immediately

# --- Functions ---

log_info() {
  echo -e "\n\e[1;32mINFO:\e[0m $1"
}

log_warn() {
  echo -e "\n\e[1;33mWARN:\e[0m $1"
}

log_error() {
  echo -e "\n\e[1;31mERROR:\e[0m $1"
  exit 1
}

detect_os() {
  if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_NAME=$ID
    OS_VERSION=$VERSION_ID
  elif type lsb_release >/dev/null 2>&1; then
    OS_NAME=$(lsb_release -si | tr '[:upper:]' '[:lower:]')
    OS_VERSION=$(lsb_release -sr)
  else
    OS_NAME=$(uname -s)
  fi

  case "$OS_NAME" in
    ubuntu|debian)
      PACKAGE_MANAGER="apt"
      NGINX_CONF_PATH="/etc/nginx/sites-available"
      NGINX_LINK_PATH="/etc/nginx/sites-enabled"
      UFW_INSTALLED=$(command -v ufw)
      ;;
    centos|rhel|fedora)
      if command -v dnf >/dev/null 2>&1; then
        PACKAGE_MANAGER="dnf"
      else
        PACKAGE_MANAGER="yum"
      fi
      NGINX_CONF_PATH="/etc/nginx/conf.d"
      FIREWALLD_INSTALLED=$(command -v firewall-cmd)
      ;;
    *)
      log_error "Unsupported OS: $OS_NAME. This script supports Ubuntu/Debian and CentOS/RHEL."
      ;;
  esac
  log_info "Detected OS: $OS_NAME $OS_VERSION with package manager: $PACKAGE_MANAGER"
}

# Function to install packages based on detected OS
install_packages() {
  log_info "Updating system packages..."
  if [ "$PACKAGE_MANAGER" == "apt" ]; then
    sudo apt update -y || log_error "Failed to update apt packages."
    sudo apt upgrade -y || log_warn "Failed to upgrade apt packages, continuing..."
    log_info "Installing required build and system packages for Ubuntu/Debian..."
    sudo apt install -y curl wget git build-essential pkg-config libssl-dev || log_error "Failed to install required apt packages."
  elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
    sudo "$PACKAGE_MANAGER" update -y || log_error "Failed to update $PACKAGE_MANAGER packages."
    log_info "Installing required build and system packages for CentOS/RHEL..."
    sudo "$PACKAGE_MANAGER" groupinstall -y "Development Tools" || log_error "Failed to install 'Development Tools' group."
    sudo "$PACKAGE_MANAGER" install -y curl wget git openssl-devel || log_error "Failed to install required $PACKAGE_MANAGER packages."
  fi
}

install_rust() {
  log_info "Installing Rust via rustup..."
  if command -v cargo &>/dev/null; then
    log_info "Rust is already installed."
  else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || log_error "Failed to download rustup script."
    source "$HOME/.cargo/env" || log_error "Failed to source Cargo environment. Please log out and back in, or run 'source ~/.cargo/env' manually."
    log_info "Rust installed successfully."
  fi

  if ! command -v rustc &>/dev/null || ! command -v cargo &>/dev/null; then
    log_error "Rust or Cargo not found after installation. Ensure '$HOME/.cargo/bin' is in your PATH."
  fi
  rustc --version
  cargo --version
}

create_app_user() {
  log_info "Creating dedicated application user '$APP_USER'..."
  if id "$APP_USER" &>/dev/null; then
    log_warn "User '$APP_USER' already exists."
  else
    sudo useradd --system --shell /bin/bash --home "$APP_HOME" --create-home "$APP_USER" || log_error "Failed to create user $APP_USER."
    log_info "User '$APP_USER' created successfully."
  fi

  # Ensure the app home directory exists and is owned by the user
  sudo mkdir -p "$APP_HOME"
  sudo chown -R "$APP_USER:$APP_GROUP" "$APP_HOME"
  log_info "Ensured $APP_HOME directory exists and is owned by $APP_USER."
}

download_and_build() {
  log_info "Switching to application user '$APP_USER' and cloning/building the application..."
  # Use sudo -H -u to ensure HOME is set correctly for the user
  sudo -H -u "$APP_USER" bash -c "
    set -e
    if [ -d \"$APP_DIR\" ]; then
      log_warn \"Application directory $APP_DIR already exists. Pulling latest code.\"
      cd \"$APP_DIR\"
      git pull || log_error \"Failed to pull latest code from $GIT_REPO_URL.\"
    else
      log_info \"Cloning repository from $GIT_REPO_URL to $APP_DIR.\"
      git clone \"$GIT_REPO_URL\" \"$APP_DIR\" || log_error \"Failed to clone repository $GIT_REPO_URL.\"
      cd \"$APP_DIR\"
    fi

    log_info \"Building the application in release mode... This may take a while.\"
    cargo build --release || log_error \"Failed to build the application.\"

    log_info \"Creating necessary application directories.\"
    mkdir -p \"$LOG_DIR\" \"$CONFIG_DIR\" \"$STATIC_DIR\" \"$SCRIPTS_DIR\" \"$BACKUP_DIR\"

    log_info \"Copying the binary to $INSTALLED_BINARY_PATH.\"
    cp target/release/$APP_BINARY_NAME \"$INSTALLED_BINARY_PATH\" || log_error \"Failed to copy binary.\"

    log_info \"Application built and binary copied successfully.\"
  " || log_error "Error during clone or build process as $APP_USER."

  log_info "Setting permissions for application files."
  sudo chown -R "$APP_USER:$APP_GROUP" "$APP_HOME" || log_error "Failed to set ownership on $APP_HOME."
  sudo chmod +x "$INSTALLED_BINARY_PATH" || log_error "Failed to set executable permissions on binary."
}

configure_app_startup() {
  log_info "Creating application startup script: $CONFIG_DIR/start.sh"
  sudo tee "$CONFIG_DIR/start.sh" >/dev/null <<EOL
#!/bin/bash

# WiFi Voucher Generator Startup Script
# Modify these variables as needed

# Server configuration
HOST="$DEFAULT_HOST"  # Use 0.0.0.0 to bind to all interfaces
PORT="$DEFAULT_PORT"

# Optional: Default WiFi network (can be omitted)
# SSID="Your-Default-WiFi"
# PASSWORD="your-wifi-password"

# Application directory
APP_HOME="$APP_HOME"
BINARY="$INSTALLED_BINARY_PATH"
LOG_FILE="$LOG_DIR/wifi-voucher.log"

# Create log directory if it doesn't exist
mkdir -p "\$LOG_DIR"

# Start the application
cd "\$APP_HOME/app"

# Option 1: With default WiFi network
# exec "\$BINARY" --ssid "\$SSID" --password "\$PASSWORD" --host "\$HOST" --port "\$PORT" >> "\$LOG_FILE" 2>&1

# Option 2: Without default WiFi network (recommended for multi-network setups)
exec "\$BINARY" --host "\$HOST" --port "\$PORT" >> "\$LOG_FILE" 2>&1
EOL
  sudo chmod +x "$CONFIG_DIR/start.sh" || log_error "Failed to make start.sh executable."
  sudo chown "$APP_USER:$APP_GROUP" "$CONFIG_DIR/start.sh" || log_error "Failed to set ownership of start.sh."

  log_info "Creating environment file: $CONFIG_DIR/wifi-voucher.env (Alternative configuration)"
  sudo tee "$CONFIG_DIR/wifi-voucher.env" >/dev/null <<EOL
# Server Configuration
WIFI_VOUCHER_HOST=$DEFAULT_HOST
WIFI_VOUCHER_PORT=$DEFAULT_PORT

# Optional: Default Network
# WIFI_VOUCHER_SSID=Your-Default-WiFi
# WIFI_VOUCHER_PASSWORD=your-wifi-password

# Logging
RUST_LOG=info
EOL
  sudo chmod 600 "$CONFIG_DIR/wifi-voucher.env" || log_error "Failed to set permissions for wifi-voucher.env."
  sudo chown "$APP_USER:$APP_GROUP" "$CONFIG_DIR/wifi-voucher.env" || log_error "Failed to set ownership of wifi-voucher.env."
}

setup_systemd_service() {
  log_info "Creating systemd service file: /etc/systemd/system/wifi-voucher.service"
  sudo tee "/etc/systemd/system/wifi-voucher.service" >/dev/null <<EOL
[Unit]
Description=WiFi Voucher Generator
After=network.target
Wants=network.target

[Service]
Type=exec
User=$APP_USER
Group=$APP_GROUP
WorkingDirectory=$APP_HOME/app
ExecStart=$CONFIG_DIR/start.sh
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
PrivateDevices=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$LOG_DIR

# Environment
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOL

  log_info "Reloading systemd, enabling and starting the service."
  sudo systemctl daemon-reload || log_error "Failed to reload systemd daemon."
  sudo systemctl enable wifi-voucher || log_error "Failed to enable wifi-voucher service."
  sudo systemctl start wifi-voucher || log_error "Failed to start wifi-voucher service."
  log_info "WiFi Voucher Generator service status:"
  sudo systemctl status wifi-voucher --no-pager
}

configure_reverse_proxy() {
  if [ "$USE_NGINX" == "true" ]; then
    log_info "Configuring Nginx reverse proxy."
    install_nginx
    create_nginx_config
    test_and_reload_nginx
  else
    log_warn "Nginx reverse proxy not enabled. You may need to manually configure Apache or another reverse proxy."
  fi
}

install_nginx() {
  log_info "Installing Nginx..."
  if [ "$PACKAGE_MANAGER" == "apt" ]; then
    sudo apt install -y nginx || log_error "Failed to install Nginx via apt."
  elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
    sudo "$PACKAGE_MANAGER" install -y nginx || log_error "Failed to install Nginx via $PACKAGE_MANAGER."
  fi
  sudo systemctl enable nginx || log_error "Failed to enable Nginx service."
  sudo systemctl start nginx || log_error "Failed to start Nginx service."
  log_info "Nginx installed and started."
}

create_nginx_config() {
  log_info "Creating Nginx configuration file: $NGINX_CONF_PATH/wifi-voucher"
  sudo tee "$NGINX_CONF_PATH/wifi-voucher" >/dev/null <<EOL
server {
    listen 80;
    server_name $YOUR_DOMAIN www.$YOUR_DOMAIN;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate limiting (optional, uncomment to enable)
    # limit_req_zone \$binary_remote_addr zone=wifi_voucher:10m rate=10r/m;
    # limit_req zone=wifi_voucher burst=20 nodelay;

    # File upload size (for CSV files)
    client_max_body_size 10M;

    location / {
        proxy_pass http://127.0.0.1:$DEFAULT_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;

        # Timeout settings
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Static files (if your app serves them from /static/)
    # Ensure this path matches where your application serves static files
    location /static/ {
        alias $APP_DIR/static/; # Assuming static files are in app_home/app/static
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # Health check endpoint (optional)
    location /health {
        access_log off;
        return 200 "OK";
        add_header Content-Type text/plain;
    }
}
EOL

  if [ "$PACKAGE_MANAGER" == "apt" ]; then
    log_info "Enabling Nginx site and removing default site for Ubuntu/Debian."
    sudo ln -sf "$NGINX_CONF_PATH/wifi-voucher" "$NGINX_LINK_PATH/" || log_error "Failed to link Nginx site."
    if [ -f "$NGINX_LINK_PATH/default" ]; then
      sudo rm "$NGINX_LINK_PATH/default" || log_warn "Failed to remove default Nginx site, continuing..."
    fi
  elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
    log_info "Copying Nginx config to conf.d for CentOS/RHEL."
    sudo cp "$NGINX_CONF_PATH/wifi-voucher" "$NGINX_CONF_PATH/wifi-voucher.conf" || log_error "Failed to copy Nginx config."
  fi
}

test_and_reload_nginx() {
  log_info "Testing Nginx configuration and reloading..."
  sudo nginx -t || log_error "Nginx configuration test failed."
  sudo systemctl reload nginx || log_error "Failed to reload Nginx service."
  log_info "Nginx reloaded successfully."
}

configure_firewall() {
  log_info "Configuring firewall..."
  if [ "$PACKAGE_MANAGER" == "apt" ]; then
    if [ -n "$UFW_INSTALLED" ]; then
      log_info "Using UFW for firewall configuration."
      sudo ufw enable || log_error "Failed to enable UFW."
      sudo ufw allow ssh || log_error "Failed to allow SSH in UFW."
      sudo ufw allow 'Nginx Full' || log_error "Failed to allow Nginx Full profile in UFW."
      sudo ufw deny "$DEFAULT_PORT" || log_info "Blocked direct access to application port $DEFAULT_PORT via UFW."
      log_info "UFW status:"
      sudo ufw status verbose
    else
      log_warn "UFW not found. Please install and configure a firewall manually."
    fi
  elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
    if [ -n "$FIREWALLD_INSTALLED" ]; then
      log_info "Using firewalld for firewall configuration."
      sudo systemctl enable firewalld || log_error "Failed to enable firewalld."
      sudo systemctl start firewalld || log_error "Failed to start firewalld."
      sudo firewall-cmd --permanent --add-service=ssh || log_warn "Failed to add SSH service to firewalld, continuing..."
      sudo firewall-cmd --permanent --add-service=http || log_warn "Failed to add HTTP service to firewalld, continuing..."
      sudo firewall-cmd --permanent --add-service=https || log_warn "Failed to add HTTPS service to firewalld, continuing..."
      sudo firewall-cmd --permanent --add-port="$DEFAULT_PORT/tcp" --zone=public || log_warn "Failed to open port $DEFAULT_PORT, continuing..." # Allow internal communication if 127.0.0.1
      sudo firewall-cmd --reload || log_error "Failed to reload firewalld."
      log_info "firewalld status:"
      sudo firewall-cmd --list-all
    else
      log_warn "firewalld not found. Please install and configure a firewall manually."
    fi
  fi
}

setup_ssl() {
  if [ "$ENABLE_SSL" == "true" ] && [ "$USE_NGINX" == "true" ]; then
    log_info "Setting up SSL/TLS with Let's Encrypt Certbot."
    log_info "Installing Certbot and Nginx plugin..."
    if [ "$PACKAGE_MANAGER" == "apt" ]; then
      sudo apt install -y certbot python3-certbot-nginx || log_error "Failed to install Certbot via apt."
    elif [ "$PACKAGE_MANAGER" == "yum" ] || [ "$PACKAGE_MANAGER" == "dnf" ]; then
      sudo "$PACKAGE_MANAGER" install -y certbot python3-certbot-nginx || log_error "Failed to install Certbot via $PACKAGE_MANAGER."
    fi

    log_info "Obtaining SSL certificate for $YOUR_DOMAIN using Certbot..."
    # Attempt to obtain certificate. Certbot will automatically modify Nginx config.
    sudo certbot --nginx -d "$YOUR_DOMAIN" -d "www.$YOUR_DOMAIN" --agree-tos --redirect --email "your_email@example.com" --no-eff-email || log_error "Failed to obtain SSL certificate. Check Certbot output for errors."
    log_info "SSL certificate obtained successfully."

    log_info "Verifying Certbot auto-renewal configuration..."
    sudo certbot renew --dry-run || log_warn "Certbot dry run failed. Auto-renewal might not work correctly. Please check manually."
  else
    log_warn "SSL/TLS setup skipped. (ENABLE_SSL is false or Nginx is not used)."
  fi
}

configure_log_rotation() {
  log_info "Creating log rotation configuration for application logs."
  sudo tee "/etc/logrotate.d/wifi-voucher" >/dev/null <<EOL
$LOG_DIR/*.log {
    weekly
    rotate 4
    compress
    delaycompress
    missingok
    notifempty
    create 644 $APP_USER $APP_GROUP
    postrotate
        systemctl restart wifi-voucher
    endscript
}
EOL
  sudo chown root:root "/etc/logrotate.d/wifi-voucher"
  sudo chmod 644 "/etc/logrotate.d/wifi-voucher"
  log_info "Log rotation configured."
}

create_monitoring_script() {
  log_info "Creating monitoring script: $SCRIPTS_DIR/monitor.sh"
  sudo tee "$SCRIPTS_DIR/monitor.sh" >/dev/null <<EOL
#!/bin/bash

# WiFi Voucher Generator Monitoring Script

SERVICE="wifi-voucher"
URL="http://127.0.0.1:$DEFAULT_PORT/health" # Using localhost as the health check URL
LOG_FILE="$LOG_DIR/monitor.log"

# Check if service is running
if ! systemctl is-active --quiet \$SERVICE; then
    echo "\$(date): Service \$SERVICE is not running, attempting restart" >> \$LOG_FILE
    systemctl restart \$SERVICE
    sleep 5
fi

# Check HTTP response (if /health endpoint is implemented and accessible)
HTTP_CODE=\$(curl -s -o /dev/null -w "%{http_code}" \$URL)
if [ \$HTTP_CODE -ne 200 ]; then
    echo "\$(date): HTTP check failed with code \$HTTP_CODE for \$URL" >> \$LOG_FILE
    systemctl restart \$SERVICE
fi
EOL
  sudo chmod +x "$SCRIPTS_DIR/monitor.sh" || log_error "Failed to make monitor.sh executable."
  sudo chown "$APP_USER:$APP_GROUP" "$SCRIPTS_DIR/monitor.sh" || log_error "Failed to set ownership for monitor.sh."

  log_info "Adding monitoring script to root crontab."
  # Add to root's crontab for periodic checks
  (sudo crontab -l 2>/dev/null; echo "*/5 * * * * $SCRIPTS_DIR/monitor.sh") | sudo crontab - || log_error "Failed to add cron job."
  log_info "Monitoring script added to crontab to run every 5 minutes."
}

create_backup_script() {
  log_info "Creating backup script: $SCRIPTS_DIR/backup.sh"
  sudo tee "$SCRIPTS_DIR/backup.sh" >/dev/null <<EOL
#!/bin/bash

BACKUP_DIR="$BACKUP_DIR"
DATE=\$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="wifi-voucher-config-\$DATE.tar.gz"

mkdir -p \$BACKUP_DIR

# Backup configuration files and service definitions
tar -czf \$BACKUP_DIR/\$BACKUP_FILE \\
    "$CONFIG_DIR/" \\
    "/etc/systemd/system/wifi-voucher.service" \\
    "/etc/nginx/sites-available/wifi-voucher" \\
    "$SCRIPTS_DIR/"

# Keep only last 7 backups
find \$BACKUP_DIR -name "wifi-voucher-config-*.tar.gz" -mtime +7 -delete

echo "Backup created: \$BACKUP_FILE"
EOL
  sudo chmod +x "$SCRIPTS_DIR/backup.sh" || log_error "Failed to make backup.sh executable."
  sudo chown "$APP_USER:$APP_GROUP" "$SCRIPTS_DIR/backup.sh" || log_error "Failed to set ownership for backup.sh."

  log_info "Adding backup script to root crontab for daily execution."
  # Add to root's crontab for daily backups
  (sudo crontab -l 2>/dev/null; echo "0 2 * * * $SCRIPTS_DIR/backup.sh") | sudo crontab - || log_error "Failed to add backup cron job."
  log_info "Backup script added to crontab to run daily at 2 AM."
}

set_final_permissions() {
  log_info "Setting final file permissions."
  sudo chmod 700 "$CONFIG_DIR" || log_error "Failed to set permissions on config directory."
  sudo chmod 600 "$CONFIG_DIR/wifi-voucher.env" || log_error "Failed to set permissions on env file."
  sudo chmod 755 "$LOG_DIR" || log_error "Failed to set permissions on logs directory."
  sudo chmod 755 "$SCRIPTS_DIR" || log_error "Failed to set permissions on scripts directory."
  sudo chmod 700 "$BACKUP_DIR" || log_error "Failed to set permissions on backup directory."
  sudo chown -R "$APP_USER:$APP_GROUP" "$APP_HOME" || log_error "Failed to set recursive ownership."
}

# --- Main Script Execution ---

log_info "Starting WiFi Voucher Generator Deployment Script..."

if [ "$EUID" -ne 0 ]; then
  log_error "Please run this script with sudo: sudo ./deploy.sh"
fi

# Detect OS and package manager
detect_os

# Step 1: System Preparation
install_packages

# Step 2: Install Rust
install_rust

# Step 3: Create Application User
create_app_user

# Step 4: Download and Build Application
download_and_build

# Step 5: Configuration
configure_app_startup

# Step 6: Service Setup
setup_systemd_service

# Step 7: Reverse Proxy Configuration (Nginx)
configure_reverse_proxy

# Step 8: Security Considerations (Firewall)
configure_firewall

# Step 9: SSL/TLS Setup
setup_ssl

# Step 10: Monitoring and Logs
configure_log_rotation
create_monitoring_script

# Step 11: Backup and Recovery
create_backup_script

# Step 12: Final Permissions
set_final_permissions

log_info "Deployment complete!"
log_info "You should now be able to access your WiFi Voucher Generator at: http://$YOUR_DOMAIN"
if [ "$ENABLE_SSL" == "true" ] && [ "$USE_NGINX" == "true" ]; then
  log_info "And securely at: https://$YOUR_DOMAIN"
fi

log_info "Please remember to replace 'your_email@example.com' in the Certbot command if you enabled SSL."
log_info "Also, update '$GIT_REPO_URL' to your actual repository."
log_info "Review the configuration files in $CONFIG_DIR for further customization."
log_info "For troubleshooting, check service status with 'sudo systemctl status wifi-voucher' and logs with 'sudo journalctl -u wifi-voucher -f'."
