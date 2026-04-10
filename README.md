# telescrap-sr
<br>
<p align="center">
	<img src="doc/img/logo.png" width="150">
</p>

[![Cross-Compile and Deploy](https://github.com/Thejulfi/telescrap-sr/actions/workflows/deploy.yml/badge.svg)](https://github.com/Thejulfi/telescrap-sr/actions/workflows/deploy.yml)

Scraping tool to get notification for resale ticket, currently implemented for Stade Rochelais rugby matches.

## Roadmap

- [x] Publish overview of the seat that is available for sale
- [ ] Filters to only get notified for specific : 
	- [x] Match (to focus on seats for specific matches)
	- [x] Seat location
	- [x] Price range
	- [ ] Date range
- [ ] Aggressive mode that add to cart matches corresponding to the filters and notify the user only if the purchase is successful
- [ ] Admin panel to manage the bot (web interface or terminal)
- [x] Add a store functionality to save scan configurations results and differences

## Why this project ?

This is a project born from a simple observation. The number of people subscribed to well known rugby clubs is constantly increasing and prevents in its current state any new person who does not follow the matches closely from being able to access tickets. Fortunately, there is a resale platform, which is however itself saturated.

By creating this bot, I wanted to give everyone the opportunity to access tickets for the matches of their favorite club, even if they are not subscribed to the club's news or do not have the time to check the resale platform regularly.

The current ticket resale platforms are not designed to be easily accessible to everyone, and often require a lot of time and effort to find the right tickets. By automating this process, I hope to make it easier for everyone to access tickets for their favorite matches.

## How does it work?

The bot analyzes the homepage of the ticketing site, looking for the matches that are currently resaling tickets. When it finds a match, it checks if the tickets are available for resale and if they are, it sends a notification to the Telegram channel with the details of the match and the price of the tickets.

## Configuration

1. Create a Telegram bot and get its token (see [this documentation](https://core.telegram.org/bots#6-botfather)).

2. Get your Telegram chat ID (see [this documentation](https://stackoverflow.com/questions/32423837/telegram-bot-how-to-get-a-group-chat-id)).

3. Create a `.env` file at the root of the project with the following environment variables:
```
TELEGRAM_BOT_TOKEN=your_bot_token_here
TELEGRAM_CHAT_ID=your_channel_chat_id_here
```

## 🚨 Telegram Resale Channel 🚨

The bot is currently active on a private resale channel, accessible only by manual addition.
You can request access by contacting the project administrator via private message.

## Github Workflow

Current Github workflow (`deploy.yml`) is configured to automatically build and deploy the bot on a server when a new release is published.
## See also

- [ARCHITECTURE.md](doc/ARCHITECTURE.md) : for more details about the architecture of the project and the crates organization.
- [CHANGELOG.md](CHANGELOG.md) : for a detailed list of changes and updates made to the project over time.