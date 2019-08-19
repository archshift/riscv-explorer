use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::memory::Memory;


#[derive(Default)]
pub(crate) struct Program {
    pub(crate) code: Vec<String>,
    labels: HashMap<String, usize>,
    pub(crate) has_labels: bool,
    pub(crate) breakpoints: HashSet<usize>
}

impl Program {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}


pub(crate) struct Registers {
    pub(crate) file: [u32; 32],
    pub(crate) pc: usize,
}

impl Registers {
    pub(crate) fn new() -> Self {
        Self {
            file: Default::default(),
            pc: 0
        }
    }
}

lazy_static! {
    static ref PARSER_LABEL: Regex = Regex::new(r"(?x)
        # LABEL :
          (\w+) :
    ").unwrap();

    static ref PARSER_TYPE_R: Regex = Regex::new(r"(?x)
        # OPERATOR   RD           RS1         RS2
          (\w+)   \s (\w+),    \s (\w+),   \s (\w+)
    ").unwrap();

    static ref PARSER_TYPE_J: Regex = Regex::new(r"(?x)
        # OPERATOR   RD           LABEL
          (\w+)   \s (\w+),    \s (\w+)
    ").unwrap();

    static ref PARSER_TYPE_I: Regex = Regex::new(r"(?x)
        # OPERATOR   RD           RS1         IMMED
          (\w+)   \s (\w+),    \s (\w+),   \s (-?\d\w*)
    ").unwrap();
    
    static ref PARSER_TYPE_LI: Regex = Regex::new(r"(?x)
        # OPERATOR   RD           IMMED
          (\w+)   \s (\w+),    \s (-?\d\w*)
    ").unwrap();

    static ref PARSER_TYPE_LS: Regex = Regex::new(r"(?x)
        # OPERATOR   RD           OFFS            RA
          (\w+)   \s (\w+),    \s (\d\w*)  \s* \( (\w+) \)
    ").unwrap();
}

fn capture_field<'a>(caps: &'a Captures, field: usize) -> Result<&'a str, String> {
    let cap = caps.get(field).ok_or("Failed to acquire capture field")?;
    Ok(cap.as_str())
}

fn reg(name: &str) -> Result<usize, String> {
    Ok(match name {
        "x0" | "zero" => 0,
        "x1" | "ra" => 1,
        "x2" | "sp" => 2,
        "x3" | "gp" => 3,
        "x4" | "tp" => 4,
        
        "x5" | "t0" => 5,
        "x6" | "t1" => 6,
        "x7" | "t2" => 7,
        
        "x8" | "s0" | "fp" => 8,
        "x9" | "s1" => 9,

        "x10" | "a0" => 10,
        "x11" | "a1" => 11,
        "x12" | "a2" => 12,
        "x13" | "a3" => 13,
        "x14" | "a4" => 14,
        "x15" | "a5" => 15,
        "x16" | "a6" => 16,
        "x17" | "a7" => 17,

        "x18" | "s2" => 18,
        "x19" | "s3" => 19,
        "x20" | "s4" => 29,
        "x21" | "s5" => 21,
        "x22" | "s6" => 22,
        "x23" | "s7" => 23,
        "x24" | "s8" => 24,
        "x25" | "s9" => 25,
        "x26" | "s10" => 26,
        "x27" | "s11" => 27,

        _ => return Err(format!("Could not match register {}", name)),
    })
}

fn immed(field: &str) -> Result<i32, String> {
    field.parse::<i32>()
        .map_err(|_| format!("Could not convert immed `{}` to integer", field))
}

fn register_label(program: &mut Program, pc: usize) -> Result<(), String> {
    let line = &program.code[pc];
    if line.len() == 0 { return Ok(()) }

    let label_caps = PARSER_LABEL.captures(line);

    if let Some(ref label) = label_caps {
        let label_name = capture_field(label, 1)?;
        program.labels.insert(label_name.to_owned(), pc);
        return Ok(())
    }
    Ok(())
}

pub(crate) fn scan_labels(program: &mut Program) -> Result<(), String> {
    for i in 0..program.code.len() {
        register_label(program, i)?;
    }
    Ok(())
}

pub(crate) fn run(regs: &mut Registers, mem: &mut Memory, program: &mut Program) -> Result<(), String> {
    let line = &program.code[regs.pc];

    let rtype_caps = PARSER_TYPE_R.captures(line);
    let jtype_caps = PARSER_TYPE_J.captures(line);
    let itype_caps = PARSER_TYPE_I.captures(line);
    let litype_caps = PARSER_TYPE_LI.captures(line);
    let lstype_caps = PARSER_TYPE_LS.captures(line);
    let label_caps = PARSER_LABEL.captures(line);

    let rtype = rtype_caps.as_ref().ok_or("Could not match r-type fields: <OP> <RD>, <RS1>, <RS2>");
    let jtype = jtype_caps.as_ref().ok_or("Could not match jal-type fields: <OP> <RD>, <LABEL>");
    let itype = itype_caps.as_ref().ok_or("Could not match i-type fields: <OP> <RD>, <RS1>, <IMM>");
    let mvtype = jtype_caps.as_ref().ok_or("Could not match mv-type fields: <OP> <RD>, <RS>");
    let litype = litype_caps.as_ref().ok_or("Could not match li-type fields: <OP> <RD>, <IMM>");
    let lstype = lstype_caps.as_ref().ok_or("Could not match l/s-type fields: <OP> <RD>, <OFFS>(<RA>)");

    let r_rd  = || reg(capture_field(rtype?, 2)?);
    let r_rs1 = || reg(capture_field(rtype?, 3)?);
    let r_rs2 = || reg(capture_field(rtype?, 4)?);

    let j_rd  = || reg(capture_field(jtype?, 2)?);
    let j_lbl = || capture_field(jtype?, 3);

    let i_rd    = || reg(capture_field(itype?, 2)?);
    let i_rs1   = || reg(capture_field(itype?, 3)?);
    let i_immed = || immed(capture_field(itype?, 4)?);

    let mv_rd  = || reg(capture_field(mvtype?, 2)?);
    let mv_rs1 = || reg(capture_field(mvtype?, 3)?);

    let li_rd    = || reg(capture_field(litype?, 2)?);
    let li_immed = || immed(capture_field(litype?, 3)?);

    let ls_rd   = || reg(capture_field(lstype?, 2)?);
    let ls_offs = || immed(capture_field(lstype?, 3)?);
    let ls_ra   = || reg(capture_field(lstype?, 4)?);

    let mut operator = line.split(" ").next().ok_or("Could not find operator!");

    if label_caps.is_some() {
        operator = Ok("");
    }

    let mut advance_pc = true;

    match operator? {
        // MV-TYPE INSTRUCTIONS
        "MV" => {
            regs.file[mv_rd()?] = regs.file[mv_rs1()?];
        }
        "LI" => {
            regs.file[li_rd()?] = li_immed()? as u32;
        }

        // R-TYPE INSTRUCTIONS
        "ADD" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?].wrapping_add(regs.file[r_rs2()?]);
        }
        "SUB" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?].wrapping_sub(regs.file[r_rs2()?]);
        }
        "AND" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?] & regs.file[r_rs2()?];
        }
        "OR" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?] | regs.file[r_rs2()?];
        }
        "MUL" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?].wrapping_mul(regs.file[r_rs2()?]);
        }
        "XOR" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?] ^ regs.file[r_rs2()?];
        }
        "SLL" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?] << regs.file[r_rs2()?];
        }
        "SLR" => {
            regs.file[r_rd()?] = regs.file[r_rs1()?] >> regs.file[r_rs2()?];
        }

        // I-TYPE INSTRUCTIONS
        "ADDI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?].wrapping_add(i_immed()? as u32);
        }
        "SUBI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?].wrapping_sub(i_immed()? as u32);
        }
        "ANDI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?] & (i_immed()? as u32);
        }
        "ORI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?] | (i_immed()? as u32);
        }
        "XORI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?] ^ (i_immed()? as u32);
        }
        "SLLI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?] << (i_immed()? as u32);
        }
        "SLRI" => {
            regs.file[i_rd()?] = regs.file[i_rs1()?] >> (i_immed()? as u32);
        }
        "JALR" => {
            regs.file[i_rd()?] = regs.pc as u32 + 1;
            regs.pc = (regs.file[i_rs1()?] + i_immed()? as u32) as usize;
            advance_pc = false;
        }

        // JAL-TYPE INSTRUCTIONS
        "JAL" => {
            let label = j_lbl()?;
            regs.file[j_rd()?] = regs.pc as u32 + 1;
            regs.pc = *program.labels.get(label)
                .ok_or_else(|| format!("Could not find label {}", label))?;
            advance_pc = false;
        }

        // LD/ST INSTRUCTIONS
        "LW" => {
            let addr = regs.file[ls_ra()?].wrapping_add(ls_offs()? as u32);
            regs.file[ls_rd()?] = mem.read32(addr)?;
        }
        "SW" => {
            let addr = regs.file[ls_ra()?].wrapping_add(ls_offs()? as u32);
            mem.write32(addr, regs.file[ls_rd()?])?;
        }

        "" => {}

        unk => return Err(format!("Could not match operator {}", unk))
    }
    regs.file[0] = 0;

    if advance_pc {
        regs.pc += 1;
    }

    Ok(())
}

#[test]
fn test_interpret() {
    let mut regs = Registers::new();
    let mut mem = Memory::new();
    let mut program = Program::new();
    program.code = vec![
        "ADD x1, x1, x1".to_owned(),
        "ADDI x1, x1, 1".to_owned(),
        "LW x1, 0(x0)".to_owned()
    ];
    regs.file[1] = 3;
    
    run(&mut regs, &mut mem, &mut program).unwrap();
    assert_eq!(regs.file[1], 6);
    regs.pc += 1;
    
    run(&mut regs, &mut mem, &mut program).unwrap();
    assert_eq!(regs.file[1], 7);
    regs.pc += 1;

    run(&mut regs, &mut mem, &mut program).unwrap();
    assert_eq!(regs.file[1], 0);
    regs.pc += 1;
}
