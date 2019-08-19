#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

mod interpreter;
mod memory;

#[wasm_bindgen]
pub struct SimState {
    program: interpreter::Program,
    regs: interpreter::Registers,
    mem: memory::Memory,
    error: String,
}

#[wasm_bindgen]
pub fn makeSimstate() -> SimState {
    SimState {
        program: interpreter::Program::new(),
        regs: interpreter::Registers::new(),
        mem: memory::Memory::new(),
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
    state.program.breakpoints.insert(line);
}

#[wasm_bindgen]
pub fn clearBreakpoints(state: &mut SimState) {
    state.program.breakpoints.clear();
}

#[wasm_bindgen]
pub fn removeBreakpoint(state: &mut SimState, line: usize) {
    state.program.breakpoints.remove(&line);
}

#[wasm_bindgen]
pub fn runAmount(state: &mut SimState, count: usize) -> bool {
    let label_ret = interpreter::scan_labels(&mut state.program);
    if let Err(s) = label_ret {
        state.error = s.to_owned();
        return false
    }

    for i in 0..count {
        if state.regs.pc >= state.program.code.len() {
            break;
        }
        if i != 0 {
            let found_bp = state.program.breakpoints.get(&i).is_some();
            if found_bp {
                return true
            }
        }
        if !run_inner(state) {
            return false
        }
    }
    true
}

fn run_inner(state: &mut SimState) -> bool {
    if state.regs.pc >= state.program.code.len() {
        state.error = "Stepped over end of program!".to_owned();
        return false;
    }

    if let Err(what) = interpreter::run(&mut state.regs, &mut state.mem, &mut state.program) {
        state.error = format!("Error on line {}, `{}`: \"{}\"",
            state.regs.pc,
            state.program.code[state.regs.pc],
            what);
        return false;
    }

    true
}

#[wasm_bindgen]
pub fn step(state: &mut SimState) -> bool {
    runAmount(state, 1)
}

#[wasm_bindgen]
pub fn getRegs(state: &SimState, regs_out: &mut [u32]) {
    regs_out.copy_from_slice(&state.regs.file);
}

#[wasm_bindgen]
pub fn getMem(state: &SimState, mem_out: &mut [u32]) {
    for (i, line) in state.mem.file.iter().enumerate() {
        let start = i * 16;
        let end = (i + 1) * 16;
        mem_out[start..end].copy_from_slice(&line.data);
    }
}

#[wasm_bindgen]
pub fn getErr(state: &SimState) -> String {
    state.error.clone()
}
