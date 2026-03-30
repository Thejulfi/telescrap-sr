# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-03-30

### Added
- Message on telegram channel to notify users about upcoming matches at the beginning of the week, 1h before the match and at the kick-off.
- log printed and registered in a local file
- Administrator configuration on a private channel to start/stop get the status or setting the polling interval
- Auomatic deletion of resale messages at the end of the week or day with a local database to keep track of sent messages

### Changed
- Better looking notifications on telegram channel with HTML formatting
- Improve project's structure for better maintainability and readability
- update roadmap in the README, add tags, and add more details about the project in the README

## [1.0.0] - 2026-03-16

### Added

- Telegram bot that perform basic scans the Stade Rochelais ticketing website for resale tickets.
- Notifications sent to a Telegram channel when resale tickets are detected.
- Configuration options for bot token, chat IDs, and admin chat IDs.
- Telegram commands to allow supervision, configuration, start and stop of the bot.