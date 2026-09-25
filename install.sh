#!/bin/sh
set -e

REPO="ZHOURA-24/zh_webterm"
URL="https://github.com/$REPO/releases/latest/download/zh_webterm-linux-amd64"

echo "Stopping existing service if running..."
if command -v systemctl >/dev/null 2>&1; then
    sudo systemctl stop zh_webterm 2>/dev/null || true
fi

echo "Downloading zh_webterm binary..."
TMP_BIN="/tmp/zh_webterm_bin"
curl -fsSL "$URL" -o "$TMP_BIN"
sudo rm -f /usr/local/bin/zh_webterm
sudo mv "$TMP_BIN" /usr/local/bin/zh_webterm
sudo chmod +x /usr/local/bin/zh_webterm

if command -v systemctl >/dev/null 2>&1; then
    echo "Setting up systemd service..."

    CONFIG_FILE="/etc/zh_webterm.env"
    NEW_INSTALL=0
    if [ ! -f "$CONFIG_FILE" ]; then
        GEN_PASSWORD=$(tr -dc 'A-Za-z0-9' </dev/urandom | head -c 16 2>/dev/null || openssl rand -hex 8 2>/dev/null || echo "zh_$(date +%s)")
        echo "Creating secure configuration at $CONFIG_FILE..."
        sudo tee "$CONFIG_FILE" > /dev/null << EOF
PORT=2424
WEBTERM_PASSWORD=$GEN_PASSWORD
EOF
        sudo chmod 600 "$CONFIG_FILE"
        NEW_INSTALL=1
    fi

    TARGET_USER="${SUDO_USER:-$USER}"
    TARGET_HOME=$(getent passwd "$TARGET_USER" 2>/dev/null | cut -d: -f6)
    TARGET_HOME="${TARGET_HOME:-/root}"

    sudo tee /etc/systemd/system/zh_webterm.service > /dev/null << EOF
[Unit]
Description=zh_webterm service
After=network.target

[Service]
Type=simple
User=$TARGET_USER
WorkingDirectory=$TARGET_HOME
EnvironmentFile=-$CONFIG_FILE
ExecStart=/usr/local/bin/zh_webterm
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable --now zh_webterm
    echo "✅ zh_webterm service enabled for user '$TARGET_USER' on http://localhost:2424"
    if [ "$NEW_INSTALL" = "1" ]; then
        echo "🔑 Initial Generated Password: $GEN_PASSWORD"
    fi
    echo "💡 Config file: $CONFIG_FILE (edit and run 'sudo systemctl restart zh_webterm' to customize)"
else
    echo "✅ zh_webterm installed to /usr/local/bin/zh_webterm"
    echo "Run: zh_webterm"
fi