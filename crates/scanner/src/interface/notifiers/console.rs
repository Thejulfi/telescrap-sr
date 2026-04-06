/// This module defines the `ConsoleNotifier` struct, which implements the `Notify` trait to send notifications to the console.
/// The `ConsoleNotifier` is a simple implementation of the `Notify` trait that prints messages to the standard output,
/// allowing for easy debugging and monitoring of the scanning process without the need for external
use crate::controller::notify::Notify;

pub struct ConsoleNotifier;

impl Notify for ConsoleNotifier {
    fn send(&self, message: &str) {
        println!("[NOTIF] {}\n-----------------", message);
    }
}
