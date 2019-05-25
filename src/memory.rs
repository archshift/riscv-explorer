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
    pub(crate) data: [u32; 16],
    state: MoesiState
}

#[derive(Default)]
pub(crate) struct Memory {
    pub(crate) file: [CacheLine; 8]
}

impl Memory {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn read32(&self, addr: u32) -> Result<u32, String> {
        let index = (addr / 64) as usize;
        let offs = (addr % 64) as usize;
        if offs % 4 != 0 {
            return Err(format!("Tried to read word from unaligned address 0x{:08X}", addr));
        }
        if index >= self.file.len() {
            return Err(format!("Tried to read word from unmapped address 0x{:08X}", addr));
        }

        Ok(self.file[index].data[offs / 4])
    }

    pub(crate) fn write32(&mut self, addr: u32, val: u32) -> Result<(), String> {
        let index = (addr / 64) as usize;
        let offs = (addr % 64) as usize;
        if offs % 4 != 0 {
            return Err(format!("Tried to write word to unaligned address 0x{:08X}", addr));
        }
        if index >= self.file.len() {
            return Err(format!("Tried to write word to unmapped address 0x{:08X}", addr));
        }

        self.file[index].data[offs / 4] = val;
        Ok(())
    }
}
