use crate::net_structs::*;

pub struct Bot {
    last_ticcmd: TicCmd,
}

impl Bot {
    pub fn new() -> Self {
        Bot {
            last_ticcmd: TicCmd::default(),
        }
    }

    pub fn init(&mut self) {
        // TODO: Placeholder
    }

    pub fn tick(&mut self) -> TicCmd {
        // TODO: Placeholder for bot behavior
        self.last_ticcmd.forwardmove = 50;
        self.last_ticcmd.sidemove = 0;
        self.last_ticcmd.angleturn = 0;

        self.last_ticcmd
    }
}
