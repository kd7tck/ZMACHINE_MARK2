use crate::header::StoryHeader;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor; // Removed Write

pub const SUPPORTED_VERSION: u16 = 0x0200; // Z-Machine Model 2, Version 0 Made Public

#[derive(Debug)]
pub struct Memory {
    header: StoryHeader,
    data: Vec<u8>,
}

impl Memory {
    pub fn new(story_file_data: Vec<u8>) -> Result<Self, String> {
        if story_file_data.len() < 1024 {
            return Err(format!(
                "Story file data too short for header. Expected at least 1024 bytes, got {}",
                story_file_data.len()
            ));
        }

        let header = StoryHeader::from_bytes(&story_file_data[0..1024])?;

        if header.version != SUPPORTED_VERSION {
            return Err(format!(
                "Unsupported Z-machine version: 0x{:04X}. Expected 0x{:04X}",
                header.version, SUPPORTED_VERSION
            ));
        }

        // For now, the 'data' Vec<u8> will be a clone of the input story_file_data.
        // Later, we might adjust size based on header fields if story_file_data
        // could be larger than the actual required memory.
        let data = story_file_data;

        Ok(Memory { header, data })
    }

    pub fn read_byte(&self, address: u64) -> Result<u8, String> {
        let addr = address as usize;
        if addr >= self.data.len() {
            return Err(format!(
                "Read out of bounds: address 0x{:X} is beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        Ok(self.data[addr])
    }

    pub fn read_word(&self, address: u64) -> Result<u64, String> {
        let addr = address as usize;
        if addr + 7 >= self.data.len() { // A word is 8 bytes
            return Err(format!(
                "Read word out of bounds: address 0x{:X} (needs 8 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&self.data[addr..addr + 8]);
        match cursor.read_u64::<BigEndian>() {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("Failed to read word at 0x{:X}: {}", address, e)),
        }
    }

    pub fn write_byte(&mut self, address: u64, value: u8) -> Result<(), String> {
        let addr = address as usize;
        if addr >= self.data.len() {
            return Err(format!(
                "Write out of bounds: address 0x{:X} is beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        self.data[addr] = value;
        Ok(())
    }

    pub fn write_word(&mut self, address: u64, value: u64) -> Result<(), String> {
        let addr = address as usize;
        if addr + 7 >= self.data.len() { // A word is 8 bytes
            return Err(format!(
                "Write word out of bounds: address 0x{:X} (needs 8 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&mut self.data[addr..addr + 8]);
        match cursor.write_u64::<BigEndian>(value) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write word at 0x{:X}: {}", address, e)),
        }
    }

    // Getter for the header if needed for other parts of the VM
    pub fn header(&self) -> &StoryHeader {
        &self.header
    }

    // Additional memory access functions

    pub fn read_u16(&self, address: u64) -> Result<u16, String> {
        let addr = address as usize;
        if addr + 1 >= self.data.len() { // A u16 is 2 bytes
            return Err(format!(
                "Read u16 out of bounds: address 0x{:X} (needs 2 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&self.data[addr..addr + 2]);
        match cursor.read_u16::<BigEndian>() {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("Failed to read u16 at 0x{:X}: {}", address, e)),
        }
    }

    pub fn write_u16(&mut self, address: u64, value: u16) -> Result<(), String> {
        let addr = address as usize;
        if addr + 1 >= self.data.len() { // A u16 is 2 bytes
            return Err(format!(
                "Write u16 out of bounds: address 0x{:X} (needs 2 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&mut self.data[addr..addr + 2]);
        match cursor.write_u16::<BigEndian>(value) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write u16 at 0x{:X}: {}", address, e)),
        }
    }

    pub fn read_u32(&self, address: u64) -> Result<u32, String> {
        let addr = address as usize;
        if addr + 3 >= self.data.len() { // A u32 is 4 bytes
            return Err(format!(
                "Read u32 out of bounds: address 0x{:X} (needs 4 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&self.data[addr..addr + 4]);
        match cursor.read_u32::<BigEndian>() {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("Failed to read u32 at 0x{:X}: {}", address, e)),
        }
    }

    pub fn write_u32(&mut self, address: u64, value: u32) -> Result<(), String> {
        let addr = address as usize;
        if addr + 3 >= self.data.len() { // A u32 is 4 bytes
            return Err(format!(
                "Write u32 out of bounds: address 0x{:X} (needs 4 bytes) is near/beyond memory size 0x{:X}",
                address,
                self.data.len()
            ));
        }
        let mut cursor = Cursor::new(&mut self.data[addr..addr + 4]);
        match cursor.write_u32::<BigEndian>(value) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write u32 at 0x{:X}: {}", address, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::create_dummy_header_bytes; // Removed duplicate import

    // Helper to create minimal valid story data (header + a bit more)
    fn create_minimal_story_data(version: u16) -> Vec<u8> {
        let mut story_data = create_dummy_header_bytes();
        // Modify version in the dummy header bytes
        story_data[0] = (version >> 8) as u8;
        story_data[1] = (version & 0xFF) as u8;

        // Add some minimal section data based on dummy header pointers
        // Code section: 1024 to 1024+256=1280
        // Static data: 1280 to 1280+128=1408
        // Dynamic data: 1408 to 1408+64=1472
        // Total size needed: 1472 bytes.
        // Header is 1024. So, 1472 - 1024 = 448 bytes of additional data.
        // Current story_data.len() is 1024.
        // We need to ensure story_data is large enough for the operations we test.
        // For now, let's just make it slightly larger than header.
        // The dummy header defines sections up to around 1538.
        // Let's make the total data 2048 bytes.
        story_data.resize(2048, 0xCC); // Fill extra with a pattern
        story_data
    }

    #[test]
    fn test_memory_new_valid() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let memory = Memory::new(story_data.clone()).unwrap();
        assert_eq!(memory.header().version, SUPPORTED_VERSION);
        assert_eq!(memory.data.len(), story_data.len());
    }

    #[test]
    fn test_memory_new_insufficient_data() {
        let story_data = vec![0u8; 512]; // Less than 1024
        let result = Memory::new(story_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Story file data too short for header. Expected at least 1024 bytes, got 512"
        );
    }

    #[test]
    fn test_memory_new_invalid_version() {
        let story_data = create_minimal_story_data(0x0100); // Invalid version
        let result = Memory::new(story_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Unsupported Z-machine version: 0x0100. Expected 0x0200"
        );
    }

    #[test]
    fn test_read_byte_valid() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let memory = Memory::new(story_data).unwrap();
        // Header part: Story ID is at offset 4, 8 bytes long. Last byte is at 4+7=11 (0x0B).
        assert_eq!(memory.read_byte(0x0B).unwrap(), 0xAA); // Story ID last byte
        // Data part (filled with 0xCC after header)
        assert_eq!(memory.read_byte(1024).unwrap(), 0xCC);
    }

    #[test]
    fn test_read_byte_out_of_bounds() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let memory = Memory::new(story_data.clone()).unwrap();
        let len = story_data.len() as u64;
        let result = memory.read_byte(len); // Try to read at data.len()
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Read out of bounds: address 0x{:X} is beyond memory size 0x{:X}", len, len)
        );
    }

    #[test]
    fn test_write_byte_valid() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data).unwrap();
        memory.write_byte(1025, 0xFF).unwrap();
        assert_eq!(memory.read_byte(1025).unwrap(), 0xFF);
    }

    #[test]
    fn test_write_byte_out_of_bounds() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data.clone()).unwrap();
        let len = story_data.len() as u64;
        let result = memory.write_byte(len, 0xFF);
        assert!(result.is_err());
         assert_eq!(
            result.unwrap_err(),
            format!("Write out of bounds: address 0x{:X} is beyond memory size 0x{:X}", len, len)
        );
    }

    #[test]
    fn test_read_word_valid() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let memory = Memory::new(story_data).unwrap();
        // Story ID from header (starts at offset 4): 0x00000000000000AA
        assert_eq!(memory.read_word(0x04).unwrap(), 0xAA);
        // Checksum from header (starts at offset 12): 0x123456789ABCDEF0
        assert_eq!(memory.read_word(0x0C).unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn test_read_word_out_of_bounds() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let memory = Memory::new(story_data.clone()).unwrap();
        let len = story_data.len() as u64;
        // Exact boundary (start of word is okay, but word extends beyond)
        let result1 = memory.read_word(len - 7);
        assert!(result1.is_err());
         assert_eq!(
            result1.unwrap_err(),
            format!("Read word out of bounds: address 0x{:X} (needs 8 bytes) is near/beyond memory size 0x{:X}", len - 7, len)
        );
        // Clearly out of bounds
        let result2 = memory.read_word(len);
        assert!(result2.is_err());
    }

    #[test]
    fn test_write_word_valid() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data).unwrap();
        let test_addr = 1032; // Make sure it's 8-byte aligned for simplicity, though not required by logic
        let test_val = 0xAABBCCDDEEFF0011;
        memory.write_word(test_addr, test_val).unwrap();
        assert_eq!(memory.read_word(test_addr).unwrap(), test_val);

        // Verify individual bytes for endianness
        assert_eq!(memory.read_byte(test_addr + 0).unwrap(), 0xAA);
        assert_eq!(memory.read_byte(test_addr + 1).unwrap(), 0xBB);
        assert_eq!(memory.read_byte(test_addr + 2).unwrap(), 0xCC);
        assert_eq!(memory.read_byte(test_addr + 3).unwrap(), 0xDD);
        assert_eq!(memory.read_byte(test_addr + 4).unwrap(), 0xEE);
        assert_eq!(memory.read_byte(test_addr + 5).unwrap(), 0xFF);
        assert_eq!(memory.read_byte(test_addr + 6).unwrap(), 0x00);
        assert_eq!(memory.read_byte(test_addr + 7).unwrap(), 0x11);
    }

    #[test]
    fn test_write_word_out_of_bounds() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data.clone()).unwrap();
        let len = story_data.len() as u64;
        let test_val = 0x1122334455667788;

        // Exact boundary (start of word is okay, but word extends beyond)
        let result1 = memory.write_word(len - 7, test_val);
        assert!(result1.is_err());
        assert_eq!(
            result1.unwrap_err(),
            format!("Write word out of bounds: address 0x{:X} (needs 8 bytes) is near/beyond memory size 0x{:X}", len - 7, len)
        );
        // Clearly out of bounds
        let result2 = memory.write_word(len, test_val);
        assert!(result2.is_err());
    }

    // Test to ensure header parsing from memory.rs test context works
    // This requires create_dummy_header_bytes to be accessible.
    // If header.rs mod tests is not pub, we might need to duplicate or rethink access.
    // For now, assuming `crate::header::tests::create_dummy_header_bytes` works due to `pub mod tests` or similar.
    // If not, this test will fail to compile, and I'll adjust header.rs test module visibility.
    #[test]
    fn can_access_dummy_header_creator() {
        let _ = create_dummy_header_bytes(); // Check if it compiles
    }

    #[test]
    fn test_read_write_u16() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data).unwrap();
        let test_addr = 1024;
        let test_val = 0xABCD;
        memory.write_u16(test_addr, test_val).unwrap();
        assert_eq!(memory.read_u16(test_addr).unwrap(), test_val);
        assert_eq!(memory.read_byte(test_addr).unwrap(), 0xAB);
        assert_eq!(memory.read_byte(test_addr + 1).unwrap(), 0xCD);

        let len = memory.data.len() as u64;
        assert!(memory.write_u16(len - 1, test_val).is_err());
        assert!(memory.read_u16(len - 1).is_err());
    }

    #[test]
    fn test_read_write_u32() {
        let story_data = create_minimal_story_data(SUPPORTED_VERSION);
        let mut memory = Memory::new(story_data).unwrap();
        let test_addr = 1028; // Ensure enough space
        let test_val = 0x12345678;
        memory.write_u32(test_addr, test_val).unwrap();
        assert_eq!(memory.read_u32(test_addr).unwrap(), test_val);
        assert_eq!(memory.read_byte(test_addr).unwrap(), 0x12);
        assert_eq!(memory.read_byte(test_addr + 1).unwrap(), 0x34);
        assert_eq!(memory.read_byte(test_addr + 2).unwrap(), 0x56);
        assert_eq!(memory.read_byte(test_addr + 3).unwrap(), 0x78);

        let len = memory.data.len() as u64;
        assert!(memory.write_u32(len - 3, test_val).is_err());
        assert!(memory.read_u32(len - 3).is_err());
    }
}
