//! # ZM2-VM
//!
//! A Z-Machine Mark 2 (Version 2) Virtual Machine implementation in Rust.
//! This crate provides the necessary structures and logic to load and execute
//! Z-Machine Version 2 story files.

pub mod header;
pub mod memory;
pub mod cpu;
mod opcodes;

use byteorder::BigEndian; // Removed WriteBytesExt as it seems unused now
use std::io::Cursor;
use std::fs::File;
use std::io::Read;

// --- Core Data Structures ---

#[derive(Debug, PartialEq)]
pub enum StoryFileError {
    Io(std::io::ErrorKind),
    MemoryInitialization(String),
    UnsupportedVersion(u16),
    ChecksumMismatch,
    SectionTooLarge(String),
}

impl From<std::io::Error> for StoryFileError {
    fn from(err: std::io::Error) -> Self {
        StoryFileError::Io(err.kind())
    }
}


#[derive(Debug)]
pub struct VirtualMachine {
    memory: memory::Memory,
    cpu: cpu::Cpu,
    running: bool,
}

#[derive(Debug, PartialEq)]
pub enum MemoryError {
    OutOfBounds,
    CpuStackError(cpu::StackError),
    InternalError(String),
    // StackOverflow and StackUnderflow are now part of cpu::StackError
}

impl From<String> for MemoryError {
    fn from(s: String) -> Self {
        if s.to_lowercase().contains("out of bounds") {
            MemoryError::OutOfBounds
        } else {
            MemoryError::InternalError(s)
        }
    }
}

impl From<cpu::StackError> for MemoryError {
    fn from(e: cpu::StackError) -> Self {
        MemoryError::CpuStackError(e)
    }
}


impl VirtualMachine {
    fn push_stack(&mut self, value: u64) -> Result<(), MemoryError> {
        self.cpu.push_value(value, &mut self.memory).map_err(MemoryError::from)
    }

    fn pop_stack(&mut self) -> Result<u64, MemoryError> {
        self.cpu.pop_value(&self.memory).map_err(MemoryError::from)
    }

    fn fetch_operand_type(&mut self) -> Result<u8, MemoryError> {
        let type_byte = self.read_byte(self.cpu.pc)?;
        self.cpu.pc += 1;
        Ok(type_byte)
    }

    fn read_variable_operand(&mut self) -> Result<u8, MemoryError> {
        let var_specifier = self.read_byte(self.cpu.pc)?;
        self.cpu.pc += 1;
        Ok(var_specifier)
    }

    fn get_variable(&mut self, var_spec: u8) -> Result<u64, String> {
        match var_spec {
            0x00 => self.pop_stack().map_err(|e| format!("get_variable (stack pop): {:?}", e)),
            0x01..=0x0F => {
                let local_num = var_spec - 0x01;
                let num_locals_on_stack = self.read_qword(self.cpu.fp).map_err(|e| format!("get_variable (L{}: read num_locals): {:?}", local_num, e))? as u8;
                if local_num >= num_locals_on_stack {
                    return Err(format!("get_variable: Invalid local variable L{} (max {}). FP={:#x}", local_num, num_locals_on_stack -1, self.cpu.fp));
                }
                let addr = self.cpu.fp + 8 * (1 + local_num as u64);
                self.read_qword(addr).map_err(|e| format!("get_variable (L{} at addr {:#x}): {:?}", local_num, addr, e))
            }
            0x10..=0xFF => {
                let global_num = var_spec - 0x10;
                let addr = self.memory.header().globals_table_start + (global_num as u64 * 8);
                self.read_qword(addr).map_err(|e| format!("get_variable (G{} at addr {:#x}): {:?}", global_num, addr, e))
            }
        }
    }

    fn set_variable(&mut self, var_spec: u8, value: u64) -> Result<(), String> {
        match var_spec {
            0x00 => self.push_stack(value).map_err(|e| format!("set_variable (stack push): {:?}", e)),
            0x01..=0x0F => {
                let local_num = var_spec - 0x01;
                let num_locals_on_stack = self.read_qword(self.cpu.fp).map_err(|e| format!("set_variable (L{}: read num_locals): {:?}", local_num, e))? as u8;
                if local_num >= num_locals_on_stack {
                    return Err(format!("set_variable: Invalid local variable L{} (max {}). FP={:#x}", local_num, num_locals_on_stack -1, self.cpu.fp));
                }
                let addr = self.cpu.fp + 8 * (1 + local_num as u64);
                self.write_qword(addr, value).map_err(|e| format!("set_variable (L{} at addr {:#x}): {:?}", local_num, addr, e))
            }
            0x10..=0xFF => {
                let global_num = var_spec - 0x10;
                let addr = self.memory.header().globals_table_start + (global_num as u64 * 8);
                self.write_qword(addr, value).map_err(|e| format!("set_variable (G{} at addr {:#x}): {:?}", global_num, addr, e))
            }
        }
    }

