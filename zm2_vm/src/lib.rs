//! # ZM2-VM
//!
//! A Z-Machine Mark 2 (Version 2) Virtual Machine implementation in Rust.
//! This crate provides the necessary structures and logic to load and execute
//! Z-Machine Version 2 story files.

use byteorder::{BigEndian, ByteOrder};

// --- Opcode Definitions ---

/// Represents Z-Machine opcodes.
/// Opcodes are fundamental instructions that the Z-Machine executes.
#[derive(Debug, PartialEq)]
pub enum Opcode {
    /// No operation. Does nothing. Represented by value `0x0007`.
    Nop,
    /// An unknown or unimplemented opcode encountered during execution.
    /// Contains the raw value of the unknown opcode.
    Unknown(u64),
}

/// The raw 64-bit value for the `Nop` opcode.
/// As per the Z-Machine Mark 2 Design Spec (example value).
pub const NOP_OPCODE_VALUE: u64 = 0x0000_0000_0000_0007;


// --- Core Data Structures ---

/// Represents the header of a Z-Machine story file.
/// The header contains metadata about the story, such as its version,
/// release number, and memory layout information.
/// Fields correspond to those defined in Section 2 of the Z-Machine Mark 2 Design Specification.
#[derive(Debug, Default, Clone, PartialEq)]
#[repr(C)] // Ensures C-like memory layout, though not strictly necessary for field-by-field parsing.
pub struct Header {
    /// Z-Machine version number (must be 2 for this VM). (Offset 0)
    pub version: u16,
    /// Release number of the story. (Offset 2)
    pub release_number: u16,
    /// Unique identifier for the story. (Offset 4)
    pub story_id: u64,
    /// Checksum of the story file. (Offset 12)
    pub checksum: u64,
    /// Start address of the code section in memory. (Offset 20)
    pub code_section_start: u64,
    /// Start address of the static data section in memory. (Offset 36)
    pub static_data_section_start: u64,
    /// Start address of the dynamic data section in memory. (Offset 52)
    pub dynamic_data_section_start: u64,
    // Other header fields as per spec (e.g., dictionary, object table, globals)
    // would be added here if needed for further implementation.
}

impl Header {
    /// The minimum required size of a byte slice to parse a `Header`. (1024 bytes)
    pub const MIN_SIZE: usize = 1024;

    // Internal constants for field offsets within the header byte slice.
    const VERSION_OFFSET: usize = 0;
    const RELEASE_NUMBER_OFFSET: usize = 2;
    const STORY_ID_OFFSET: usize = 4;
    const CHECKSUM_OFFSET: usize = 12;
    const CODE_SECTION_START_OFFSET: usize = 20;
    const STATIC_DATA_SECTION_START_OFFSET: usize = 36;
    const DYNAMIC_DATA_SECTION_START_OFFSET: usize = 52;

    /// Parses a `Header` from a byte slice.
    ///
    /// The slice must be at least `Header::MIN_SIZE` (1024) bytes long.
    /// Fields are read in Big Endian byte order from their specified offsets.
    ///
    /// # Arguments
    ///
    /// * `slice`: A byte slice representing the raw header data from a story file.
    ///
    /// # Returns
    ///
    /// * `Ok(Header)` if parsing is successful.
    /// * `Err(String)` if the slice is too short or another parsing error occurs.
    pub fn from_slice(slice: &[u8]) -> Result<Header, String> {
        if slice.len() < Self::MIN_SIZE {
            return Err(format!(
                "Header data too short. Expected at least {} bytes, got {}",
                Self::MIN_SIZE,
                slice.len()
            ));
        }

        Ok(Header {
            version: BigEndian::read_u16(&slice[Self::VERSION_OFFSET..]),
            release_number: BigEndian::read_u16(&slice[Self::RELEASE_NUMBER_OFFSET..]),
            story_id: BigEndian::read_u64(&slice[Self::STORY_ID_OFFSET..]),
            checksum: BigEndian::read_u64(&slice[Self::CHECKSUM_OFFSET..]),
            code_section_start: BigEndian::read_u64(&slice[Self::CODE_SECTION_START_OFFSET..]),
            static_data_section_start: BigEndian::read_u64(&slice[Self::STATIC_DATA_SECTION_START_OFFSET..]),
            dynamic_data_section_start: BigEndian::read_u64(&slice[Self::DYNAMIC_DATA_SECTION_START_OFFSET..]),
        })
    }
}

