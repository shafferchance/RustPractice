pub struct MyChips8 {
    opcode: u16,          // 2B for storing current opcode
    memory: [u8; 4096],   // Represent 4K block of memory
    registers: [u16; 16], // 15 general purpose and 16th is for carry flag
    i: u16,               // Index register
    pc: u16,              // Program Counter

    gfx: [u8; 64 * 32], // Stores whether pixel[idx] is on or off (1 or 0)
    delay_timer: u16,   // Will cound down to 0 when > 0
    sound_timer: u16,   // Will count down to 0 when > 0

    stack: [u16; 16], // Storing before JMP, ensure that PC is saved as well
    sp: u16,          // Stack Pointer
    key: [u8; 16],    // HEX Based keypad, this is used to store state
}

impl MyChips8 {
    pub fn new() -> Self {
        MyChips8 {
            opcode: 0x0,
            memory: [0; 4096],
            registers: [0; 16],
            i: 0x0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,

            stack: [0x0; 16],
            sp: 0x0,
            key: [0; 16],
        }
    }

    // pub fn initialize(&self) {
    // Reset register, memory, stack

    // Load fontsets

    // Reset Timers
    // }

    // TODO: Decode to function pointers
    // fn decodeOpcode (opcode: u16) {
    // }

    fn get_kk(&self) -> u16 {
        return self.opcode & 0x00FF;
    }

    fn get_nibble(&self) -> u16 {
        return self.opcode & 0x000F;
    }

    fn get_nnn(&self) -> u16 {
        return self.opcode & 0x0FFF;
    }

    fn get_vx(&self) -> u16 {
        return (self.opcode & 0x0F00) >> 4;
    }