    fn read_operand_value(&mut self, operand_type: u8) -> Result<u64, String> {
        match operand_type {
            0x00 => {
                let value = self.read_qword(self.cpu.pc).map_err(|e| format!("Failed to read LC: {:?}", e))?;
                self.cpu.pc += 8;
                Ok(value)
            }
            0x01 => {
                let value = self.read_byte(self.cpu.pc).map_err(|e| format!("Failed to read SC: {:?}", e))?;
                self.cpu.pc += 1;
                Ok(value as u64)
            }
            0x02 => {
                let var_spec = self.read_byte(self.cpu.pc).map_err(|e| format!("VAR: Failed to read var_spec byte: {:?}", e))?;
                self.cpu.pc += 1;
                self.get_variable(var_spec)
            }
            0x03 => {
                let value = self.read_dword(self.cpu.pc).map_err(|e| format!("Failed to read PADDR: {:?}", e))?;
                self.cpu.pc += 4;
                Ok(value as u64)
            }
            _ => Err(format!("Unknown operand type: {:#04x} at PC={:#x}", operand_type, self.cpu.pc)),
        }
    }

    pub fn read_byte(&self, address: u64) -> Result<u8, MemoryError> {
        self.memory.read_byte(address).map_err(MemoryError::from)
    }

    pub fn write_byte(&mut self, address: u64, value: u8) -> Result<(), MemoryError> {
        self.memory.write_byte(address, value).map_err(MemoryError::from)
    }

    pub fn read_word(&self, address: u64) -> Result<u16, MemoryError> {
        self.memory.read_u16(address).map_err(MemoryError::from)
    }

    pub fn write_word(&mut self, address: u64, value: u16) -> Result<(), MemoryError> {
        self.memory.write_u16(address, value).map_err(MemoryError::from)
    }

    pub fn read_dword(&self, address: u64) -> Result<u32, MemoryError> {
        self.memory.read_u32(address).map_err(MemoryError::from)
    }

    pub fn write_dword(&mut self, address: u64, value: u32) -> Result<(), MemoryError> {
        self.memory.write_u32(address, value).map_err(MemoryError::from)
    }

    pub fn read_qword(&self, address: u64) -> Result<u64, MemoryError> {
        self.memory.read_word(address).map_err(MemoryError::from)
    }

    pub fn write_qword(&mut self, address: u64, value: u64) -> Result<(), MemoryError> {
        self.memory.write_word(address, value).map_err(MemoryError::from)
    }

    const OPCODE_SIZE: u64 = 8;

    pub fn load_story(file_path: &str) -> Result<Self, StoryFileError> {
        let mut file_content = Vec::new();
        File::open(file_path)?.read_to_end(&mut file_content)?;

        let new_memory = memory::Memory::new(file_content)
            .map_err(StoryFileError::MemoryInitialization)?;

        let new_cpu = cpu::Cpu::new(&new_memory);

        Ok(VirtualMachine {
            memory: new_memory,
            cpu: new_cpu,
            running: true,
        })
    }

    pub fn fetch_opcode(&mut self) -> Result<u64, MemoryError> {
        let opcode_val = self.read_qword(self.cpu.pc)?;
        self.cpu.pc += Self::OPCODE_SIZE;
        Ok(opcode_val)
    }

