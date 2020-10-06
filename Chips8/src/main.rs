fn main() {
    // Render and input system init
    // setupGraphics();
    // setupInput();

    // chips8::initialize();
    // chips8::loadGame("pong");

    // My best guess is that this is representing the clock of the system
    loop {
        // Emulate one cycle
        // chips8::enumlateCycle();

        // If the draw flag is set, update the screen
        // chips8.drawFlag -> drawGraphics()

        // Store key press state (Press and Release)
        // chips8.setKeys();
    }
}

pub struct MyChips8 {
    opcode: u16,         // 2B for storing current opcode
    memory: [u8; 4096],  // Represent 4K block of memory
    registers: [u8; 16], // 15 general purpose and 16th is for carry flag
    I: u16,              // Index register
    pc: u16,             // Program Counter

    gfx: [u8; 64 * 32], // Stores whether pixel[idx] is on or off (1 or 0)
    delay_timer: u8,    // Will cound down to 0 when > 0
    sound_timer: u8,    // Will count down to 0 when > 0

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
            I: 0x0,
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

    pub fn enumlate_cycle(&mut self) {
        // Fetch
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        // Decode
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x00FF {
                    // 0x0000: Calls machine code routine at addr NNN
                    0x0000 => {}
                    // 0x00E0: Clears the screen
                    0x00E0 => {}
                    // 0x00EE: Returns from subroutine
                    0x00EE => {}
                    _ => panic!("Unsupported opcode detected"),
                }
            }

            // 0x1NNN: JMP to addr NNN
            0x1000 => {
                self.I = self.opcode & 0x0FFF;
            }

            // 0x2NNN: Calls subroutine at address NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }

            // 0x3XNN: Skip next if VX eq NN
            0x3000 => {
                // const VX = self.opcode &
            }

            0x4000 => {}

            0x5000 => {}

            0x6000 => {}

            0x7000 => {}

            0x8000 => match self.opcode & 0x000F {
                0x0001 => {}
                0x0002 => {}
                0x0003 => {}
                // 0x8XY4: Add VY to VX, VF set to 1 if carry else to 0
                0x0004 => {
                    if self.registers[((self.opcode & 0x00F0) >> 4) as usize]
                        > (0xFF - self.registers[((self.opcode & 0x0F00) >> 4) as usize])
                    {
                        self.registers[0xF] = 0x1;
                    } else {
                        self.registers[0xF] = 0x0;
                    }
                    self.registers[((self.opcode & 0x0F00) >> 8) as usize] +=
                        self.registers[((self.opcode & 0x00F0) >> 4) as usize];
                    self.pc += 2;
                }
                0x0005 => {}
                0x0006 => {}
                0x0007 => {}
                0x000E => {}
                _ => {}
            },

            // 0xANNN: Sets I to the address NNN
            0xA000 => {
                self.I = self.opcode & 0x0FFF;
                self.pc += 2;
            }

            0xB000 => {}

            0xC000 => {}

            // Draw
            0xD000 => {}

            0xE000 => match self.opcode & 0x00FF {
                0x009E => {}
                0x00A1 => {}
                _ => {}
            },

            0xF000 => match self.opcode & 0x00FF {
                0x0007 => {}
                0x000A => {}
                0x0015 => {}
                0x0018 => {}
                0x001E => {}
                0x0029 => {
                    // TODO: Implement sprint loading
                }
                // 0xFX33 set_BCD
                0x0033 => {
                    self.memory[self.I as usize] =
                        self.registers[((self.opcode & 0x0F00) >> 8) as usize] / 100;
                    self.memory[(self.I + 1) as usize] =
                        (self.registers[((self.opcode & 0x0F00) >> 8) as usize] / 10) % 10;
                    self.memory[(self.I + 2) as usize] =
                        (self.registers[((self.opcode & 0x0F00) >> 8) as usize] % 100) % 10;
                }
                // 0xFX55 reg_dump
                0x0055 => {}
                // 0xFX65 reg_load
                0x0065 => {}
                _ => {}
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