    fn get_vy(&self) -> u16 {
        return (self.opcode & 0x00F0) >> 4;
    }

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
            | (self.memory[(self.pc + 1) as usize] as u16);
        // Decode
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x00FF {
                    // Deprecated on modern interpreter
                    // 0x0000 - SYS addr - Calls machine code routine at addr NNN
                    0x0000 => {}
                    // 0x00E0 - CLS - Clears the screen
                    0x00E0 => {}
                    // 0x00EE - RET - Returns from subroutine
                    0x00EE => {}
                    _ => panic!("Unsupported opcode detected"),
                }
            }

            // 0x1nnn - JP addr - JMP to addr nnn
            0x1000 => {
                self.i = self.get_nnn();
                self.pc = self.get_nnn();
            }

            // 0x2nnn - CALL addr - Calls subroutine at address nnn
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = self.get_nnn();
            }

            // 0x3xkk - SE Vx, byte - Skip next if Vx == kk
            0x3000 => {
                if self.get_vx() == self.get_kk() {
                    self.pc += 2;
                } else {
                    self.pc += 1;
                }
            }

            // 0x4xkk - SNE Vx, byte - Skip next if Vx != kk
            0x4000 => {
                if self.get_vx() != self.get_kk() {
                    self.pc += 2;
                } else {
                    self.pc += 1;
                }
            }

            // 0x5xy0 - SE Vx, Vy - Skip next if Vx == Vy
            0x5000 => {
                if self.get_vx() == self.get_vy() {
                    self.pc += 2;
                } else {
                    self.pc += 1;
                }
            }

            // 0x6xkk - LD Vx, byte - Load kk into Vx
            0x6000 => {
                self.set_register(self.get_vx(), self.get_kk());
                self.pc += 1;
            }

            // 0x7xkk - ADD Vx, byte - Add value of byte
            0x7000 => {
                self.set_register(
                    self.get_vx(),
                    self.get_register_value(self.get_vx()) + self.get_kk(),
                );
                self.pc += 1;
            }

            0x8000 => match self.opcode & 0x000F {
                // LD Vx, Vy
                0x0000 => {
                    self.set_register(self.get_vx(), self.get_register_value(self.get_vy()));
                    self.pc += 1;
                }
                // OR Vx, Vy
                0x0001 => {
                    self.set_register(self.get_vx(), self.get_vx() | self.get_vy());
                    self.pc += 1;
                }
                // AND Vx, Vy
                0x0002 => {
                    self.set_register(self.get_vx(), self.get_vx() & self.get_vy());
                    self.pc += 1;
                }
                // XOR Vx, Vy
                0x0003 => {
                    self.set_register(self.get_vx(), self.get_vx() ^ self.get_vy());
                    self.pc += 1;
                }
                // ADD Vx, Vy - Add Vx to Vy
                0x0004 => {
                    if self.get_register_value(self.get_vy())
                        > (0xFF - self.get_register_value(self.get_vx()))
                    {
                        self.registers[0xF] = 0x1;
                    } else {
                        self.registers[0xF] = 0x0;
                    }

                    self.set_register(
                        self.get_vx(),
                        self.get_register_value(self.get_vx())
                            + self.get_register_value(self.get_vy()),
                    );
                    self.pc += 1;
                }
                // SUB Vx, Vy - Subtract Vx from Vy
                0x0005 => {
                    if self.get_register_value(self.get_vx())
                        > self.get_register_value(self.get_vy())
                    {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(
                        self.get_vx(),
                        self.get_register_value(self.get_vx())
                            - self.get_register_value(self.get_vy()),
                    );
                    self.pc += 1;
                }
                // SHR Vx - RHS 1
                0x0006 => {
                    // Checking LSB of Vx
                    if (self.get_vx() & 0x1) == 1 {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(self.get_vx(), self.get_register_value(self.get_vx()) >> 1);
                    self.pc += 1;
                }
                // SUBN Vx, Vy - Subtract Vy from Vx
                0x0007 => {
                    if self.get_register_value(self.get_vy())
                        > self.get_register_value(self.get_vx())
                    {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(
                        self.get_vx(),
                        self.get_register_value(self.get_vy())
                            - self.get_register_value(self.get_vx()),
                    );
                    self.pc += 1;
                }
                // SHL Vx - LHS 1
                0x000E => {
                    if (self.get_vx() & 0x8) == 1 {
                        self.set_register(0xF, 0x1);
                    } else {
                        self.set_register(0xF, 0x0);
                    }

                    self.set_register(self.get_vx(), self.get_register_value(self.get_vx()) << 1);
                    self.pc += 1;
                }
                _ => panic!("Unsupportd opcode detected"),
            },

            // 0x9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            0x9000 => {
                if self.get_register_value(self.get_vx()) != self.get_register_value(self.get_vy())
                {
                    self.pc += 2;
                } else {
                    self.pc += 1;
                }
            }

            // 0xAnnn - LD I, addr - Sets I to the address nnn
            0xA000 => {
                self.i = self.get_nnn();
                self.pc += 1;
            }

            // Bnnn - JP V0, addr - Jump to location addr
            0xB000 => {
                self.pc = self.get_nnn() + self.get_register_value(0x0);
            }

            // Cxkk - RND Vx, byte - Random number from 0 to 255, then &'d w/ byte which is stored into Vx
            0xC000 => {
                self.set_register(
                    self.get_vx(),
                    self.get_rand((self.get_kk() as u8) & self.get_kk() as u8) as u16,
                );
                self.pc += 1;
            }

            // 0xDxyn - DRW Vx, Vy, nibble - Draw n-byte sprite starting at mem loc I @ (vx, Vy), set VF = collision
            0xD000 => {
                for (i, &pixel) in self.memory
                    [self.i as usize..(self.i + self.get_nibble()) as usize]
                    .iter()
                    .enumerate()
                {
                    // Collision check
                    if self.gfx[(self.get_vx() * self.get_vy()) as usize] == 1 && pixel == 1 {
                        self.registers[0xF] = 0x1;
                    } else {
                        self.registers[0xF] = 0x0;
                    }
                    // Wrap around logic, where both (x,y) are out of bounds
                    match (self.get_vx(), self.get_vy()) {
                        (x, y) if x > 63 && y > 31 => {
                            self.gfx[((63 - x) * (31 - y)) as usize + i] =
                                self.gfx[((63 - x) * (31 - y)) as usize + i] ^ pixel;
                        }
                        // (x, y) if x < 0 && y < 0 => {
                        //     self.gfx[((x + 63) * (y + 31)) as usize] =
                        //         self.gfx[((x + 63) * (y + 31)) as usize] ^ pixel;
                        // }
                        (x, y) if x > 63 => {
                            self.gfx[((63 - x) * y) as usize + i] =
                                self.gfx[((63 - x) * y) as usize + i] ^ pixel;
                        }
                        (x, y) if y > 31 => {
                            self.gfx[(x * (31 - y)) as usize + i] =
                                self.gfx[(x * (31 - y)) as usize + i] ^ pixel;
                        }
                        // (x, y) if x < 0 => {
                        //     self.gfx[((x + 63) * y) as usize] =
                        //         self.gfx[((x + 63) * y) as usize] ^ pixel;
                        // }
                        // (x, y) if y < 0 => {
                        //     self.gfx[(x * (y + 31)) as usize] =
                        //         self.gfx[(x * (y + 31)) as usize] ^ pixel;
                        // }
                        (x, y) => {
                            self.gfx[(x * y) as usize + i] = self.gfx[(x * y) as usize + i] ^ pixel;
                        }
                    }
                }
                self.pc += 1;
            }

            0xE000 => match self.opcode & 0x00FF {
                // 0xEx9E - SKP Vx - Skip next instruction if key pressed
                0x009E => {
                    if self.key[self.get_vx() as usize] == 1 {
                        self.pc += 2;
                    } else {
                        self.pc += 1;
                    }
                }

                // 0xExA1 - SKNP Vx - Skip next instruction if key not pressed
                0x00A1 => {
                    if self.key[self.get_vx() as usize] == 0 {
                        self.pc += 2;
                    } else {
                        self.pc += 1;
                    }
                }
                _ => panic!("Unsupported opcode detected"),
            },

            0xF000 => match self.opcode & 0x00FF {
                // 0xFx07 - LD Vx, DT - Loading Delay Timer into Vx
                0x0007 => {
                    self.set_register(self.get_vx(), self.delay_timer);
                    self.pc += 1;
                }
                // 0xFx0A - LD Vx, K - Stop execution till key press
                0x000A => {
                    // TODO: implement coroutine essentially :/
                    println!("Will add implementation soon");
                    self.pc += 1;
                }
                // 0xFx15 - LD DT, Vx - Load Vx into DT
                0x0015 => {
                    self.delay_timer = self.get_register_value(self.get_vx());
                    self.pc += 1;
                }
                // 0xFx18 - LD ST, Vx - Set sound timer to Vx
                0x0018 => {
                    self.sound_timer = self.get_register_value(self.get_vx());
                    self.pc += 1;
                }
                // 0xFx1E - ADD I, Vx - Add I and Vx then store in I
                0x001E => {
                    self.i = self.i + self.get_register_value(self.get_vx());
                    self.pc += 1;
                }
                // 0xFx29 - LD F, Vx - Set I = location of sprite for digit Vx
                0x0029 => {
                    // TODO: implmentation of loading hexidecimal font. Need to choose location in memory
                    println!("Will add implemenation soon");
                    self.pc += 1;
                }
                // 0xFx33 - LD F, Vx - set_BCD
                0x0033 => {
                    self.memory[self.i as usize] =
                        (self.get_register_value(self.get_vx()) / 100) as u8;
                    self.memory[(self.i + 1) as usize] =
                        (((self.get_register_value(self.get_vx()) / 100) / 10) % 10) as u8;
                    self.memory[(self.i + 2) as usize] =
                        (((self.get_register_value(self.get_vx()) / 100) % 100) % 10) as u8;
                    self.pc += 1;
                }
                // 0xFx55 - LD [I], Vx -reg_dump
                0x0055 => {
                    for (i, &registry) in self.registers[0 as usize..self.get_vx() as usize]
                        .iter()
                        .enumerate()
                    {
                        // This might cause a panic :fearful:
                        self.memory[self.i as usize + i] = registry as u8;
                    }
                    self.pc += 1;
                }
                // 0xFx65 - LD Vx, [I] - reg_load
                0x0065 => {
                    for (i, &stored_registry) in self.memory
                        [self.i as usize..self.get_vx() as usize]
                        .iter()
                        .enumerate()
                    {
                        // Since this is growing not shrinking it should work just fine
                        self.registers[i as usize] = stored_registry as u16;
                    }
                    self.pc += 1;
                }
                _ => panic!("Unsupported opcode detected"),
            },

            // TODO: Impl other opcodes
            _ => panic!("Unsupported opcode detected"),
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
