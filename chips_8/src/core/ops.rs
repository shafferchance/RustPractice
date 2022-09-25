use std::fmt::Debug;

use super::utils::{get_kk, get_nibble, get_nnn, get_x, get_y};

pub struct MyChips8 {
    opcode: u16,          // 2B for storing current opcode
    memory: [u8; 0x1000],   // Represent 4K block of memory
    registers: [u16; 0x10], // 15 general purpose and 16th is for carry flag
    i: u16,               // Index register
    pub pc: u16,              // Program Counter

    pub gfx: [u8; 64 * 32], // Stores whether pixel[idx] is on or off (1 or 0)
    delay_timer: u16,   // Will cound down to 0 when > 0
    sound_timer: u16,   // Will count down to 0 when > 0

    stack: [u16; 0x10], // Storing before JMP, ensure that PC is saved as well
    sp: u16,          // Stack Pointer
    key: [u8; 0x10],    // HEX Based keypad, this is used to store state

    // event flags -- temp
    pub wait: bool, // Marker for event loop to wait for next key
    pub draw: bool, // Marker for event loop to draw 
}

const FONT_SET: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
const FONT_BEGIN: usize = 0x50;

impl Debug for MyChips8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("My Chips 8")
         .field("opcode", &format!("{:X}", &self.opcode))
         .field("pc", &format!("{:X}", &self.pc))
         .field("i", &format!("{:X}", &self.i))
         .field("sp", &format!("{:X}", self.sp))
         .finish()
    }
}

impl MyChips8 {
    pub fn new() -> Self {
        let mut my_chips_8 = MyChips8 {
            opcode: 0x0,
            memory: [0; 4096],
            registers: [0; 0x10],
            i: 0x0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,

            stack: [0x0; 0x10],
            sp: 0x0,
            key: [0; 0x10],
            wait: false,
            draw: false
        };

        // Init steps
        my_chips_8.load_font_set();

        my_chips_8
    }

    // Load bytes into memory
    pub fn load_rom(&mut self, chip_8_program: &[u8]) {
        let program_len = chip_8_program.len();
        (0..program_len).for_each(|idx| {
            self.memory[0x200 + idx] = chip_8_program[idx];
        })
    }

    // Load fontsets
    fn load_font_set(&mut self) {
        (FONT_BEGIN..0x9F).enumerate().for_each(|(count, idx)| {
            self.memory[idx] = FONT_SET[count];
        });
    }

    // Reset Timers
    // }


    // Utilizing a clamped LCG, not sure if this is random enough :/
    fn get_rand(&self, state: u8) -> u8 {
        return ((state % 10) * ((((self.pc & 0x00FF) >> 2) as u8) % 122) + 1) % 255;
    }

    fn get_register_value(&self, index: u16) -> u16 {
        return self.registers[index as usize];
    }

    fn set_register(&mut self, index: u16, value: u16) {
        self.registers[index as usize] = value;
    }

    pub fn enumlate_cycle(&mut self) {
        // Fetch
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;

        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x00FF {
                    // 0x00E0 - CLS - Clears the screen
                    0x00E0 => {
                        // Compiles to memset so fastest possible solution
                        self.gfx.iter_mut().for_each(|entry| *entry = 0);
                    }
                    // 0x00EE - RET - Returns from subroutine
                    0x00EE => {
                        self.pc = self.stack[0];
                    }
                    // 0x0nnn - SYS addr - no-op this is ignored on modern compilers
                    _ => {}
                }
            }

            // 0x1nnn - JP addr - JMP to addr nnn
            0x1000 => {
                self.i = get_nnn(&self.opcode);
                self.pc = get_nnn(&self.opcode);
            }

