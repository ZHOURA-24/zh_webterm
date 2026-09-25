# zh_webterm

---

## Installation

### 🐧 Linux
```bash
curl -fsSL https://raw.githubusercontent.com/ZHOURA-24/zh_webterm/main/install.sh | bash
```

### 🪟 Windows Powershell
```powershell
irm https://raw.githubusercontent.com/ZHOURA-24/zh_webterm/main/install.ps1 | iex
```

### 📦 Build from Source (Cargo)
```bash
git clone https://github.com/ZHOURA-24/zh_webterm.git
cd zh_webterm
cargo install --path .
```

---

## Configuration

`zh_webterm` natively loads configuration from `.env` / `zh_webterm.env`.

### 🐧 Linux Configuration
1. Edit `/etc/zh_webterm.env`
2. Restart service:
```bash
sudo systemctl restart zh_webterm
```

### 🪟 Windows Configuration
1. Edit `%LOCALAPPDATA%\Programs\zh_webterm\zh_webterm.env`:
```powershell
notepad "$env:LOCALAPPDATA\Programs\zh_webterm\zh_webterm.env"
```
2. Restart background process:
```powershell
Stop-Process -Name "zh_webterm" -ErrorAction SilentlyContinue
Start-Process "$env:LOCALAPPDATA\Programs\zh_webterm\zh_webterm.exe" -WindowStyle Hidden
```

---

## Usage

Once installed, **zh_webterm runs automatically in the background** on port `2424` and auto-starts on boot/login.

### Manual Commands / Custom Port:
```bash
zh_webterm -p 8080
```

### Service Management (Linux):
```bash
sudo systemctl status zh_webterm
sudo systemctl restart zh_webterm
sudo systemctl stop zh_webterm
```

