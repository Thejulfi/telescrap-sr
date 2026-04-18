# How to Install and Use Telescrap

## Telegram bot creation

1. Create a Telegram bot and get its token (see [this documentation](https://core.telegram.org/bots#6-botfather)).

2. Get your Telegram chat ID and save it for later (see [this documentation](https://stackoverflow.com/questions/32423837/telegram-bot-how-to-get-a-group-chat-id)).


## Compilation and deployment (manual)

*Pre-requisite: Rust and Cargo must be installed on your machine.*

Install the `cargo-zigbuild` tool and the stable Rust toolchain to enable cross-compilation:

```bash
cargo install cargo-zigbuild
rustup install stable
```

To deploy on a Raspberry Pi 3/4, target the `aarch64-unknown-linux-gnu` architecture:

```bash
rustup target add aarch64-unknown-linux-gnu
cargo zigbuild --release --target aarch64-unknown-linux-gnu
```

The compiled binary will be at `target/aarch64-unknown-linux-gnu/release/telescrap-sr`. Copy it to your server:

```bash
scp target/aarch64-unknown-linux-gnu/release/telescrap-sr user@your-server-ip:/home/user/telescrap/
```

## Set up your running server

To run the bot, you need a server — either a physical machine (like a Raspberry Pi) or a VPS from a hosting provider.

### Raspberry Pi setup

In this section, we will see how to structure your Raspberry Pi to run one or multiple instances of the bot.

**1. Create the file tree**

The binary is shared between all instances, while each instance has its own directory for its configuration:

```bash
mkdir ~/telescrap
mkdir ~/telescrap/instances
mkdir ~/telescrap/instances/bot1
```

**2. Copy the binary**

If you haven't already, copy the compiled binary to the server (see [Compilation and deployment](#compilation-and-deployment-manual)):

```bash
scp target/aarch64-unknown-linux-gnu/release/telescrap-sr user@your-server-ip:/home/user/telescrap/
```

**3. Create the .env file**

Create a `.env` file in the `bot1` directory with the following content:

```env
TELEGRAM_BOT_TOKEN=your-telegram-bot-token
TELEGRAM_CHAT_ID=your-telegram-chat-id

# Required only for aggressive scan mode
SHOP_EMAIL=your-shop-email
SHOP_PASSWORD=your-shop-password

# Admin panel port for this instance (default: 3000)
ADMIN_PANEL_PORT=3000
```

**4. Create the systemd service**

Create a service file at `/etc/systemd/system/telescrap-bot1.service` with the following content:

```ini
[Unit]
Description=Telescrap Bot 1
After=network.target

[Service]
WorkingDirectory=/home/user/telescrap/instances/bot1
ExecStart=/home/user/telescrap/telescrap-sr
EnvironmentFile=/home/user/telescrap/instances/bot1/.env
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

**5. Start the service**

Reload systemd and enable the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable telescrap-bot1
sudo systemctl start telescrap-bot1
```

**Useful management commands:**

```bash
# Check the status of the service
sudo systemctl status telescrap-bot1

# View live logs
sudo journalctl -u telescrap-bot1 -f

# Restart the service (e.g. after updating the binary)
sudo systemctl restart telescrap-bot1

# Stop the service
sudo systemctl stop telescrap-bot1

# Disable autostart on boot
sudo systemctl disable telescrap-bot1
```

## Access the Web admin panel

The bot also provides a web admin panel to manage its configuration at runtime. It is accessible at `http://localhost:3000` (port is configurable via `ADMIN_PANEL_PORT`).

From it you can:
- Update the scan configuration (scan mode, scan interval, active filters, etc.)