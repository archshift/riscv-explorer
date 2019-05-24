use wasm_bindgen::prelude::*;

mod interpreter;
mod memory;

#[wasm_bindgen]
pub struct SimState {
    program: interpreter::Program,
    regs: interpreter::Registers,
    mem: memory::Memory,
    curr_line: usize,
    error: String,
}

#[wasm_bindgen]
pub fn makeSimstate() -> SimState {
    SimState {
        program: interpreter::Program::new(),
        regs: interpreter::Registers::new(),
        mem: memory::Memory::new(),
        curr_line: 0,
        error: String::new(),
    }
}

#[wasm_bindgen]
pub fn setCodeText(state: &mut SimState, code: &str) {
    let code: Vec<String> = code.split("\n")
        .map(|s| s.trim().to_owned())
        .collect();
    state.program.code = code;
}

#[wasm_bindgen]
pub fn addBreakpoint(state: &mut SimState, line: usize) {
    
}

#[wasm_bindgen]
pub fn clearBreakpoints(state: &mut SimState) {

}

#[wasm_bindgen]
pub fn removeBreakpoint(state: &mut SimState, line: usize) {

}

#[wasm_bindgen]
pub fn runToBreak(state: &mut SimState) -> bool {
    while state.curr_line < state.program.code.len() {
        if !step(state) {
            return false;
        }
    }
    true
}

#[wasm_bindgen]
pub fn step(state: &mut SimState) -> bool {
    if state.curr_line >= state.program.code.len() {
        state.error = "Stepped over end of program!".to_owned();
        return false;
    }

    let inst = &state.program.code[state.curr_line];
    state.curr_line += 1;
    if inst.len() == 0 { return true }

    if let Err(what) = interpreter::run(&mut state.regs, &mut state.mem, inst) {
        state.error = format!("Could not execute `{}`: \"{}\"", inst, what);
        return false;
    }

    true
}

#[wasm_bindgen]
pub fn getRegs(state: &SimState, regs_out: &mut [u32]) {
    regs_out.copy_from_slice(&state.regs.file);
}

#[wasm_bindgen]
pub fn getErr(state: &SimState) -> String {
    state.error.clone()
}
