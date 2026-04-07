# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-04-06

### Added
- Better difference detection to identify new and removed seats more accurately, even when the seat numbers are not in the same order or when some seats are added or removed between scans.
- Filters : 
    - Filter by seat composition (e.g., only notify about seats that are together or only about single seats).
    - Filter by match type (e.g., only notify for certain matches titles).
    - Filter by price range (e.g., only notify about seats within a certain price range).
- Documentation for subcrates (still very light, but it's a start).
- Database integration to keep track of sent of each matches potential resale link and avoid scanning the main page for target's matches filtering.

### Changed
- Adding comments to the whole codebase to improve readability and maintainability, especially for the core logic of the parser and scanner.

### Removed

## [2.0.0] - 2026-04-04

### Added
- Crate parser : responsible for parsing the HTML of the ticketing website and extract relevant information about the matches and the available seats.
- Crate scanner : responsible for performing the scans of the ticketing website, comparing the results with the previous scans, and notifying the Telegram bot of any changes.
- Crate telegram-notifier : responsible for sending notifications to the Telegram channel when resale tickets are detected;

### Changed
- The whole project's structure for better maintainability and readability (using library crates and clean architecture principles).

### Removed
- The old additional implementaion of the bot that made him more than a simple parser/notifier.
    - Message on telegram channel to notify users about upcoming matches at the beginning of the week, 1h before the match and at the kick-off.
    - log printed and registered in a local file
    - Administrator configuration on a private channel to start/stop get the status or setting the polling interval
    - Auomatic deletion of resale messages at the end of the week or day with a local database to keep track of sent messages

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