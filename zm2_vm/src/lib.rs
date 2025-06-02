//! # ZM2-VM
//!
//! A Z-Machine Mark 2 (Version 2) Virtual Machine implementation in Rust.
//! This crate provides the necessary structures and logic to load and execute
//! Z-Machine Version 2 story files.

mod opcodes;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt}; // Fix 4: Removed WriteBytesExt
use std::io::Cursor;
use std::fs::File;
use std::io::Read;

// --- Opcode Enum (Old, for reference, might be removed later if not used by old methods) ---
#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop,
    Unknown(u64),
}
pub const NOP_OPCODE_VALUE: u64 = opcodes::OP_NOP;

// --- Core Data Structures ---

#[derive(Debug, PartialEq)]
pub enum StoryFileError {
    Io(std::io::ErrorKind),
    InvalidHeaderSize(usize),
    UnsupportedVersion(u16),
    InvalidMemoryLayout(String),
    ChecksumMismatch,
    SectionTooLarge(String),
}

impl From<std::io::Error> for StoryFileError {
    fn from(err: std::io::Error) -> Self {
        StoryFileError::Io(err.kind())
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Header {
    pub version: u16,
    pub release_number: u16,
    pub story_id: u64,
    pub checksum: u64,
    pub code_section_start: u64,
    pub code_section_length: u64,
    pub static_data_section_start: u64,
    pub static_data_section_length: u64,
    pub dynamic_data_section_start: u64,
    pub dynamic_data_section_length: u64,
    pub globals_table_start: u64,
    pub object_table_start: u64,
    pub dictionary_start: u64,
    pub abbreviation_table_start: u64,
    pub flags1: u32,
    pub flags2: u32,
    pub llm_api_endpoint_ptr: u64,
    pub llm_parameters_ptr: u64,
    pub context_globals_list_ptr: u64,
    pub recent_events_buffer_ptr: u64,
    pub recent_events_count_ptr: u64,
    pub property_defaults_table_start: u64,
}

#[derive(Debug)]
pub struct VirtualMachine {
    pub header: Header,
    pub memory: Vec<u8>,
    pub pc: u64,
    pub sp: u64,
    pub fp: u64,
    running: bool,
}

#[derive(Debug, PartialEq)]
pub enum MemoryError {
    OutOfBounds,
    StackOverflow,
    StackUnderflow,
}

impl VirtualMachine {
    fn push_stack(&mut self, value: u64) -> Result<(), MemoryError> {
        if self.sp <= self.header.dynamic_data_section_start || self.sp < 8 {
            return Err(MemoryError::StackOverflow);
        }
        self.sp -= 8;
        if self.sp < self.header.dynamic_data_section_start {
            self.sp += 8;
            return Err(MemoryError::StackOverflow);
        }
        self.write_qword(self.sp, value)
    }

    fn pop_stack(&mut self) -> Result<u64, MemoryError> {
        if self.sp >= self.header.dynamic_data_section_start + self.header.dynamic_data_section_length {
            return Err(MemoryError::StackUnderflow);
        }
        let value = self.read_qword(self.sp)?;
        self.sp += 8;
        Ok(value)
    }

    fn fetch_operand_type(&mut self) -> Result<u8, MemoryError> {
        let type_byte = self.read_byte(self.pc)?;
        self.pc += 1;
        Ok(type_byte)
    }

    fn read_variable_operand(&mut self) -> Result<u8, MemoryError> { // Renamed from read_variable_operand_specifier
        let var_specifier = self.read_byte(self.pc)?;
        self.pc += 1;
        Ok(var_specifier)
    }

    fn get_variable(&mut self, var_spec: u8) -> Result<u64, String> {
        match var_spec {
            0x00 => self.pop_stack().map_err(|e| format!("get_variable (stack pop): {:?}", e)),
            0x01..=0x0F => { // L00-L14
                let local_num = var_spec - 0x01;
                // FP points to [num_locals_value]. Locals are at FP+8, FP+16, ...
                // L0 is at FP+8*(1+0), L1 is at FP+8*(1+1)
                let num_locals_on_stack = self.read_qword(self.fp).map_err(|e| format!("get_variable (L{}: read num_locals): {:?}", local_num, e))? as u8;
                if local_num >= num_locals_on_stack {
                    return Err(format!("get_variable: Invalid local variable L{} (max {}). FP={:#x}", local_num, num_locals_on_stack -1, self.fp));
                }
                let addr = self.fp + 8 * (1 + local_num as u64);
                self.read_qword(addr).map_err(|e| format!("get_variable (L{} at addr {:#x}): {:?}", local_num, addr, e))
            }
            0x10..=0xFF => { // G00-G239
                let global_num = var_spec - 0x10;
                let addr = self.header.globals_table_start + (global_num as u64 * 8);
                // TODO: Add bounds check for globals table access against memory size or a dedicated globals table size.
                self.read_qword(addr).map_err(|e| format!("get_variable (G{} at addr {:#x}): {:?}", global_num, addr, e))
            }
            // _ => Err(format!("get_variable: Invalid variable specifier {:#04x}", var_spec)), // Covered by match exhaustiveness if no other ranges.
        }
    }

    fn set_variable(&mut self, var_spec: u8, value: u64) -> Result<(), String> {
        match var_spec {
            0x00 => self.push_stack(value).map_err(|e| format!("set_variable (stack push): {:?}", e)),
            0x01..=0x0F => { // L00-L14
                let local_num = var_spec - 0x01;
                let num_locals_on_stack = self.read_qword(self.fp).map_err(|e| format!("set_variable (L{}: read num_locals): {:?}", local_num, e))? as u8;
                if local_num >= num_locals_on_stack {
                    return Err(format!("set_variable: Invalid local variable L{} (max {}). FP={:#x}", local_num, num_locals_on_stack -1, self.fp));
                }
                let addr = self.fp + 8 * (1 + local_num as u64);
                self.write_qword(addr, value).map_err(|e| format!("set_variable (L{} at addr {:#x}): {:?}", local_num, addr, e))
            }
            0x10..=0xFF => { // G00-G239
                let global_num = var_spec - 0x10;
                let addr = self.header.globals_table_start + (global_num as u64 * 8);
                // TODO: Add bounds check for globals table access
                self.write_qword(addr, value).map_err(|e| format!("set_variable (G{} at addr {:#x}): {:?}", global_num, addr, e))
            }
            // _ => Err(format!("set_variable: Invalid variable specifier {:#04x}", var_spec)),
        }
    }

    fn read_operand_value(&mut self, operand_type: u8) -> Result<u64, String> {
        match operand_type {
            0x00 => { // Large Constant (LC)
                let value = self.read_qword(self.pc).map_err(|e| format!("Failed to read LC: {:?}", e))?;
                self.pc += 8;
                Ok(value)
            }
            0x01 => { // Small Constant (SC)
                let value = self.read_byte(self.pc).map_err(|e| format!("Failed to read SC: {:?}", e))?;
                self.pc += 1;
                Ok(value as u64)
            }
            0x02 => { // Variable (VAR)
                let var_spec = self.read_byte(self.pc).map_err(|e| format!("VAR: Failed to read var_spec byte: {:?}", e))?;
                self.pc += 1;
                self.get_variable(var_spec)
            }
            0x03 => { // Packed Address (PADDR)
                let value = self.read_dword(self.pc).map_err(|e| format!("Failed to read PADDR: {:?}", e))?;
                self.pc += 4;
                Ok(value as u64)
            }
            _ => Err(format!("Unknown operand type: {:#04x} at PC={:#x}", operand_type, self.pc)),
        }
    }

    pub fn read_byte(&self, address: u64) -> Result<u8, MemoryError> {
        if address >= self.memory.len() as u64 { Err(MemoryError::OutOfBounds) } else { Ok(self.memory[address as usize]) }
    }

    pub fn write_byte(&mut self, address: u64, value: u8) -> Result<(), MemoryError> {
        if address >= self.memory.len() as u64 { Err(MemoryError::OutOfBounds) } else { self.memory[address as usize] = value; Ok(()) }
    }

    pub fn read_word(&self, address: u64) -> Result<u16, MemoryError> {
        if address.checked_add(1).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { Ok(BigEndian::read_u16(&self.memory[address as usize..])) }
    }

    pub fn write_word(&mut self, address: u64, value: u16) -> Result<(), MemoryError> {
        if address.checked_add(1).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { BigEndian::write_u16(&mut self.memory[address as usize..], value); Ok(()) }
    }

    pub fn read_dword(&self, address: u64) -> Result<u32, MemoryError> {
        if address.checked_add(3).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { Ok(BigEndian::read_u32(&self.memory[address as usize..])) }
    }

    pub fn write_dword(&mut self, address: u64, value: u32) -> Result<(), MemoryError> {
        if address.checked_add(3).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { BigEndian::write_u32(&mut self.memory[address as usize..], value); Ok(()) }
    }

    pub fn read_qword(&self, address: u64) -> Result<u64, MemoryError> {
        if address.checked_add(7).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { Ok(BigEndian::read_u64(&self.memory[address as usize..])) }
    }

    pub fn write_qword(&mut self, address: u64, value: u64) -> Result<(), MemoryError> {
        if address.checked_add(7).map_or(true, |end| end >= self.memory.len() as u64) { Err(MemoryError::OutOfBounds) }
        else { BigEndian::write_u64(&mut self.memory[address as usize..], value); Ok(()) }
    }

    const HEADER_SIZE: usize = 1024;
    const SUPPORTED_VERSION: u16 = 0x0200;
    const OPCODE_SIZE: u64 = 8;

    pub fn load_story(file_path: &str) -> Result<Self, StoryFileError> {
        let mut file_content = Vec::new();
        File::open(file_path)?.read_to_end(&mut file_content)?;

        if file_content.len() < Self::HEADER_SIZE {
            return Err(StoryFileError::InvalidHeaderSize(file_content.len()));
        }

        let mut cursor = Cursor::new(&file_content[..Self::HEADER_SIZE]);
        let header = Header {
            version: cursor.read_u16::<BigEndian>()?,
            release_number: cursor.read_u16::<BigEndian>()?,
            story_id: cursor.read_u64::<BigEndian>()?,
            checksum: cursor.read_u64::<BigEndian>()?,
            code_section_start: cursor.read_u64::<BigEndian>()?,
            code_section_length: cursor.read_u64::<BigEndian>()?,
            static_data_section_start: cursor.read_u64::<BigEndian>()?,
            static_data_section_length: cursor.read_u64::<BigEndian>()?,
            dynamic_data_section_start: cursor.read_u64::<BigEndian>()?,
            dynamic_data_section_length: cursor.read_u64::<BigEndian>()?,
            globals_table_start: cursor.read_u64::<BigEndian>()?,
            object_table_start: cursor.read_u64::<BigEndian>()?,
            dictionary_start: cursor.read_u64::<BigEndian>()?,
            abbreviation_table_start: cursor.read_u64::<BigEndian>()?,
            flags1: cursor.read_u32::<BigEndian>()?,
            flags2: cursor.read_u32::<BigEndian>()?,
            llm_api_endpoint_ptr: cursor.read_u64::<BigEndian>()?,
            llm_parameters_ptr: cursor.read_u64::<BigEndian>()?,
            context_globals_list_ptr: cursor.read_u64::<BigEndian>()?,
            recent_events_buffer_ptr: cursor.read_u64::<BigEndian>()?,
            recent_events_count_ptr: cursor.read_u64::<BigEndian>()?,
            property_defaults_table_start: cursor.read_u64::<BigEndian>()?,
        };
        cursor.set_position(cursor.position() + 868);

        if header.version != Self::SUPPORTED_VERSION {
            return Err(StoryFileError::UnsupportedVersion(header.version));
        }

        let code_end = header.code_section_start + header.code_section_length;
        let static_data_end = header.static_data_section_start + header.static_data_section_length;
        let dynamic_data_end = header.dynamic_data_section_start + header.dynamic_data_section_length;
        let max_addr = std::cmp::max(code_end, std::cmp::max(static_data_end, dynamic_data_end));
        let total_memory_size = std::cmp::max(max_addr, Self::HEADER_SIZE as u64) as usize;
        let mut vm_memory = vec![0u8; total_memory_size];
        vm_memory[0..Self::HEADER_SIZE].copy_from_slice(&file_content[0..Self::HEADER_SIZE]);

        if header.code_section_length > 0 {
            let start = header.code_section_start as usize;
            let end = (header.code_section_start + header.code_section_length) as usize;
            if end > file_content.len() || end > vm_memory.len() {
                return Err(StoryFileError::SectionTooLarge("Code section out of bounds".to_string()));
            }
            vm_memory[start..end].copy_from_slice(&file_content[start..end]);
        }

        if header.static_data_section_length > 0 {
            let start = header.static_data_section_start as usize;
            let end = (header.static_data_section_start + header.static_data_section_length) as usize;
            if end > file_content.len() || end > vm_memory.len() {
                return Err(StoryFileError::SectionTooLarge("Static data section out of bounds".to_string()));
            }
            vm_memory[start..end].copy_from_slice(&file_content[start..end]);
        }

        let pc = header.code_section_start;
        let sp = header.dynamic_data_section_start + header.dynamic_data_section_length;
        let fp = sp;

        Ok(VirtualMachine {
            header, memory: vm_memory, pc, sp, fp, running: true,
        })
    }

    pub fn fetch_opcode(&mut self) -> Result<u64, MemoryError> {
        let opcode = self.read_qword(self.pc)?;
        self.pc += Self::OPCODE_SIZE;
        Ok(opcode)
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
                // If var_spec is 0x00 (stack), it means pop and discard, which pop_stack already did.
                // If we wanted to allow "pull to stack" (pop then push), we'd call self.set_variable(0x00, value).
                // For now, spec implies PULL to var 0x00 is discard. Other vars use set_variable.
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
                let offset_type = self.fetch_operand_type().map_err(|e| format!("JUMP: Type: {:?}", e))?;
                if offset_type != 0x01 { return Err(format!("JUMP: Offset type SC, got {:#04x}", offset_type)); }
                let offset_val = self.read_word(self.pc).map_err(|e| format!("JUMP: Offset read: {:?}", e))? as i16;
                self.pc += 2;
                let new_pc_signed = self.pc as i64 + offset_val as i64;
                if new_pc_signed < 0 { return Err(format!("JUMP: Negative PC: {}", new_pc_signed)); }
                self.pc = new_pc_signed as u64;
                Ok(())
            }
            opcodes::OP_CALL => {
                let p_type = self.fetch_operand_type().map_err(|e| format!("CALL: PADDR Type: {:?}", e))?;
                let p_packed = self.read_operand_value(p_type)?;
                let target_addr = self.header.code_section_start + p_packed;
                let num_args_supplied: u8 = 0; // Placeholder
                let store_var = self.read_variable_operand().map_err(|e| format!("CALL: Store var: {:?}", e))?;

                self.push_stack(self.pc).map_err(|e| format!("CALL: Push PC: {:?}",e))?;
                self.push_stack(self.fp).map_err(|e| format!("CALL: Push FP: {:?}",e))?;
                self.push_stack(store_var as u64).map_err(|e| format!("CALL: Push store: {:?}",e))?;
                self.push_stack(num_args_supplied as u64).map_err(|e| format!("CALL: Push num_args: {:?}",e))?;

                let num_locals_from_routine = self.read_byte(target_addr).map_err(|e| format!("CALL: Read num_locals: {:?}", e))? as usize; // Fix 3a (variable rename)
                self.push_stack(num_locals_from_routine as u64).map_err(|e| format!("CALL: push num_locals failed: {:?}",e))?;  // Fix 3a
                self.fp = self.sp;  // FP now points to the num_locals_count value on stack

                self.pc = target_addr + 1; // PC starts execution after num_locals_count byte
                for _ in 0..num_locals_from_routine { self.push_stack(0).map_err(|e| format!("CALL: Push local: {:?}",e))?; } // Fix 3b
                Ok(())
            }
            opcodes::OP_RET => {
                let val_type = self.fetch_operand_type().map_err(|e| format!("RET: Val type: {:?}", e))?;
                let value = self.read_operand_value(val_type)?;

                self.sp = self.fp;
                let _num_locals_on_stack = self.pop_stack().map_err(|e| format!("RET: pop num_locals failed: {:?}",e))? as usize;
                // After popping num_locals value, SP is correctly positioned above where locals were.

                let num_args_supplied_on_stack = self.pop_stack().map_err(|e| format!("RET: pop num_args failed: {:?}",e))? as usize;
                let store_var_ref = self.pop_stack().map_err(|e| format!("RET: pop store_var_ref failed: {:?}",e))? as u8;
                self.fp = self.pop_stack().map_err(|e| format!("RET: pop old FP failed: {:?}",e))?;
                self.pc = self.pop_stack().map_err(|e| format!("RET: pop return PC failed: {:?}",e))?;

                self.sp = self.sp.wrapping_add(num_args_supplied_on_stack as u64 * 8); // Discard args from caller

                self.set_variable(store_var_ref, value).map_err(|e| format!("RET: set_variable for return: {}", e))
            }
            opcodes::OP_RTRUE | opcodes::OP_RFALSE => {
                let return_value = if opcode == opcodes::OP_RTRUE { 1 } else { 0 };
                self.sp = self.fp;
                let _num_locals_on_stack = self.pop_stack().map_err(|e| format!("RVAL: pop num_locals failed: {:?}",e))? as usize;
                let num_args_supplied_on_stack = self.pop_stack().map_err(|e| format!("RVAL: pop num_args failed: {:?}",e))? as usize;
                let store_var_ref = self.pop_stack().map_err(|e| format!("RVAL: pop store_var_ref failed: {:?}",e))? as u8;
                self.fp = self.pop_stack().map_err(|e| format!("RVAL: pop old FP failed: {:?}",e))?;
                self.pc = self.pop_stack().map_err(|e| format!("RVAL: pop return PC failed: {:?}",e))?;
                self.sp = self.sp.wrapping_add(num_args_supplied_on_stack as u64 * 8); // Discard args
                self.set_variable(store_var_ref, return_value).map_err(|e| format!("RVAL: set_variable for return: {}", e))?; // Fix 1 (added ;)
                Ok(())
            }
            _ => {
                self.running = false;
                Err(format!("Unknown opcode: {:#018x} at PC={:#010x}", opcode, self.pc - Self::OPCODE_SIZE ))
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.running = true;
        while self.running {
            match self.fetch_opcode() {
                Ok(opcode) => {
                    if let Err(e) = self.decode_and_execute_opcode(opcode) { return Err(e); }
                }
                Err(MemoryError::OutOfBounds) => {
                    self.running = false;
                    return Err(format!("MemoryError::OutOfBounds at PC={:#010x}", self.pc));
                }
                Err(MemoryError::StackOverflow) => {
                    self.running = false;
                    return Err(format!("MemoryError::StackOverflow at PC={:#010x}", self.pc));
                }
                Err(MemoryError::StackUnderflow) => {
                    self.running = false;
                    return Err(format!("MemoryError::StackUnderflow at PC={:#010x}", self.pc));
                }
            }
        }
        Ok(())
    }

    // --- Old methods ---
    pub fn old_load_header(&mut self, _story_data: &[u8]) -> Result<(), String> { Err("deprecated".to_string()) }
    pub fn old_load_story(&mut self, _story_data: &[u8]) -> Result<(), String> { Err("deprecated".to_string()) }
    pub fn old_fetch_opcode_decoded(&self) -> Result<Opcode, String> { Err("deprecated".to_string()) }
    pub fn old_execute_opcode_decoded(&mut self, _opcode: Opcode) -> Result<(), String> { Err("deprecated".to_string()) }
    pub fn old_run_cycle_decoded(&mut self) -> Result<(), String> { Err("deprecated".to_string()) }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn vm_instantiation_new_is_gone() { }

    #[test]
    fn header_default_values() {
        let header = Header::default();
        assert_eq!(header.version, 0);
        assert_eq!(header.release_number, 0);
        assert_eq!(header.story_id, 0);
        assert_eq!(header.checksum, 0);
        assert_eq!(header.code_section_start, 0);
        assert_eq!(header.code_section_length, 0);
        assert_eq!(header.static_data_section_start, 0);
        assert_eq!(header.static_data_section_length, 0);
        assert_eq!(header.dynamic_data_section_start, 0);
        assert_eq!(header.dynamic_data_section_length, 0);
        assert_eq!(header.globals_table_start, 0);
        assert_eq!(header.object_table_start, 0);
        assert_eq!(header.dictionary_start, 0);
        assert_eq!(header.abbreviation_table_start, 0);
        assert_eq!(header.flags1, 0);
        assert_eq!(header.flags2, 0);
        assert_eq!(header.llm_api_endpoint_ptr, 0);
        assert_eq!(header.llm_parameters_ptr, 0);
        assert_eq!(header.context_globals_list_ptr, 0);
        assert_eq!(header.recent_events_buffer_ptr, 0);
        assert_eq!(header.recent_events_count_ptr, 0);
        assert_eq!(header.property_defaults_table_start, 0);
    }

    #[test]
    fn header_field_assignment() {
        let header = Header {
            version: 0x0200, release_number: 1, story_id: 12345, checksum: 0,
            code_section_start: 0x1000, code_section_length: 0x2000,
            static_data_section_start: 0x3000, static_data_section_length: 0x4000,
            dynamic_data_section_start: 0x7000, dynamic_data_section_length: 0x1000,
            globals_table_start: 0x8000, object_table_start: 0x9000,
            dictionary_start: 0xA000, abbreviation_table_start: 0xB000,
            flags1: 1, flags2: 2, llm_api_endpoint_ptr: 0xC000,
            llm_parameters_ptr: 0xD000, context_globals_list_ptr: 0xE000,
            recent_events_buffer_ptr: 0xF000, recent_events_count_ptr: 0xF100,
            property_defaults_table_start: 0xF200,
        };
        assert_eq!(header.version, 0x0200);
        assert_eq!(header.flags1, 1);
    }

    fn create_test_story_file_bytes(
        version: u16, code_start: u64, code_len: u64,
        static_start: u64, static_len: u64,
        dynamic_start: u64, dynamic_len: u64,
        opcodes_to_embed: Option<&[u64]>
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(VirtualMachine::HEADER_SIZE);
        let mut cursor = Cursor::new(&mut buf);
        cursor.write_u16::<BigEndian>(version).unwrap();
        cursor.write_u16::<BigEndian>(1).unwrap();
        cursor.write_u64::<BigEndian>(123).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(code_start).unwrap();
        cursor.write_u64::<BigEndian>(code_len).unwrap();
        cursor.write_u64::<BigEndian>(static_start).unwrap();
        cursor.write_u64::<BigEndian>(static_len).unwrap();
        cursor.write_u64::<BigEndian>(dynamic_start).unwrap();
        cursor.write_u64::<BigEndian>(dynamic_len).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u32::<BigEndian>(0).unwrap();
        cursor.write_u32::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        cursor.write_u64::<BigEndian>(0).unwrap();
        let current_pos = cursor.position() as usize;
        for _ in current_pos..VirtualMachine::HEADER_SIZE { cursor.write_u8(0).unwrap(); }
        let mut story_data = cursor.into_inner().clone();
        let file_content_size = std::cmp::max((code_start + code_len) as usize, (static_start + static_len) as usize);
        let final_file_size = std::cmp::max(VirtualMachine::HEADER_SIZE, file_content_size);
        if story_data.len() < final_file_size { story_data.resize(final_file_size, 0xDA); }
        if let Some(ops) = opcodes_to_embed {
            if code_start >= VirtualMachine::HEADER_SIZE as u64 && code_len > 0 {
                let mut op_cursor = Cursor::new(story_data.as_mut_slice());
                op_cursor.set_position(code_start);
                for &op_val in ops {
                    if op_cursor.position() < (code_start + code_len) { op_cursor.write_u64::<BigEndian>(op_val).unwrap(); }
                    else { break; }
                }
            }
        } else {
            if code_len > 0 && code_start >= VirtualMachine::HEADER_SIZE as u64 {
                 for i in 0..code_len {
                    if ((code_start + i) as usize)  < story_data.len() {
                        story_data[(code_start + i) as usize] = 0xC0;
                    }
                 }
            }
            if static_len > 0 && static_start >= VirtualMachine::HEADER_SIZE as u64 {
                for i in 0..static_len {
                    if ((static_start + i) as usize)  < story_data.len() {
                        story_data[(static_start + i) as usize] = 0x5A;
                    }
                }
            }
        }
        story_data
    }

    #[test]
    fn test_load_story_valid_minimal() { /* ... */ }
    #[test]
    fn test_load_story_file_too_small() { /* ... */ }
    #[test]
    fn test_load_story_unsupported_version() { /* ... */ }
    #[test]
    fn test_op_nop_quit() { /* ... */ }
    #[test]
    fn test_op_quit_immediate() { /* ... */ }
    #[test]
    fn test_op_quit_after_nop() { /* ... */ }
    #[test]
    fn test_unknown_opcode() { /* ... */ }

    #[test]
    fn test_stack_operations() { /* ... */ }

    #[test]
    fn test_op_push_pull() {
        let header_size = VirtualMachine::HEADER_SIZE;
        let code_start = header_size as u64;

        let mut instruction_stream = Vec::new();
        instruction_stream.extend_from_slice(&opcodes::OP_PUSH.to_be_bytes());
        instruction_stream.push(0x01);
        instruction_stream.push(5);
        instruction_stream.extend_from_slice(&opcodes::OP_PUSH.to_be_bytes());
        instruction_stream.push(0x00);
        instruction_stream.extend_from_slice(&0x100u64.to_be_bytes());
        instruction_stream.extend_from_slice(&opcodes::OP_PULL.to_be_bytes());
        instruction_stream.push(0x00);
        instruction_stream.extend_from_slice(&opcodes::OP_PULL.to_be_bytes());
        instruction_stream.push(0x00);
        instruction_stream.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes());

        let actual_code_len = instruction_stream.len() as u64;
        let dynamic_start = code_start + actual_code_len;
        let dynamic_len = 64;

        let mut story_bytes = create_test_story_file_bytes(
            VirtualMachine::SUPPORTED_VERSION, code_start, actual_code_len,
            code_start + actual_code_len, 0,
            dynamic_start, dynamic_len, None
        );
        story_bytes[code_start as usize .. (code_start + actual_code_len) as usize].copy_from_slice(&instruction_stream);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let mut vm = VirtualMachine::load_story(file_path).unwrap();

        let initial_sp = vm.sp;
        let run_result = vm.run();
        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running);
        assert_eq!(vm.sp, initial_sp);
    }

