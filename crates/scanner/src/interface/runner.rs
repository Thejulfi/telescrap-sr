use crate::core::scan::ScanConfig;
use crate::controller::notify::Notify;
use crate::app::scan_task::ScanTask;

pub struct ScannerHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl ScannerHandle {
    pub fn start(config: ScanConfig, notifier: impl Notify) -> Self {
        let task = ScanTask::new(config, notifier);
        let handle = tokio::spawn(async move { task.run().await });
        Self { abort_handle: handle.abort_handle() }
    }

    pub fn stop(&self) {
        self.abort_handle.abort();
    }
}