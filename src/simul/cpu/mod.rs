mod addressing;
mod instructions;
mod flags;

#[cfg(test)]
mod test;

use simul::memory;

// CPU Implemented as a state machine.
pub struct CPU {
    // Connection to main memory.
    pub memory: memory::RAM,

    // Accumulator
    pub a: u8,

    // X Index Register
    pub x: u8,

    // Y Index Register
    pub y: u8,

    // Stack Pointer
    pub sp: u8,

    // Program Counter
    pub pc: u16,

    // Processor Flags NV_BDIZC
    p: flags::ProcessorFlags,
}

pub fn new(memory: memory::RAM) -> CPU {
    CPU {
        memory,
        a: 0,
        x: 0,
        y: 0,
        sp: 0,
        pc: 0,
        p: flags::new(),
    }
}

impl CPU {
    // Returns number of elapsed cycles.
    pub fn execute_next_instruction(&mut self) -> u32 {
        let opcode = self.memory.load(self.pc);
        self.pc += 1;
        let (operation, addressing_mode) = CPU::decode_instruction(opcode);
        let op_cycles = operation(self, addressing_mode);

        // +1 for loading the opcode itself.
        op_cycles + 1
    }

    fn decode_instruction(opcode: u8) -> (instructions::Operation, addressing::AddressingMode) {
        match opcode {
            // LDA
            0xA9 => (instructions::lda, addressing::immediate),
            0xA5 => (instructions::lda, addressing::zero_page),
            0xB5 => (instructions::lda, addressing::zero_page_indexed),
            0xAD => (instructions::lda, addressing::absolute),
            0xBD => (instructions::lda, addressing::absolute_indexed_x),
            0xB9 => (instructions::lda, addressing::absolute_indexed_y),
            0xA1 => (instructions::lda, addressing::indexed_indirect),
            0xB1 => (instructions::lda, addressing::indirect_indexed),

            // STA
            0x85 => (instructions::sta, addressing::zero_page),
            0x95 => (instructions::sta, addressing::zero_page_indexed),
            0x8D => (instructions::sta, addressing::absolute),

            _ => panic!("Unknown opcode: {:X}", opcode)
        }
    }

    pub fn load_memory(&self, address: u16) -> u8 {
        self.memory.load(address)
    }

    pub fn store_memory(&mut self, address: u16, byte: u8) {
        self.memory.store(address, byte);
    }
}
