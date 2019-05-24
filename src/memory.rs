#[derive(Copy, Clone)]
pub(crate) enum MoesiState {
    Modified,
    Owner,
    Exclusive,
    Shared,
    Invalid
}

impl Default for MoesiState {
    fn default() -> Self { MoesiState::Invalid }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct CacheLine {
    data: [u32; 16],
    state: MoesiState
}

#[derive(Default)]
pub(crate) struct Memory {
    file: [CacheLine; 4]
}

impl Memory {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn read32(&self, addr: u32) -> u32 {
        0
    }

    pub(crate) fn write32(&self, addr: u32, val: u32) {
        
    }
}
