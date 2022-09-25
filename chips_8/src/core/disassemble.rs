use std::{vec::Vec, fmt::Display};

use super::utils::{get_nnn, get_x, get_kk, get_y, get_nibble};

type DisassembledOpTuple = (String, String, String, String, String, Option<String>);

pub struct DisassembledChip8 {
    op_tuple_vec: Vec<DisassembledOpTuple>
}

impl DisassembledChip8 {
    pub fn new(bytes: &[u8]) -> DisassembledChip8 {
        let op_tuple_vec = bytes.iter().enumerate().step_by(2).map(|(idx, b_val)| {
            let assembled_opcode = (*b_val as u16) << 8
                | bytes[idx + 1] as u16;
            get_disassembled_op_tuple(&idx, &assembled_opcode)
        }).collect();

        DisassembledChip8 { op_tuple_vec }
    }
}

fn create_disassembled_op_tuple(idx: &usize, op: &u16, op_type: &str, description: &str) -> DisassembledOpTuple {
    let hex_string = format!("{:0>4X}", op);
    let op_string = hex_string.split_at(2);
    (format!("{:X}", idx + 0x200), String::from(op_string.0), String::from(op_string.1), String::from(op_type), String::from(description), None)
}

fn create_disassembled_op_tuple_desc_string(idx: &usize, op: &u16, op_type: &str, description: String, comment: Option<&str>) -> DisassembledOpTuple {
    let comment_result = match comment {
        Some(comment_exists) => Some(String::from(comment_exists)),
        None => None
    };

    let hex_string = format!("{:X}", op);
    let op_string = hex_string.split_at(2);

    (format!("{:X}", idx + 0x200), String::from(op_string.0), String::from(op_string.1), String::from(op_type), description, comment_result)
}