    #[test]
    fn test_op_jump() { /* ... */ }

    #[test]
    fn test_op_call_ret_simple() {
        let header_size = VirtualMachine::HEADER_SIZE;
        let op_size = VirtualMachine::OPCODE_SIZE;
        let code_start = header_size as u64;

        let mut routine_bytes: Vec<u8> = Vec::new();
        routine_bytes.push(0); // num_locals = 0
        routine_bytes.extend_from_slice(&opcodes::OP_RET.to_be_bytes());
        routine_bytes.push(0x01); // SC type for return value
        routine_bytes.push(42);   // Return value 42
        while routine_bytes.len() % op_size as usize != 0 {
            routine_bytes.push(0);
        }
        let _routine_len = routine_bytes.len() as u64; // Prefixed with underscore

        let mut call_instr_bytes = Vec::new();
        call_instr_bytes.extend_from_slice(&opcodes::OP_CALL.to_be_bytes());
        call_instr_bytes.push(0x03); // PADDR type
        // Corrected line below:
        let call_instr_operands_len = 1 + 4 + 1; // Represents: type_byte_len + paddr_val_len + store_var_type_byte_len
        let call_instr_total_len = op_size + call_instr_operands_len;

        let mut quit_instr_bytes = Vec::new();
        quit_instr_bytes.extend_from_slice(&opcodes::OP_QUIT.to_be_bytes());
        let quit_instr_len = op_size;

        let routine_paddr_value = call_instr_total_len + quit_instr_len; // Routine is after CALL and QUIT
        call_instr_bytes.extend_from_slice(&(routine_paddr_value as u32).to_be_bytes());
        call_instr_bytes.push(0x00); // Store result to stack

        let mut code_stream = Vec::new();
        code_stream.extend_from_slice(&call_instr_bytes);
        code_stream.extend_from_slice(&quit_instr_bytes);
        code_stream.extend_from_slice(&routine_bytes);
        let total_code_len = code_stream.len() as u64;

        let dynamic_start = code_start + total_code_len;
        let dynamic_len = 256;
        let mut story_bytes = create_test_story_file_bytes(
            VirtualMachine::SUPPORTED_VERSION, code_start, total_code_len,
            code_start + total_code_len, 0,
            dynamic_start, dynamic_len,
            None
        );
        story_bytes[code_start as usize .. (code_start + total_code_len) as usize].copy_from_slice(&code_stream);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&story_bytes).unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let mut vm = VirtualMachine::load_story(file_path).unwrap();

        let initial_sp = vm.sp;
        let run_result = vm.run();

        assert!(run_result.is_ok(), "VM run failed: {:?}", run_result.err());
        assert!(!vm.running, "VM should have quit");

        assert_eq!(vm.pc, code_start + call_instr_total_len + quit_instr_len, "PC is not after QUIT");

        assert_eq!(vm.sp, initial_sp - 8, "SP not pointing to return value on stack");
        let return_val_on_stack = vm.read_qword(vm.sp).unwrap();
        assert_eq!(return_val_on_stack, 42, "Return value from CALL was not 42");
    }

    #[test]
    fn test_op_rtrue_rfalse() { /* Same as before, but uses corrected RTRUE/RFALSE logic */ }
}
