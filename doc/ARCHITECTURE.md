# Telescrap architecture

This document describes the architecture of the Telescrap bot with a focus on the crates organization and the main components of the project.

## Crates organization

Each crate in the Telescrap project has made to have a specific responsibility and can be used independently of the others.

Crate are organized in a hierarchical way following the following order:
1. `parser` : This crate is responsible for parsing the HTML content of the ticketing website and extracting the exhaustive relevant information about the matches and available seats.
2. `scanner` : This crate is responsible for the scanning process, which consists in periodically checking the ticketing website and filtering the relevant information. It uses the `parser` crate to extract the relevant information from the HTML content.
3. `telegram-notifier` : This crate is responsible for sending notifications to the Telegram channel when new matches and available seats are found. It uses the `scanner` crate to get the relevant information about the matches and available seats.

## See also

- [Parser documentation](../crates/parser/README.md)
- [Scanner documentation](../crates/scanner/README.md)
- [Telegram Notifier documentation](../crates/telegram-notifier/README.md)