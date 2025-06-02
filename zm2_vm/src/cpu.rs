use crate::memory::Memory;
use crate::header::StoryHeader; // To access header fields for sp, pc initialization

#[derive(Debug)]
pub struct Cpu {
    pub pc: u64, // Program Counter
    pub sp: u64, // Stack Pointer
    pub fp: u64, // Frame Pointer
    initial_sp: u64, // To check for stack underflow
}

#[derive(Debug, PartialEq)]
pub enum StackError {
    Overflow,
    Underflow,
    MemoryAccess(String),
}

impl From<String> for StackError {
    fn from(s: String) -> Self {
        StackError::MemoryAccess(s)
    }
}

impl Cpu {
    pub fn new(memory: &Memory) -> Self {
        let header = memory.header();
        let initial_sp_val = header.dynamic_data_section_start + header.dynamic_data_section_length;
        Cpu {
            pc: header.code_section_start,
            sp: initial_sp_val,
            fp: initial_sp_val, // Typically FP is initialized like SP
            initial_sp: initial_sp_val,
        }
    }

    pub fn push_value(&mut self, value: u64, memory: &mut Memory) -> Result<(), StackError> {
        if self.sp < 8 { // Check if SP can be decremented by 8
            return Err(StackError::Overflow);
        }
        self.sp -= 8;

        let header = memory.header(); // Re-borrow header immutably after mutable borrow for write_word
        if self.sp < header.dynamic_data_section_start {
            self.sp += 8; // Revert SP change before erroring
            return Err(StackError::Overflow);
        }
        memory.write_word(self.sp, value).map_err(StackError::from)
    }

    pub fn pop_value(&mut self, memory: &Memory) -> Result<u64, StackError> {
        if self.sp >= self.initial_sp {
            return Err(StackError::Underflow);
        }
        // No need to check against dynamic_data_section_start + length for pop if initial_sp is the absolute top.
        // SP must be < initial_sp to be valid for pop.
        // If sp is, for example, header.dynamic_data_section_start, it's a valid address to read from.

        let value = memory.read_word(self.sp).map_err(StackError::from)?;
        self.sp += 8;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory; // For creating a Memory instance
    use crate::header::create_dummy_header_bytes; // Test utility

    fn create_test_memory() -> Memory {
        let mut story_data = create_dummy_header_bytes(); // 1024 bytes
        // Configure header for a small dynamic section for stack
        // Default dummy header has dynamic_data_section_start = 1408, length = 64
        // So, dynamic section is 1408 to 1471. Stack grows downwards from 1472.
        // Let's make memory large enough for this.
        // Header points to sections up to ~1538. Let's make memory 2048 bytes.
        story_data.resize(2048, 0xDA);
        Memory::new(story_data).expect("Failed to create test memory")
    }

    fn get_header_from_memory(memory: &Memory) -> &StoryHeader {
        memory.header()
    }


    #[test]
    fn test_cpu_new() {
        let memory = create_test_memory();
        let cpu = Cpu::new(&memory);
        let header = get_header_from_memory(&memory);

        assert_eq!(cpu.pc, header.code_section_start);
        let expected_sp = header.dynamic_data_section_start + header.dynamic_data_section_length;
        assert_eq!(cpu.sp, expected_sp);
        assert_eq!(cpu.fp, expected_sp);
        assert_eq!(cpu.initial_sp, expected_sp);
    }

    #[test]
    fn test_push_pop_value() {
        let mut memory = create_test_memory();
        let mut cpu = Cpu::new(&memory);

        // Push some values
        cpu.push_value(10, &mut memory).unwrap();
        assert_eq!(cpu.sp, cpu.initial_sp - 8);
        let val_at_sp = memory.read_word(cpu.sp).unwrap();
        assert_eq!(val_at_sp, 10);

        cpu.push_value(20, &mut memory).unwrap();
        assert_eq!(cpu.sp, cpu.initial_sp - 16);
        let val_at_sp_2 = memory.read_word(cpu.sp).unwrap();
        assert_eq!(val_at_sp_2, 20);

        // Pop values
        let popped_val_2 = cpu.pop_value(&memory).unwrap();
        assert_eq!(popped_val_2, 20);
        assert_eq!(cpu.sp, cpu.initial_sp - 8);

        let popped_val_1 = cpu.pop_value(&memory).unwrap();
        assert_eq!(popped_val_1, 10);
        assert_eq!(cpu.sp, cpu.initial_sp);
    }

    #[test]
    fn test_stack_underflow() {
        let memory = create_test_memory();
        let mut cpu = Cpu::new(&memory);

        let result = cpu.pop_value(&memory);
        assert_eq!(result, Err(StackError::Underflow));
    }

    #[test]
    fn test_stack_overflow() {
        let mut memory = create_test_memory();
        let mut cpu = Cpu::new(&memory);
        let dynamic_data_start = memory.header().dynamic_data_section_start; // Extract value
        let initial_sp_val = cpu.initial_sp; // Extract value

        // Calculate how many items can be pushed onto the stack
        // Stack grows from initial_sp downwards to dynamic_data_start
        let stack_capacity_bytes = initial_sp_val - dynamic_data_start;
        let stack_capacity_items = stack_capacity_bytes / 8;

        for i in 0..stack_capacity_items {
            assert!(cpu.push_value(i as u64, &mut memory).is_ok(), "Push {} failed", i);
        }

        // Next push should overflow
        let result = cpu.push_value(999, &mut memory);
        assert_eq!(result, Err(StackError::Overflow));

        // SP should be at dynamic_data_section_start after filling capacity
        assert_eq!(cpu.sp, dynamic_data_start);

        // Popping one value should now be possible
        cpu.pop_value(&memory).unwrap();
        assert_eq!(cpu.sp, dynamic_data_start + 8);
    }
     #[test]
    fn test_stack_push_revert_sp_on_overflow() {
        let mut memory = create_test_memory();
        let mut cpu = Cpu::new(&memory);
        let dynamic_data_start = memory.header().dynamic_data_section_start; // Extract value
        let initial_sp_val = cpu.initial_sp; // Extract value


        let stack_capacity_bytes = initial_sp_val - dynamic_data_start;
        let stack_capacity_items = stack_capacity_bytes / 8;

        for i in 0..stack_capacity_items {
            cpu.push_value(i as u64, &mut memory).unwrap();
        }

        let sp_before_overflow_attempt = cpu.sp;
        assert_eq!(sp_before_overflow_attempt, dynamic_data_start);

        let result = cpu.push_value(999, &mut memory); // This should overflow
        assert_eq!(result, Err(StackError::Overflow));

        // SP should be reverted to its value before the failed push
        assert_eq!(cpu.sp, sp_before_overflow_attempt, "SP not reverted after overflow");
    }
}
