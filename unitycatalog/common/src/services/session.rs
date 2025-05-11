use datafusion::prelude::SessionContext;
use delta_kernel_datafusion::KernelContextExt as _;

pub struct KernelSession {
    ctx: SessionContext,
}

impl KernelSession {
    pub fn new() -> Self {
        let ctx = SessionContext::new().enable_delta_kernel();
        Self { ctx }
    }
}
