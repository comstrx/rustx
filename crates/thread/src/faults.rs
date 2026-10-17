use error::{Code, Fault};

use crate::Faults;

impl Faults {

    /// A worker thread panicked.
    pub const WORKER_PANICKED: Fault = Fault::new("worker_panicked", Code::Internal, "a worker panicked");

    /// A task panicked.
    pub const TASK_PANICKED: Fault = Fault::new("task_panicked", Code::Internal, "the task panicked");

    /// A task was aborted before it ended.
    pub const TASK_CANCELLED: Fault = Fault::new("task_cancelled", Code::Cancelled, "the task was aborted");

    /// Every failure above.
    pub const ALL: &[Fault] = &[Self::WORKER_PANICKED, Self::TASK_PANICKED, Self::TASK_CANCELLED];

}