    pub fn decode_and_execute_opcode(&mut self, opcode: u64) -> Result<(), String> {
        match opcode {
            opcodes::OP_NOP => Ok(()),
            opcodes::OP_QUIT => { self.running = false; Ok(()) }
            opcodes::OP_PUSH => {
                let operand_type = self.fetch_operand_type().map_err(|e| format!("PUSH: Type fetch: {:?}", e))?;
                let value = self.read_operand_value(operand_type)?;
                self.push_stack(value).map_err(|e| format!("PUSH: Stack op: {:?}", e))
            }
            opcodes::OP_PULL => {
                let value = self.pop_stack().map_err(|e| format!("PULL: Pop: {:?}", e))?;
                let var_spec = self.read_variable_operand().map_err(|e| format!("PULL: Var spec: {:?}", e))?;
                if var_spec != 0x00 {
                    self.set_variable(var_spec, value).map_err(|e| format!("PULL: Set var: {}", e))?;
                }
                Ok(())
            }
            opcodes::OP_STORE => {
                let var_spec = self.read_variable_operand().map_err(|e| format!("STORE: Failed to read var_spec: {:?}", e))?;
                let operand_type = self.fetch_operand_type().map_err(|e| format!("STORE: Failed to fetch operand type: {:?}", e))?;
                let value = self.read_operand_value(operand_type).map_err(|e| format!("STORE: Failed to read value: {}", e))?;
                self.set_variable(var_spec, value).map_err(|e| format!("STORE: Failed to set variable: {}", e))
            }
            opcodes::OP_LOAD => {
                let source_var_spec = self.read_variable_operand().map_err(|e| format!("LOAD: Failed to read source_var_spec: {:?}", e))?;
                let value = self.get_variable(source_var_spec).map_err(|e| format!("LOAD: Failed to get source variable: {}", e))?;
                let dest_var_spec = self.read_variable_operand().map_err(|e| format!("LOAD: Failed to read dest_var_spec: {:?}", e))?;
                self.set_variable(dest_var_spec, value).map_err(|e| format!("LOAD: Failed to set dest variable: {}", e))
            }
            opcodes::OP_ADD => {
                let type1 = self.fetch_operand_type().map_err(|e| format!("ADD: Failed type1: {:?}", e))?;
                let val1 = self.read_operand_value(type1).map_err(|e| format!("ADD: Failed val1: {}", e))?;
                let type2 = self.fetch_operand_type().map_err(|e| format!("ADD: Failed type2: {:?}", e))?;
                let val2 = self.read_operand_value(type2).map_err(|e| format!("ADD: Failed val2: {}", e))?;
                let result = val1.wrapping_add(val2);
                let store_var_spec = self.read_variable_operand().map_err(|e| format!("ADD: Failed store_var_spec: {:?}", e))?;
                self.set_variable(store_var_spec, result).map_err(|e| format!("ADD: Failed set_variable: {}", e))
            }
            opcodes::OP_SUB => {
                let type1 = self.fetch_operand_type().map_err(|e| format!("SUB: Failed type1: {:?}", e))?;
                let val1 = self.read_operand_value(type1).map_err(|e| format!("SUB: Failed val1: {}", e))?;
                let type2 = self.fetch_operand_type().map_err(|e| format!("SUB: Failed type2: {:?}", e))?;
                let val2 = self.read_operand_value(type2).map_err(|e| format!("SUB: Failed val2: {}", e))?;
                let result = val1.wrapping_sub(val2);
                let store_var_spec = self.read_variable_operand().map_err(|e| format!("SUB: Failed store_var_spec: {:?}", e))?;
                self.set_variable(store_var_spec, result).map_err(|e| format!("SUB: Failed set_variable: {}", e))
            }
            opcodes::OP_JUMP => {
                let offset_val = self.read_word(self.cpu.pc).map_err(|e| format!("JUMP: Offset read: {:?}", e))? as i16;
                self.cpu.pc += 2;
                let new_pc_signed = self.cpu.pc as i64 + offset_val as i64;
                if new_pc_signed < 0 { return Err(format!("JUMP: Negative PC target: {}", new_pc_signed)); }
                self.cpu.pc = new_pc_signed as u64;
                Ok(())
            }
            opcodes::OP_CALL => {
                let p_type = self.fetch_operand_type().map_err(|e| format!("CALL: PADDR Type: {:?}", e))?;
                if p_type != 0x03 { return Err(format!("CALL: Routine address must be PADDR type (0x03), got {:#04x}", p_type));}
                let p_packed = self.read_operand_value(p_type)?;

                let target_addr = self.memory.header().code_section_start + p_packed;
                let num_args_supplied: u8 = 0;
                let store_var = self.read_variable_operand().map_err(|e| format!("CALL: Store var: {:?}", e))?;

                let pc_after_call_operands = self.cpu.pc;
                let aligned_return_pc = (pc_after_call_operands + (Self::OPCODE_SIZE - 1)) & !(Self::OPCODE_SIZE - 1);

                self.push_stack(aligned_return_pc).map_err(|e| format!("CALL: Push PC: {:?}",e))?;
                self.push_stack(self.cpu.fp).map_err(|e| format!("CALL: Push FP: {:?}",e))?;
                self.push_stack(store_var as u64).map_err(|e| format!("CALL: Push store_var_ref: {:?}",e))?;
                self.push_stack(num_args_supplied as u64).map_err(|e| format!("CALL: Push num_args: {:?}",e))?;

                let num_locals_from_routine = self.read_byte(target_addr).map_err(|e| format!("CALL: Read num_locals from routine header: {:?}", e))? as usize;
                self.push_stack(num_locals_from_routine as u64).map_err(|e| format!("CALL: Push num_locals_count: {:?}",e))?;
                self.cpu.fp = self.cpu.sp;

                // Spec: PC is set to target routine address, immediately after the byte specifying number of locals.
                self.cpu.pc = target_addr + 1;
                for _ in 0..num_locals_from_routine {
                    self.push_stack(0).map_err(|e| format!("CALL: Push initial local value: {:?}",e))?;
                }
                Ok(())
            }
            opcodes::OP_RET => {
                let val_type = self.fetch_operand_type().map_err(|e| format!("RET: Val type: {:?}", e))?;
                let value = self.read_operand_value(val_type)?;

                self.cpu.sp = self.cpu.fp;
                let _num_locals_on_stack = self.pop_stack().map_err(|e| format!("RET: pop num_locals_count failed: {:?}",e))? as usize;

                let _num_args_supplied_on_stack = self.pop_stack().map_err(|e| format!("RET: pop num_args failed: {:?}",e))? as usize;
                let store_var_ref = self.pop_stack().map_err(|e| format!("RET: pop store_var_ref failed: {:?}",e))? as u8;
                self.cpu.fp = self.pop_stack().map_err(|e| format!("RET: pop old FP failed: {:?}",e))?;
                self.cpu.pc = self.pop_stack().map_err(|e| format!("RET: pop return PC failed: {:?}",e))?;

                self.set_variable(store_var_ref, value).map_err(|e| format!("RET: set_variable for return value: {}", e))
            }
            opcodes::OP_RTRUE | opcodes::OP_RFALSE => {
                let return_value = if opcode == opcodes::OP_RTRUE { 1 } else { 0 };
                self.cpu.sp = self.cpu.fp;
                let _num_locals_on_stack = self.pop_stack().map_err(|e| format!("RVAL: pop num_locals failed: {:?}",e))? as usize;
                let _num_args_supplied_on_stack = self.pop_stack().map_err(|e| format!("RVAL: pop num_args failed: {:?}",e))? as usize;
                let store_var_ref = self.pop_stack().map_err(|e| format!("RVAL: pop store_var_ref failed: {:?}",e))? as u8;
                self.cpu.fp = self.pop_stack().map_err(|e| format!("RVAL: pop old FP failed: {:?}",e))?;
                self.cpu.pc = self.pop_stack().map_err(|e| format!("RVAL: pop return PC failed: {:?}",e))?;
                self.set_variable(store_var_ref, return_value).map_err(|e| format!("RVAL: set_variable for return: {}", e))
            }
            _ => {
                self.running = false;
                Err(format!("Unknown opcode: {:#018x} at PC={:#010x}", opcode, self.cpu.pc - Self::OPCODE_SIZE ))
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.running = true;
        while self.running {
            let current_pc_before_fetch = self.cpu.pc;
            match self.fetch_opcode() {
                Ok(opcode) => {
                    if let Err(e) = self.decode_and_execute_opcode(opcode) {
                        self.running = false;
                        return Err(format!("Execution error: {} for opcode {:#018x} fetched from PC={:#010x}", e, opcode, current_pc_before_fetch));
                    }
                }
                Err(MemoryError::OutOfBounds) => {
                    self.running = false;
                    return Err(format!("MemoryError::OutOfBounds at PC={:#010x}", current_pc_before_fetch));
                }
                Err(MemoryError::CpuStackError(ref err)) => {
                    self.running = false;
                    return Err(format!("MemoryError::CpuStackError({:?}) at PC={:#010x}", err, current_pc_before_fetch));
                }
                Err(MemoryError::InternalError(s)) => {
                    self.running = false;
                    return Err(format!("MemoryError::InternalError: {} at PC={:#010x}", s, current_pc_before_fetch));
                }
            }
        }
        Ok(())
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use crate::header::create_dummy_header_bytes;

    // Helper function to create story file bytes for testing
    fn create_test_story_file_bytes(
        version: u16, code_start: u64, code_len: u64,
        static_start: u64, static_len: u64,
        dynamic_start: u64, dynamic_len: u64,
        opcodes_to_embed: Option<&[u64]>
    ) -> Vec<u8> {
        let mut story_bytes = create_dummy_header_bytes();
        let mut cursor = Cursor::new(&mut story_bytes);

        // Corrected Offsets based on StoryHeader struct:
        cursor.set_position(0); cursor.write_u16::<BigEndian>(version).unwrap(); // version: 0
        cursor.set_position(20); cursor.write_u64::<BigEndian>(code_start).unwrap(); // code_section_start: 20
        cursor.set_position(28); cursor.write_u64::<BigEndian>(code_len).unwrap(); // code_section_length: 28
        cursor.set_position(36); cursor.write_u64::<BigEndian>(static_start).unwrap(); // static_data_section_start: 36
        cursor.set_position(44); cursor.write_u64::<BigEndian>(static_len).unwrap(); // static_data_section_length: 44
        cursor.set_position(52); cursor.write_u64::<BigEndian>(dynamic_start).unwrap(); // dynamic_data_section_start: 52
        cursor.set_position(60); cursor.write_u64::<BigEndian>(dynamic_len).unwrap(); // dynamic_data_section_length: 60

        drop(cursor);
        let required_len_after_header = std::cmp::max(
            code_start + code_len,
            std::cmp::max(static_start + static_len, dynamic_start + dynamic_len)
        );
        let total_required_story_size = std::cmp::max(1024, required_len_after_header) as usize;
        if story_bytes.len() < total_required_story_size {
            story_bytes.resize(total_required_story_size, 0xDA);
        }

        if let Some(ops) = opcodes_to_embed {
            if code_start >= 1024 && code_len > 0 {
                if code_start as usize >= story_bytes.len() {
                     story_bytes.resize(code_start as usize + code_len as usize, 0xDA);
                } else if (code_start + code_len) as usize > story_bytes.len() {
                     story_bytes.resize((code_start + code_len) as usize, 0xDA);
                }
                let mut op_cursor = Cursor::new(&mut story_bytes[code_start as usize..]);
                for &op_val in ops {
                    if op_cursor.position() < code_len {
                        op_cursor.write_u64::<BigEndian>(op_val).unwrap();
                    } else { break; }
                }
            }
        } else {
            if code_len > 0 && code_start >= 1024 {
                 for i in 0..code_len {
                    let current_addr = (code_start + i) as usize;
                    if current_addr < story_bytes.len() { story_bytes[current_addr] = 0xC0; }
                    else { break; }
                 }
            }
            if static_len > 0 && static_start >= 1024 {
                for i in 0..static_len {
                     let current_addr = (static_start + i) as usize;
                    if current_addr < story_bytes.len() { story_bytes[current_addr] = 0x5A; }
                    else { break; }
                }
            }
        }
        story_bytes
    }

    #[test]
    fn test_load_story_valid_minimal() {
        let code_start = 1024;
        let code_len = 0; // No actual opcodes needed for this test
        let story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, code_len,
            code_start + code_len, 0, // static
            code_start + code_len, 256, // dynamic (needs some space for stack)
            None
        );
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let result = VirtualMachine::load_story(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "Failed to load minimal valid story: {:?}", result.err());
        if let Ok(vm) = result {
            assert_eq!(vm.cpu.pc, vm.memory.header().code_section_start); // PC should be at start of code
        }
    }

    #[test]
    fn test_load_story_file_too_small() {
        let story_data = vec![0u8; 512];
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_data).unwrap();
        let result = VirtualMachine::load_story(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(StoryFileError::MemoryInitialization(_))));
    }
    #[test]
    fn test_load_story_unsupported_version() {
        let story_bytes = create_test_story_file_bytes(0x0100, 1024, 0, 1024, 0, 1024, 0, None);
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let result = VirtualMachine::load_story(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(StoryFileError::MemoryInitialization(_))));
        if let Err(StoryFileError::MemoryInitialization(s)) = result {
            assert!(s.contains("Unsupported Z-machine version"));
        }
    }

    #[test]
    fn test_op_quit_immediate() {
        let code_start = 1024;
        let opcodes_to_run = [opcodes::OP_QUIT];
        let story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, // code_len
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 0, // static
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 256, // dynamic
            Some(&opcodes_to_run)
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let mut vm = VirtualMachine::load_story(temp_file.path().to_str().unwrap()).unwrap();

        let initial_pc = vm.cpu.pc;
        let run_result = vm.run();
        assert!(run_result.is_ok());
        assert!(!vm.running);
        assert_eq!(vm.cpu.pc, initial_pc + VirtualMachine::OPCODE_SIZE); // PC advanced past QUIT
    }

    #[test]
    fn test_op_nop_quit() {
        let code_start = 1024;
        let opcodes_to_run = [opcodes::OP_NOP, opcodes::OP_QUIT];
        let story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE,
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 0,
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 256,
            Some(&opcodes_to_run)
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let mut vm = VirtualMachine::load_story(temp_file.path().to_str().unwrap()).unwrap();

        let initial_pc = vm.cpu.pc;
        let run_result = vm.run();
        assert!(run_result.is_ok());
        assert!(!vm.running);
        assert_eq!(vm.cpu.pc, initial_pc + 2 * VirtualMachine::OPCODE_SIZE); // PC advanced past NOP and QUIT
    }

    #[test]
    fn test_unknown_opcode() {
        let code_start = 1024;
        let unknown_opcode_val = 0xFFFFFFFFFFFFFFFF; // An unlikely valid opcode
        let opcodes_to_run = [unknown_opcode_val];
        let story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE,
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 0,
            code_start + opcodes_to_run.len() as u64 * VirtualMachine::OPCODE_SIZE, 256,
            Some(&opcodes_to_run)
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let mut vm = VirtualMachine::load_story(temp_file.path().to_str().unwrap()).unwrap();

        let run_result = vm.run();
        assert!(run_result.is_err());
        if let Err(e) = run_result {
            assert!(e.contains("Unknown opcode"));
        }
        assert!(!vm.running);
    }

    // test_stack_operations was a placeholder, removing.

    #[test]
    fn test_op_push_pull() {
        let code_start = 1024u64;
        let mut instruction_stream = Vec::new();

        // Instruction 1: PUSH SC 5
        instruction_stream.extend_from_slice(&opcodes::OP_PUSH.to_be_bytes()); // 8 bytes
        instruction_stream.push(0x01); // Type: SC (1 byte)
        instruction_stream.push(5);   // Value: 5 (1 byte)
        // PC advances by 8 (fetch_opcode) + 1 (fetch_operand_type) + 1 (read_operand_value for SC) = 10 bytes total consumed by this logical instruction.

        // Instruction 2: PUSH LC 0x100
        instruction_stream.extend_from_slice(&opcodes::OP_PUSH.to_be_bytes()); // 8 bytes
        instruction_stream.push(0x00); // Type: LC (1 byte)
        instruction_stream.extend_from_slice(&0x100u64.to_be_bytes()); // Value: 0x100 (8 bytes)
        // PC advances by 8 (fetch_opcode) + 1 (fetch_operand_type) + 8 (read_operand_value for LC) = 17 bytes total.

        // Instruction 3: PULL to stack
        instruction_stream.extend_from_slice(&opcodes::OP_PULL.to_be_bytes()); // 8 bytes
        instruction_stream.push(0x00); // Var_spec: stack (1 byte)
        // PC advances by 8 (fetch_opcode) + 1 (read_variable_operand) = 9 bytes total.

        // Instruction 4: PULL to stack
        instruction_stream.extend_from_slice(&opcodes::OP_PULL.to_be_bytes()); // 8 bytes
        instruction_stream.push(0x00); // Var_spec: stack (1 byte)
        // PC advances by 8 (fetch_opcode) + 1 (read_variable_operand) = 9 bytes total.

        // Instruction 5: QUIT
        instruction_stream.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes()); // 8 bytes
        // PC advances by 8 (fetch_opcode).

        let actual_code_len = instruction_stream.len() as u64;
        let static_start = code_start + actual_code_len;
        let static_len = 0;
        let dynamic_start = static_start + static_len;
        let dynamic_len = 256;

        let mut story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, actual_code_len,
            static_start, static_len,
            dynamic_start, dynamic_len,
            None
        );

        if code_start as usize + actual_code_len as usize <= story_bytes.len() {
            story_bytes[code_start as usize .. (code_start + actual_code_len) as usize].copy_from_slice(&instruction_stream);
        } else {
            panic!("Instruction stream too large for allocated story_bytes code section");
        }

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let mut vm = VirtualMachine::load_story(file_path).unwrap();

        let initial_sp = vm.cpu.sp;
        let run_result = vm.run();
        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running);
        assert_eq!(vm.cpu.sp, initial_sp);
    }

    #[test]
    fn test_op_jump() {
        let code_start = 1024;
        let op_size = VirtualMachine::OPCODE_SIZE;

        // Target for jump: after the JUMP instruction and its operands, then a NOP, then QUIT
        // JUMP opcode (8 bytes) + offset (2 bytes) = 10 bytes. Padded to op_size (8, assuming op_size is 8).
        // Let's make JUMP block 16 bytes to be safe with padding.
        // JUMP -> NOP -> QUIT
        // Offset for JUMP will be to NOP. NOP is at PC + 16 (JUMP block).
        // JUMP operands take 2 bytes. PC after fetch is JUMP_ADDR + 8. Operands start there.
        // Offset = (Addr of NOP) - (Addr of JUMP + 8 + 2)
        // Addr of NOP = code_start + 16
        // Addr of JUMP + 8 + 2 = code_start + 10
        // Offset = (code_start + 16) - (code_start + 10) = 6
        // This offset is relative to PC *after* reading the offset.

        let jump_offset: i16 = 6; // Jump from after JUMP's own operands to NOP

        let mut instruction_stream = Vec::new();
        // JUMP Opcode Block
        instruction_stream.extend_from_slice(&opcodes::OP_JUMP.to_be_bytes()); // 8 bytes
        instruction_stream.extend_from_slice(&jump_offset.to_be_bytes());    // 2 bytes for offset
        while instruction_stream.len() % op_size as usize != 0 { instruction_stream.push(0); } // Pad to op_size

        // Target of Jump: NOP
        let nop_pc_offset = instruction_stream.len() as u64;
        instruction_stream.extend_from_slice(&opcodes::OP_NOP.to_be_bytes());    // 8 bytes

        // After NOP: QUIT
        let quit_pc_offset = instruction_stream.len() as u64;
        instruction_stream.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes());   // 8 bytes

        let actual_code_len = instruction_stream.len() as u64;

        let story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, actual_code_len,
            code_start + actual_code_len, 0,
            code_start + actual_code_len, 256, // dynamic for stack
            None // Manual embedding below
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        let mut mutable_story_bytes = story_bytes.clone(); // Clone to make it mutable
        mutable_story_bytes[code_start as usize..(code_start + actual_code_len) as usize].copy_from_slice(&instruction_stream);
        temp_file.write_all(&mutable_story_bytes).unwrap();

        let mut vm = VirtualMachine::load_story(temp_file.path().to_str().unwrap()).unwrap();
        let run_result = vm.run();

        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running, "VM should have quit");
        // Expected PC: after the NOP (jump target) and the QUIT
        assert_eq!(vm.cpu.pc, code_start + quit_pc_offset + op_size, "PC not after QUIT following JUMP and NOP");
    }

    #[test]
    fn test_op_call_ret_simple() {
        let op_size = VirtualMachine::OPCODE_SIZE;
        let code_start = 1024u64;

        let mut routine_bytes: Vec<u8> = Vec::new();
        routine_bytes.push(0);
        while routine_bytes.len() < op_size as usize { routine_bytes.push(0); }

        let mut ret_op_block: Vec<u8> = Vec::new();
        ret_op_block.extend_from_slice(&opcodes::OP_RET.to_be_bytes());
        ret_op_block.push(0x01);
        ret_op_block.push(42);
        while ret_op_block.len() < op_size as usize { ret_op_block.push(0); }
        routine_bytes.extend_from_slice(&ret_op_block);

        let mut main_code_stream: Vec<u8> = Vec::new();
        main_code_stream.extend_from_slice(&opcodes::OP_CALL.to_be_bytes());
        main_code_stream.push(0x03);

        let p_addr_for_call = (16 + 8) as u32;


        main_code_stream.extend_from_slice(&p_addr_for_call.to_be_bytes());
        main_code_stream.push(0x00);
        while main_code_stream.len() % op_size as usize != 0 { main_code_stream.push(0); }
        // main_code_stream is now 16 bytes.

        main_code_stream.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes());
        // main_code_stream is now 24 bytes.

        let mut full_code_section_bytes = main_code_stream.clone();
        full_code_section_bytes.extend_from_slice(&routine_bytes);

        let total_code_len = full_code_section_bytes.len() as u64;

        let static_start = code_start + total_code_len;
        let static_len = 0;
        let dynamic_start = static_start + static_len;
        let dynamic_len = 256;

        let mut story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, total_code_len,
            static_start, static_len,
            dynamic_start, dynamic_len,
            None
        );

        story_bytes[code_start as usize .. (code_start + total_code_len) as usize]
            .copy_from_slice(&full_code_section_bytes);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let mut vm = VirtualMachine::load_story(file_path).unwrap();

        let initial_sp = vm.cpu.sp;
        let run_result = vm.run();

        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running, "VM should have quit");

        let op_size_u64 = VirtualMachine::OPCODE_SIZE;
        let call_instr_padded_len = op_size_u64 * 2;
        let quit_instr_len = op_size_u64;

        assert_eq!(vm.cpu.pc, code_start + call_instr_padded_len + quit_instr_len, "PC is not after QUIT");

        assert_eq!(vm.cpu.sp, initial_sp - 8, "SP not pointing to return value on stack");
        let return_val_on_stack = vm.read_qword(vm.cpu.sp).unwrap();
        assert_eq!(return_val_on_stack, 42, "Return value from CALL was not 42");
    }

    #[test]
    fn test_op_rtrue_rfalse() {
        let op_size = VirtualMachine::OPCODE_SIZE;
        let code_start = 1024u64;

        // Routine 1: RTRUE
        let mut rtrue_routine_bytes: Vec<u8> = Vec::new();
        rtrue_routine_bytes.push(0); // 0 locals
        while rtrue_routine_bytes.len() < op_size as usize { rtrue_routine_bytes.push(0); } // Pad num_locals byte to opcode size
        rtrue_routine_bytes.extend_from_slice(&opcodes::OP_RTRUE.to_be_bytes()); // RTRUE opcode

        // Routine 2: RFALSE
        let mut rfalse_routine_bytes: Vec<u8> = Vec::new();
        rfalse_routine_bytes.push(0); // 0 locals
        while rfalse_routine_bytes.len() < op_size as usize { rfalse_routine_bytes.push(0); } // Pad
        rfalse_routine_bytes.extend_from_slice(&opcodes::OP_RFALSE.to_be_bytes()); // RFALSE opcode

        let rtrue_routine_start_offset = op_size * 2; // After CALL to RTRUE and CALL to RFALSE in main
        let rfalse_routine_start_offset = rtrue_routine_start_offset + rtrue_routine_bytes.len() as u64;

        // Main code stream
        let mut main_code_stream: Vec<u8> = Vec::new();
        // CALL RTRUE, store result to stack
        main_code_stream.extend_from_slice(&opcodes::OP_CALL.to_be_bytes());
        main_code_stream.push(0x03); // PADDR type for routine address
        main_code_stream.extend_from_slice(&(rtrue_routine_start_offset as u32).to_be_bytes()); // PADDR value
        main_code_stream.push(0x00); // Store result to stack
        while main_code_stream.len() % op_size as usize != 0 { main_code_stream.push(0); } // Pad CALL block

        // CALL RFALSE, store result to stack
        main_code_stream.extend_from_slice(&opcodes::OP_CALL.to_be_bytes());
        main_code_stream.push(0x03); // PADDR type
        main_code_stream.extend_from_slice(&(rfalse_routine_start_offset as u32).to_be_bytes()); // PADDR value
        main_code_stream.push(0x00); // Store result to stack
        while main_code_stream.len() % op_size as usize != 0 { main_code_stream.push(0); } // Pad CALL block

        let call_rtrue_block_len = op_size * 2; // Padded call
        let call_rfalse_block_len = op_size * 2; // Padded call

        // QUIT
        main_code_stream.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes());

        let mut full_code_section_bytes = main_code_stream;
        full_code_section_bytes.extend_from_slice(&rtrue_routine_bytes);
        full_code_section_bytes.extend_from_slice(&rfalse_routine_bytes);

        let total_code_len = full_code_section_bytes.len() as u64;

        let mut story_bytes = create_test_story_file_bytes(
            memory::SUPPORTED_VERSION,
            code_start, total_code_len,
            code_start + total_code_len, 0, // static
            code_start + total_code_len, 256, // dynamic for stack
            None // Manual embedding
        );
        story_bytes[code_start as usize .. (code_start + total_code_len) as usize]
            .copy_from_slice(&full_code_section_bytes);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let mut vm = VirtualMachine::load_story(temp_file.path().to_str().unwrap()).unwrap();

        let initial_sp = vm.cpu.sp;
        let run_result = vm.run();

        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running, "VM should have quit");

        // PC should be after the two CALL blocks and the QUIT block
        let expected_pc = code_start + call_rtrue_block_len + call_rfalse_block_len + op_size;
        assert_eq!(vm.cpu.pc, expected_pc, "PC not after QUIT");

        // Check stack for results (RFALSE result, then RTRUE result)
        // RFALSE was called second, its result (0) should be on top if stack grows down.
        // RTRUE was called first, its result (1) should be below RFALSE's result.
        // SP currently points to the last pushed item (0 from RFALSE).
        // After two calls storing results to the stack, SP should be initial_sp - 16.
        assert_eq!(vm.cpu.sp, initial_sp - 16, "SP not reflecting two values pushed for two CALLs storing to stack");
        let rfalse_ret_val = vm.read_qword(vm.cpu.sp).unwrap(); // RFALSE result is on top
        assert_eq!(rfalse_ret_val, 0, "Return value from RFALSE was not 0");

        let rtrue_ret_val = vm.read_qword(vm.cpu.sp + 8).unwrap(); // RTRUE result is below RFALSE result
        assert_eq!(rtrue_ret_val, 1, "Return value from RTRUE was not 1");
    }
}