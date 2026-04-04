use crate::controller::notify::Notify;

pub struct ConsoleNotifier;

impl Notify for ConsoleNotifier {
    fn send(&self, message: &str) {
        println!("[NOTIF] {}\n-----------------", message);
    }
}
