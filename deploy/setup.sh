#!/bin/bash
# setup.sh - Iron Cage deployment automation

set -e

echo "[*] Setting up Iron Cage services..."

# Create system user
if ! id "ironcage" &>/dev/null; then
    echo "[*] Creating ironcage system user..."
    sudo useradd --system --shell /usr/sbin/nologin ironcage
fi

# Create necessary directories
echo "[*] Creating directories..."
sudo mkdir -p /var/lib/iron_cage
sudo mkdir -p /etc/iron_cage
sudo mkdir -p /etc/nginx/ssl

# Copy systemd units
echo "[*] Installing systemd services..."
sudo cp deploy/systemd/iron_control_api.service /etc/systemd/system/
sudo cp deploy/systemd/iron_server_proxy.service /etc/systemd/system/

# Copy Nginx config
if [ -d "/etc/nginx/sites-available" ]; then
    echo "[*] Configuring Nginx..."
    sudo cp deploy/nginx/iron_cage.conf /etc/nginx/sites-available/
    sudo ln -sf /etc/nginx/sites-available/iron_cage.conf /etc/nginx/sites-enabled/
fi

# Copy Environment examples (if not exist)
echo "[*] Preparing environment files..."
if [ ! -f "/etc/iron_cage/iron_control_api.env" ]; then
    sudo cp deploy/env/iron_control_api.env.example /etc/iron_cage/iron_control_api.env
fi
if [ ! -f "/etc/iron_cage/iron_server_proxy.env" ]; then
    sudo cp deploy/env/iron_server_proxy.env.example /etc/iron_cage/iron_server_proxy.env
fi

# Validate that placeholder keys are not left in production env files
for env_file in /etc/iron_cage/iron_control_api.env /etc/iron_cage/iron_server_proxy.env; do
    if [ -f "$env_file" ]; then
        if grep -q "^IP_TOKEN_KEY=0\{64\}" "$env_file"; then
            echo "[ERROR] Placeholder IP_TOKEN_KEY detected in $env_file"
            echo "        Replace with a real key: openssl rand -hex 32"
            exit 1
        fi
        if grep -q "^IRON_SECRETS_MASTER_KEY=CHANGE_ME" "$env_file"; then
            echo "[ERROR] Placeholder IRON_SECRETS_MASTER_KEY detected in $env_file"
            echo "        Replace with a real key: openssl rand -base64 32"
            exit 1
        fi
    fi
done

# Set permissions
echo "[*] Setting permissions..."
sudo chown -R ironcage:ironcage /var/lib/iron_cage
sudo chown -R ironcage:ironcage /etc/iron_cage
sudo chmod 640 /etc/iron_cage/iron_control_api.env /etc/iron_cage/iron_server_proxy.env

# Reload and enable services
echo "[*] Activating services..."
sudo systemctl daemon-reload
sudo systemctl enable iron_control_api
sudo systemctl enable iron_server_proxy

echo "[+] Setup complete."
echo "[!] Configure your .env files in /etc/iron_cage/ and add SSL certs to /etc/nginx/ssl/"
