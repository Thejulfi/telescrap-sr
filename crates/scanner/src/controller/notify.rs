/// This module defines the `Notify` trait, which is used to send notifications about changes detected in the scanned encounters.
/// The trait provides a method for sending messages, and it is designed to be implemented by any type that can handle notifications,
/// such as a Telegram bot or an email sender.
pub trait Notify: Send + 'static {
    fn send(&self, message: &str);
    fn send_photo(&self, photo_url: &str, caption: &str);
    fn send_and_pin(&self, message: &str) -> Option<i32>;
    fn edit_message(&self, message_id: i32, message: &str);
}