fn get_disassembled_op_tuple(idx: &usize, op: &u16) -> DisassembledOpTuple {
    match op & 0xF000 {
        0x0000 => {
            match op & 0x00FF {
                // 0x00E0 - CLS - Clears the screen
                0x00E0 => {
                    // Compiles to memset so fastest possible solution
                    return create_disassembled_op_tuple(idx, op, "CLS", "Clears the screen")
                }
                // 0x00EE - RET - Returns from subroutine
                0x00EE => {
                    return create_disassembled_op_tuple(idx, op, "RET", "Returns from subroutine")
                }
                // 0x0nnn - SYS addr - no-op this is ignored on modern compilers
                _ => {
                    return create_disassembled_op_tuple(idx, op, "SYS", "Feature used by old comnputers")
                }
            }
        }

        // 0x1nnn - JP addr - JMP to addr nnn
        0x1000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "JMP", format!("JMP {:X}", get_nnn(&op)), None)
        }

        // 0x2nnn - CALL addr - Calls subroutine at address nnn
        0x2000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "CALL", format!("S, #${:X}", get_nnn(op)), None)
        }

        // 0x3xkk - SE Vx, byte - Skip next if Vx == kk
        0x3000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "SE", format!("V{:X}, {:X}", get_x(op), get_kk(op)), Some("Vx == kk"));
        }

        // 0x4xkk - SNE Vx, byte - Skip next if Vx != kk
        0x4000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "SNE", format!("V{:X}, {:X}", get_x(op), get_kk(op)), Some("Vx != kk"));
        }

        // 0x5xy0 - SE Vx, Vy - Skip next if Vx == Vy
        0x5000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "SE", format!("V{:X}, {:X}", get_x(op), get_y(op)), Some("Vx == Vy"));
        }

        // 0x6xkk - LD Vx, byte - Load kk into Vx
        0x6000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("V{:X}, {:X}", get_x(op), get_kk(op)), None)
        }

        // 0x7xkk - ADD Vx, byte - Add value of byte
        0x7000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "ADD", format!("V{:X}, {:X}", get_x(op), get_kk(op)), None)
        }

        0x8000 => match op & 0x000F {
            // LD Vx, Vy
            0x0000 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // OR Vx, Vy
            0x0001 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "OR", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // AND Vx, Vy
            0x0002 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "AND", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // XOR Vx, Vy
            0x0003 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "XOR", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // ADD Vx, Vy - Add Vx to Vy
            0x0004 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "ADD", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // SUB Vx, Vy - Subtract Vx from Vy
            0x0005 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SUB", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // SHR Vx - RHS 1
            0x0006 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SHR", format!("V{:X}", get_x(op)), None)
            }
            // SUBN Vx, Vy - Subtract Vy from Vx
            0x0007 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SUBN", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
            }
            // SHL Vx - LHS 1
            0x000E => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SHL", format!("V{:X}", get_x(op)), None)
            }
            _ => create_disassembled_op_tuple(idx, op, "X", "Unsupported op found"),
        },

        // 0x9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
        0x9000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "SNE", format!("V{:X}, V{:X}", get_x(op), get_y(op)), None)
        }

        // 0xAnnn - LD I, addr - Sets I to the address nnn
        0xA000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("I, {:X}", get_nnn(op)), None)
        }

        // Bnnn - JP V0, addr - Jump to location addr
        0xB000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "JMP", format!("V0, {:X}", get_nnn(op)), None)
        }

        // Cxkk - RND Vx, byte - Random number from 0 to 255, then &'d w/ byte which is stored into Vx
        0xC000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "RND", format!("V{:X}, {:X}", get_x(op), get_nnn(op)), None)
        }

        // 0xDxyn - DRW Vx, Vy, nibble - Draw n-byte sprite starting at mem loc I @ (vx, Vy), set VF = collision
        0xD000 => {
            return create_disassembled_op_tuple_desc_string(idx, op, "DRW", format!("V{:X}, V{:X}, {:X}", get_x(op), get_y(op), get_nibble(op)), None)
        }

        0xE000 => match op & 0x00FF {
            // 0xEx9E - SKP Vx - Skip next instruction if key pressed
            0x009E => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SKP", format!("V{:X}", get_x(op)), None)
            }

            // 0xExA1 - SKNP Vx - Skip next instruction if key not pressed
            0x00A1 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "SKNP", format!("V{:X}", get_x(op)), None)
            }
            _ => create_disassembled_op_tuple(idx, op, "X", "Unsupported op found"),
        },

        0xF000 => match op & 0x00FF {
            // 0xFx07 - LD Vx, DT - Loading Delay Timer into Vx
            0x0007 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("V{:X}, DT", get_x(op)), Some("DT - Delay Timer"))
            }
            // 0xFx0A - LD Vx, K - Stop execution till key press
            0x000A => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("V{:X}, K", get_x(op)), Some("Stop execution till keypress"))
            }
            // 0xFx15 - LD DT, Vx - Load Vx into DT
            0x0015 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("DT, V{:X}", get_x(op)), None)
            }
            // 0xFx18 - LD ST, Vx - Set sound timer to Vx
            0x0018 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("ST, V{:X}", get_x(op)), Some("ST - Sound Timer"))
            }
            // 0xFx1E - ADD I, Vx - Add I and Vx then store in I
            0x001E => {
                return create_disassembled_op_tuple_desc_string(idx, op, "ADD", format!("I, V{:X}", get_x(op)), None)
            }
            // 0xFx29 - LD F, Vx - Set I = location of sprite for digit Vx
            0x0029 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("F, V{:X}", get_x(op)), Some("Load font"))
            }
            // 0xFx33 - LD F, Vx - set_BCD
            0x0033 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("F, V{:X}", get_x(op)), Some("Set BCD"))
            }
            // 0xFx55 - LD [I], Vx -reg_dump
            0x0055 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("[I], V{:X}", get_x(op)), Some("Reg Dump"))
            }
            // 0xFx65 - LD Vx, [I] - reg_load
            0x0065 => {
                return create_disassembled_op_tuple_desc_string(idx, op, "LD", format!("V{:X}, I", get_x(op)), None)
            }
            _ => create_disassembled_op_tuple(idx, op, "X", "Unsupported op found"),
        },

        // TODO: Impl other opcodes
        _ => create_disassembled_op_tuple(idx, op, "X", "Unsupported op found"),
    }
}

impl Display for DisassembledChip8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let default_comment = String::from("");
        self.op_tuple_vec.iter().for_each(|op_tuple| {
            let comment = if let Some(comment_result) = &op_tuple.5 {
                comment_result
            } else {
                &default_comment
            };

            match write!(f, "{} {:>4} {:>2} {:>5} {} {}\n", op_tuple.0, op_tuple.1, op_tuple.2, op_tuple.3, op_tuple.4, comment) {
                Err(x) => println!("{:?}", x),
                Ok(_) => {}
            }
        });

        Ok(())
    }
}