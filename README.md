# Server Monitor

Server monitor is a simple Rust program that monitors the statuses of multiple endpoints and notifies you when they go down using your telegram bot.

## Installation

Clone the repository and run the following command to build the project.

```bash
cargo build --release
```

## Env Variables

- `TELOXIDE_TOKEN` - Your telegram bot token.
- `TELEGRAM_CHAT_ID` - Your telegram chat id.
- `URLS` - Comma separated list of urls to monitor.
- `INTERVAL` - Interval in milliseconds to check the urls.

Here's an example:

```bash
TELOXIDE_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
TELEGRAM_CHAT_ID=1234567890
URLS=https://google.com,https://bing.com,https://github.com
INTERVAL=5000
```

## Run the program

```bash
cargo run --release
```

## Docker

Run the following command to run on Docker:

```bash
docker run -d \
  --name server-monitor \
  -e URLS=https://example1.com,https://example2.com \
  -e TELOXIDE_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11 \
  -e TELEGRAM_CHAT_ID=1234567890 \
  -e INTERVAL=5000 \
  ghcr.io/dcodesdev/server-monitor
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](LICENSE)