            // 0x2nnn - CALL addr - Calls subroutine at address nnn
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = get_nnn(&self.opcode);
            }

            // 0x3xkk - SE Vx, byte - Skip next if Vx == kk
            0x3000 => {
                if self.get_register_value(get_x(&self.opcode)) == get_kk(&self.opcode) {
                    self.pc += 2;
                }
            }

            // 0x4xkk - SNE Vx, byte - Skip next if Vx != kk
            0x4000 => {
                if self.get_register_value(get_x(&self.opcode)) != get_kk(&self.opcode) {
                    self.pc += 2;
                }
            }

            // 0x5xy0 - SE Vx, Vy - Skip next if Vx == Vy
            0x5000 => {
                if self.get_register_value(get_x(&self.opcode)) == self.get_register_value(get_y(&self.opcode)) {
                    self.pc += 2;
                }
            }

            // 0x6xkk - LD Vx, byte - Load kk into Vx
            0x6000 => {
                let (x, kk) = (get_x(&self.opcode), get_kk(&self.opcode));
                // println!("Op: {:X} | Vx {:X} | kk {:X}", &self.opcode, &vx, &kk);
                self.set_register(x, kk);
            }

            // 0x7xkk - ADD Vx, byte - Add value of byte
            0x7000 => {
                self.set_register(
                    get_x(&self.opcode),
                    self.get_register_value(get_x(&self.opcode)) + get_kk(&self.opcode),
                );
            }

            0x8000 => match self.opcode & 0x000F {
                // LD Vx, Vy
                0x0000 => {
                    self.set_register(get_x(&self.opcode), self.get_register_value(get_y(&self.opcode)));
                }
                // OR Vx, Vy
                0x0001 => {
                    self.set_register(get_x(&self.opcode), get_x(&self.opcode) | get_y(&self.opcode));
                }
                // AND Vx, Vy
                0x0002 => {
                    self.set_register(get_x(&self.opcode), get_x(&self.opcode) & get_y(&self.opcode));
                }
                // XOR Vx, Vy
                0x0003 => {
                    self.set_register(get_x(&self.opcode), get_x(&self.opcode) ^ get_y(&self.opcode));
                }
                // ADD Vx, Vy - Add Vx to Vy
                0x0004 => {
                    if self.get_register_value(get_y(&self.opcode))
                        > (0xFF - self.get_register_value(get_x(&self.opcode)))
                    {
                        self.registers[0xF] = 0x1;
                    } else {
                        self.registers[0xF] = 0x0;
                    }

                    self.set_register(
                        get_x(&self.opcode),
                        self.get_register_value(get_x(&self.opcode))
                            + self.get_register_value(get_y(&self.opcode)),
                    );
                }
                // SUB Vx, Vy - Subtract Vx from Vy
                0x0005 => {
                    if self.get_register_value(get_x(&self.opcode))
                        > self.get_register_value(get_y(&self.opcode))
                    {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(
                        get_x(&self.opcode),
                        self.get_register_value(get_x(&self.opcode))
                            - self.get_register_value(get_y(&self.opcode)),
                    );
                }
                // SHR Vx - RHS 1
                0x0006 => {
                    // Checking LSB of Vx
                    if (self.get_register_value(get_x(&self.opcode)) & 0x1) == 1 {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(get_x(&self.opcode), self.get_register_value(get_x(&self.opcode)) >> 1);
                }
                // SUBN Vx, Vy - Subtract Vy from Vx
                0x0007 => {
                    if self.get_register_value(get_y(&self.opcode))
                        > self.get_register_value(get_x(&self.opcode))
                    {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(
                        get_x(&self.opcode),
                        self.get_register_value(get_y(&self.opcode))
                            - self.get_register_value(get_x(&self.opcode)),
                    );                }
                // SHL Vx - LHS 1
                0x000E => {
                    if (self.get_register_value(get_x(&self.opcode)) & 0x8) == 1 {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(get_x(&self.opcode), self.get_register_value(get_x(&self.opcode)) << 1);
                }
                _ => panic!("Unsupported opcode detected: {:X}", self.opcode),
            },

            // 0x9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            0x9000 => {
                if self.get_register_value(get_x(&self.opcode)) != self.get_register_value(get_y(&self.opcode))
                {
                    self.pc += 2;
                }
            }

            // 0xAnnn - LD I, addr - Sets I to the address nnn
            0xA000 => {
                self.i = get_nnn(&self.opcode);
                // println!("i: {:X}", self.i);
            }

            // Bnnn - JP V0, addr - Jump to location addr
            0xB000 => {
                self.pc = get_nnn(&self.opcode) + self.get_register_value(0x0);
            }

            // Cxkk - RND Vx, byte - Random number from 0 to 255, then &'d w/ byte which is stored into Vx
            0xC000 => {
                self.set_register(
                    get_x(&self.opcode),
                    self.get_rand((get_kk(&self.opcode) as u8) & get_kk(&self.opcode) as u8) as u16,
                );
            }

            // 0xDxyn - DRW Vx, Vy, nibble - Draw n-byte sprite starting at mem loc I @ (vx, Vy), set VF = collision
            0xD000 => {
                self.registers[0xF] = 0x0;
                for idx in 0..get_nibble(&self.opcode)
                {
                    let v_x = self.registers[get_x(&self.opcode) as usize];
                    let v_y = self.registers[get_y(&self.opcode) as usize];

                    let row = (v_y as usize + idx as usize) % 32;
                    let mut sprite = self.memory[(self.i + idx) as usize];

                    for bit_idx in 0..(sprite.count_ones() + sprite.count_zeros()) {
                        let bit_value = (sprite & 0x80) >> 7;
                        let col = ((v_x as u32 + bit_idx) % 64) as usize;
                        let offset = row * 64 + col;

                        if bit_value == 0x1 {
                            if self.gfx[offset] != 0x0 {
                                self.gfx[offset] = 0x0;
                                self.registers[0xF] = 0x1;
                            } else {
                                self.gfx[offset] = 0x1;
                            }
                        }
                        sprite = sprite << 1;
                    }
                }
                self.draw = true;
            }

            0xE000 => match self.opcode & 0x00FF {
                // 0xEx9E - SKP Vx - Skip next instruction if key pressed
                0x009E => {
                    if self.key[get_x(&self.opcode) as usize] == 1 {
                        self.pc += 2;
                    }
                }

                // 0xExA1 - SKNP Vx - Skip next instruction if key not pressed
                0x00A1 => {
                    if self.key[get_x(&self.opcode) as usize] == 0 {
                        self.pc += 2;
                    }
                }
                _ => panic!("Unsupported opcode detected: {:X}", self.opcode),
            },

            0xF000 => match self.opcode & 0x00FF {
                // 0xFx07 - LD Vx, DT - Loading Delay Timer into Vx
                0x0007 => {
                    self.set_register(get_x(&self.opcode), self.delay_timer);
                }
                // 0xFx0A - LD Vx, K - Stop execution till key press
                0x000A => {
                    println!("Wait set");
                    self.wait = true;
                }
                // 0xFx15 - LD DT, Vx - Load Vx into DT
                0x0015 => {
                    self.delay_timer = self.get_register_value(get_x(&self.opcode));
                }
                // 0xFx18 - LD ST, Vx - Set sound timer to Vx
                0x0018 => {
                    self.sound_timer = self.get_register_value(get_x(&self.opcode));
                }
                // 0xFx1E - ADD I, Vx - Add I and Vx then store in I
                0x001E => {
                    self.i = self.i + self.get_register_value(get_x(&self.opcode));
                }
                // 0xFx29 - LD F, Vx - Set I = location of sprite for digit Vx
                0x0029 => {
                    self.i = (FONT_BEGIN as u16 + self.get_register_value(get_x(&self.opcode))) as u16;
                    println!("Font Pointer: {:X} | Opcode: {:X}", get_x(&self.opcode), self.opcode);
                    self.wait = true;
                }
                // 0xFx33 - LD F, Vx - set_BCD
                0x0033 => {
                    self.memory[self.i as usize] =
                        (self.get_register_value(get_x(&self.opcode)) / 100) as u8;
                    self.memory[(self.i + 1) as usize] =
                        (((self.get_register_value(get_x(&self.opcode)) / 100) / 10) % 10) as u8;
                    self.memory[(self.i + 2) as usize] =
                        (((self.get_register_value(get_x(&self.opcode)) / 100) % 100) % 10) as u8;
                }
                // 0xFx55 - LD [I], Vx -reg_dump
                0x0055 => {
                    for (i, &registry) in self.registers[0 as usize..get_x(&self.opcode) as usize]
                        .iter()
                        .enumerate()
                    {
                        // This might cause a panic :fearful:
                        self.memory[self.i as usize + i] = registry as u8;
                    }
                }
                // 0xFx65 - LD Vx, [I] - reg_load
                0x0065 => {
                    for (i, &stored_registry) in self.memory
                        [self.i as usize..self.get_register_value(get_x(&self.opcode)) as usize]
                        .iter()
                        .enumerate()
                    {
                        // Since this is growing not shrinking it should work just fine
                        self.registers[i as usize] = stored_registry as u16;
                    }
                }
                _ => panic!("Unsupported opcode detected: {:X}", self.opcode),
            },

            // TODO: Impl other opcodes
            _ => panic!("Unsupported opcode detected: {:X}", self.opcode),
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!\n");
            }
            self.sound_timer -= 1;
        }
    }
}

#[cfg(test)]
#[path ="./ops_test.rs"]
mod ops_test;