/// Represents the Z-Machine Virtual Machine.
/// It holds the machine's memory, program counter (PC), stack pointer (SP),
/// and the loaded story's header.
#[derive(Debug)]
pub struct VirtualMachine {
    /// Main memory of the Z-Machine, storing the story file and dynamic game state.
    pub memory: Vec<u8>,
    /// Program Counter: points to the current instruction in `memory`.
    pub pc: u64,
    /// Stack Pointer: points to the top of the stack in `memory`.
    /// (Note: ZM2 spec is less prescriptive on stack; this is a common VM feature).
    pub sp: u64,
    /// The parsed header of the loaded story file. `None` if no story is loaded.
    pub header: Option<Header>,
}

// --- Virtual Machine Implementation ---

impl Default for VirtualMachine {
    /// Creates a new `VirtualMachine` in its initial state.
    /// Memory is empty, PC and SP are 0, and no header is loaded.
    fn default() -> Self {
        VirtualMachine {
            memory: Vec::new(),
            pc: 0,
            sp: 0,
            header: None,
        }
    }
}

impl VirtualMachine {
    /// Default total memory size (1MB) if not otherwise determinable. Placeholder.
    const DEFAULT_TOTAL_MEMORY_SIZE: usize = 1024 * 1024;
    /// Size of a Z-Machine opcode in bytes (64-bit instructions).
    const OPCODE_SIZE: u64 = 8;

