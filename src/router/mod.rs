pub struct Router {}

impl Router {
    pub fn new() -> Self {
        Self {}
    }

    pub fn allocate_queue(&self) -> CommandQueue {
        CommandQueue {}
    }
}

pub struct CommandQueue {}
