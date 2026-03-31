pub trait Notify: Send + 'static {
    fn send(&self, message: &str);
}