    /// Loads the header from the provided story data.
    /// This is the first step in loading a story.
    ///
    /// # Arguments
    ///
    /// * `story_data`: A byte slice containing the raw data of the Z-Machine story file.
    ///                 Only the first `Header::MIN_SIZE` bytes are read for the header.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the header is successfully parsed and loaded.
    /// * `Err(String)` if parsing fails (e.g., data too short).
    pub fn load_header(&mut self, story_data: &[u8]) -> Result<(), String> {
        let header_slice = if story_data.len() < Header::MIN_SIZE {
            story_data
        } else {
            &story_data[..Header::MIN_SIZE]
        };

        match Header::from_slice(header_slice) {
            Ok(loaded_header) => {
                self.header = Some(loaded_header);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Loads a Z-Machine story file into the VM.
    /// This involves parsing the header, allocating memory, copying the story data,
    /// and initializing the Program Counter (PC) and Stack Pointer (SP).
    ///
    /// # Arguments
    ///
    /// * `story_data`: A byte slice containing the raw data of the Z-Machine story file.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the story is successfully loaded.
    /// * `Err(String)` if loading fails (e.g., header parsing error, memory allocation issues).
    pub fn load_story(&mut self, story_data: &[u8]) -> Result<(), String> {
        // 1. Load header
        self.load_header(story_data)?;
        let header = self.header.as_ref().ok_or_else(|| "Header not loaded after successful call".to_string())?;

        // 2. Memory Allocation (Simplified for now)
        // TODO: Implement more robust memory size calculation based on header fields.
        let total_memory_size = Self::DEFAULT_TOTAL_MEMORY_SIZE;
        self.memory = vec![0; total_memory_size];

        // 3. Copy Story Data into allocated memory
        let bytes_to_copy = std::cmp::min(story_data.len(), self.memory.len());
        self.memory[0..bytes_to_copy].copy_from_slice(&story_data[0..bytes_to_copy]);

        // 4. Set Program Counter from header
        self.pc = header.code_section_start;

        // 5. Set Stack Pointer (Placeholder logic)
        // Use dynamic_data_section_start if valid, otherwise fallback to end of memory.
        if header.dynamic_data_section_start != 0 && (header.dynamic_data_section_start as usize) < total_memory_size {
             self.sp = header.dynamic_data_section_start;
        } else if total_memory_size > 0 {
             self.sp = total_memory_size as u64; // Fallback: end of memory
        } else {
            self.sp = 0; // Should not happen if total_memory_size is always >0
        }
        Ok(())
    }

    // --- Fetch-Decode-Execute Cycle ---

    /// Fetches the next opcode from memory at the current Program Counter (PC).
    ///
    /// # Returns
    ///
    /// * `Ok(Opcode)` containing the decoded opcode.
    /// * `Err(String)` if the PC is out of bounds or memory cannot be read.
    pub fn fetch_opcode(&self) -> Result<Opcode, String> {
        if self.pc as usize + (Self::OPCODE_SIZE as usize) > self.memory.len() {
            return Err(format!(
                "Program Counter (0x{:08x}) out of bounds (memory size 0x{:08x})",
                self.pc, self.memory.len()
            ));
        }

        // Read 8 bytes for the opcode
        let opcode_value = BigEndian::read_u64(&self.memory[self.pc as usize..]);

        // Decode the opcode value
        match opcode_value {
            NOP_OPCODE_VALUE => Ok(Opcode::Nop),
            unknown_val => Ok(Opcode::Unknown(unknown_val)),
        }
    }

    /// Executes a given opcode.
    /// Modifies the VM state according to the opcode's definition.
    ///
    /// # Arguments
    ///
    /// * `opcode`: The `Opcode` to execute.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if execution is successful or the opcode is a NOP.
    /// * `Err(String)` if the opcode is unknown or an error occurs during execution.
    pub fn execute_opcode(&mut self, opcode: Opcode) -> Result<(), String> {
        match opcode {
            Opcode::Nop => {
                // NOP effectively does nothing to the VM state here.
                Ok(())
            }
            Opcode::Unknown(val) => Err(format!("Unknown opcode encountered: {:#018x}", val)),
            // Other opcodes would be handled here.
        }
    }

    /// Runs a single fetch-decode-execute cycle of the Z-Machine.
    /// 1. Fetches an opcode from the current PC.
    /// 2. Advances the PC.
    /// 3. Executes the fetched opcode.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the cycle completes successfully.
    /// * `Err(String)` if any part of the cycle (fetch, decode, execute) fails.
    pub fn run_cycle(&mut self) -> Result<(), String> {
        // 1. Fetch
        let opcode = self.fetch_opcode()?;

        // 2. Advance PC (occurs regardless of execution success, if fetch succeeded)
        self.pc += Self::OPCODE_SIZE;

        // 3. Execute
        self.execute_opcode(opcode)?;

        Ok(())
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper Functions for Tests ---

    fn setup_vm_for_opcode_tests(pc: u64, opcode_val: u64) -> VirtualMachine {
        let mut vm = VirtualMachine::default();
        vm.memory = vec![0; 256]; // Small memory for these tests
        vm.pc = pc;
        if (pc as usize) + VirtualMachine::OPCODE_SIZE as usize <= vm.memory.len() {
             BigEndian::write_u64(&mut vm.memory[pc as usize..], opcode_val);
        }
        vm
    }

    fn create_test_header_bytes(
        version: u16, code_start: u64, static_start: u64, dynamic_start: u64
    ) -> Vec<u8> {
        let mut data = vec![0u8; Header::MIN_SIZE];
        BigEndian::write_u16(&mut data[Header::VERSION_OFFSET..], version);
        BigEndian::write_u16(&mut data[Header::RELEASE_NUMBER_OFFSET..], 1); // Dummy value
        BigEndian::write_u64(&mut data[Header::STORY_ID_OFFSET..], 12345); // Dummy value
        BigEndian::write_u64(&mut data[Header::CHECKSUM_OFFSET..], 0); // Dummy value
        BigEndian::write_u64(&mut data[Header::CODE_SECTION_START_OFFSET..], code_start);
        BigEndian::write_u64(&mut data[Header::STATIC_DATA_SECTION_START_OFFSET..], static_start);
        BigEndian::write_u64(&mut data[Header::DYNAMIC_DATA_SECTION_START_OFFSET..], dynamic_start);
        data
    }

    // --- Existing Tests (Header, VM Load) ---
    #[test]
    fn header_struct_layout() { assert!(std::mem::size_of::<Header>() >= 44); }
    #[test]
    fn test_header_from_slice_too_short() { let d = vec![0u8;1]; assert!(Header::from_slice(&d).is_err());}
    #[test]
    fn test_header_from_slice_valid() { let d = create_test_header_bytes(1,1024,2048,3072); assert!(Header::from_slice(&d).is_ok());}
    #[test]
    fn test_header_from_slice_exact_size() { let d = create_test_header_bytes(1,0,0,0); assert!(Header::from_slice(&d).is_ok());}
    #[test]
    fn vm_instantiation() { let _vm = VirtualMachine::default(); } // Checks Default trait
    #[test]
    fn vm_load_header_success() {let mut vm = VirtualMachine::default(); let d = create_test_header_bytes(0x205,2048,0,0); assert!(vm.load_header(&d).is_ok());}
    #[test]
    fn vm_load_header_success_exact_size() {let mut vm = VirtualMachine::default(); let d = create_test_header_bytes(0x206,0,0,0); assert!(vm.load_header(&d).is_ok());}
    #[test]
    fn vm_load_header_failure() {let mut vm = VirtualMachine::default(); let d = vec![0u8;1]; assert!(vm.load_header(&d).is_err());}
    #[test]
    fn test_vm_load_story_success() {
        let mut vm = VirtualMachine::default();
        let code_start_addr = Header::MIN_SIZE as u64;
        let static_start_addr = code_start_addr + 1000;
        let dynamic_start_addr = static_start_addr + 1000;
        let mut story_content = vec![0u8; Header::MIN_SIZE + 500]; // Story content beyond header
        let header_bytes = create_test_header_bytes(0x0200, code_start_addr, static_start_addr, dynamic_start_addr);
        story_content[0..Header::MIN_SIZE].copy_from_slice(&header_bytes);
        // Add some identifiable data after the header in story_content for verification
        story_content[Header::MIN_SIZE] = 0xAA;
        story_content[Header::MIN_SIZE + 1] = 0xBB;

        assert!(vm.load_story(&story_content).is_ok());
        assert_eq!(vm.pc, code_start_addr, "PC should be code_section_start");
        assert_eq!(&vm.memory[0..Header::MIN_SIZE], header_bytes.as_slice(), "Header part of story not copied correctly");
        assert_eq!(vm.memory[Header::MIN_SIZE], 0xAA); // Verify story data copied
        assert_eq!(vm.memory[Header::MIN_SIZE + 1], 0xBB);
    }
    #[test]
    fn test_vm_load_story_success_sp_fallback() {
        let mut vm = VirtualMachine::default();
        // Header with dynamic_data_section_start = 0, should trigger SP fallback
        let header_bytes = create_test_header_bytes(0x0200, 1024, 2048, 0);
        assert!(vm.load_story(&header_bytes).is_ok());
        assert_eq!(vm.sp, VirtualMachine::DEFAULT_TOTAL_MEMORY_SIZE as u64);
    }
    #[test]
    fn test_vm_load_story_header_failure() {let mut vm = VirtualMachine::default(); let d = vec![0u8;1]; assert!(vm.load_story(&d).is_err());}

    // --- Tests for Fetch-Decode-Execute ---

    #[test]
    fn test_fetch_opcode_nop() {
        let vm = setup_vm_for_opcode_tests(0, NOP_OPCODE_VALUE);
        match vm.fetch_opcode() {
            Ok(Opcode::Nop) => { /* Success */ }
            Ok(other) => panic!("Expected Opcode::Nop, got {:?}", other),
            Err(e) => panic!("Fetch failed: {}", e),
        }
    }

    #[test]
    fn test_fetch_opcode_unknown() {
        let unknown_val = 0x1234_5678_9ABC_DEF0;
        let vm = setup_vm_for_opcode_tests(0, unknown_val);
        match vm.fetch_opcode() {
            Ok(Opcode::Unknown(val)) => assert_eq!(val, unknown_val),
            Ok(other) => panic!("Expected Opcode::Unknown, got {:?}", other),
            Err(e) => panic!("Fetch failed: {}", e),
        }
    }

    #[test]
    fn test_fetch_opcode_out_of_bounds() {
        let mut vm = setup_vm_for_opcode_tests(0, NOP_OPCODE_VALUE);
        vm.pc = (vm.memory.len() - (VirtualMachine::OPCODE_SIZE as usize) + 1) as u64;
        match vm.fetch_opcode() {
            Ok(_) => panic!("Expected out of bounds error, but got an opcode."),
            Err(e) => {
                assert!(e.contains("out of bounds"), "Error message mismatch: {}", e);
            }
        }
    }

    #[test]
    fn test_fetch_opcode_at_exact_end_boundary() {
        let mut vm = VirtualMachine::default();
        vm.memory = vec![0; VirtualMachine::OPCODE_SIZE as usize];
        vm.pc = 0;
        BigEndian::write_u64(&mut vm.memory[0..], NOP_OPCODE_VALUE);

        assert!(vm.fetch_opcode().is_ok(), "Fetch should succeed at exact boundary.");
    }


    #[test]
    fn test_execute_opcode_nop() {
        let mut vm = VirtualMachine::default();
        let original_pc = vm.pc;
        let original_sp = vm.sp;
        assert!(vm.execute_opcode(Opcode::Nop).is_ok());
        assert_eq!(vm.pc, original_pc); // NOP shouldn't change PC by itself
        assert_eq!(vm.sp, original_sp);
    }

    #[test]
    fn test_execute_opcode_unknown() {
        let mut vm = VirtualMachine::default();
        let unknown_val = 0xBAD_C0DE_0FF_C0DE;
        assert!(vm.execute_opcode(Opcode::Unknown(unknown_val)).is_err());
    }

    #[test]
    fn test_run_cycle_nop() {
        let mut vm = setup_vm_for_opcode_tests(0, NOP_OPCODE_VALUE);
        let initial_pc = vm.pc;
        assert!(vm.run_cycle().is_ok());
        assert_eq!(vm.pc, initial_pc + VirtualMachine::OPCODE_SIZE);
    }

    #[test]
    fn test_run_cycle_unknown_opcode() {
        let unknown_val = 0xFFFF_FFFF_FFFF_FFFF;
        let mut vm = setup_vm_for_opcode_tests(0, unknown_val);
        let initial_pc = vm.pc;
        assert!(vm.run_cycle().is_err());
        assert_eq!(vm.pc, initial_pc + VirtualMachine::OPCODE_SIZE); // PC advances even if execute fails
    }

    #[test]
    fn test_run_cycle_fetch_out_of_bounds() {
        let mut vm = setup_vm_for_opcode_tests(0, NOP_OPCODE_VALUE);
        vm.pc = (vm.memory.len() - (VirtualMachine::OPCODE_SIZE as usize) + 1) as u64;
        let initial_pc = vm.pc;
        assert!(vm.run_cycle().is_err());
        assert_eq!(vm.pc, initial_pc); // PC should not advance if fetch fails
    }
}
