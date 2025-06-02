use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read}; // Added std::io::Read

#[derive(Debug, PartialEq)]
pub struct StoryHeader {
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
    pub globals_table_length: u64,
    pub abbreviations_table_start: u64,
    pub abbreviations_table_length: u64,
    pub objects_table_start: u64,
    pub objects_table_length: u64,
    pub properties_table_start: u64,
    pub properties_table_length: u64,
    pub attributes_table_start: u64,
    pub attributes_table_length: u64,
    pub string_table_start: u64,
    pub string_table_length: u64,
    pub dictionary_table_start: u64,
    pub dictionary_table_length: u64,
    pub system_functions_table_start: u64,
    pub system_functions_table_length: u64,
    pub opcodes_table_start: u64,
    pub opcodes_table_length: u64,
    pub flags: u64,
    pub reserved1: u64,
    pub reserved2: u64,
    pub reserved3: u64,
    // The following 32 fields are 8 bytes each, total 256 bytes
    pub reserved_block: [u64; 32], // 32 * 8 = 256 bytes
    // Padding to 1024 bytes.
    // Total size of fields above:
    // version (2) + release_number (2) = 4
    // story_id (8) + checksum (8) = 16
    // code_section_start (8) ... opcodes_table_length (8) = 24 fields * 8 bytes/field = 192
    // flags (8) + reserved1 (8) + reserved2 (8) + reserved3 (8) = 32
    // reserved_block (256)
    // Current total: 4 + 16 + 192 + 32 + 256 = 500 bytes.
    // Remaining padding: 1024 - 500 = 524 bytes.
    // This means 524 / 8 = 65.5 u64 fields, which is not right.
    // Let's re-check ZMACHINE_MARK_2_DESIGN_SPEC.md for the exact layout.

    // Section 2: Memory Model -> Header Structure
    // Version: 2 bytes
    // Release Number: 2 bytes
    // Story ID: 8 bytes
    // Checksum: 8 bytes
    // Code Section Start Address: 8 bytes
    // Code Section Length: 8 bytes
    // Static Data Section Start Address: 8 bytes
    // Static Data Section Length: 8 bytes
    // Dynamic Data Section Start Address: 8 bytes
    // Dynamic Data Section Length: 8 bytes
    // Globals Table Start Address: 8 bytes
    // Globals Table Length: 8 bytes
    // Abbreviations Table Start Address: 8 bytes
    // Abbreviations Table Length: 8 bytes
    // Objects Table Start Address: 8 bytes
    // Objects Table Length: 8 bytes
    // Properties Table Start Address: 8 bytes
    // Properties Table Length: 8 bytes
    // Attributes Table Start Address: 8 bytes
    // Attributes Table Length: 8 bytes
    // String Table Start Address: 8 bytes
    // String Table Length: 8 bytes
    // Dictionary Table Start Address: 8 bytes
    // Dictionary Table Length: 8 bytes
    // System Functions Table Start Address: 8 bytes
    // System Functions Table Length: 8 bytes
    // Opcodes Table Start Address: 8 bytes
    // Opcodes Table Length: 8 bytes
    // Flags: 8 bytes (e.g., transcripting, fixed-pitch font, screen refresh required)
    // Reserved 1: 8 bytes
    // Reserved 2: 8 bytes
    // Reserved 3: 8 bytes
    // Reserved Block (256 bytes): This is likely an array of 32 u64s.
    // Total so far: 2+2+8+8 + (24 * 8) + (4*8) + (32*8) = 4 + 16 + 192 + 32 + 256 = 500 bytes.

    // The spec says: "The header is 1024 bytes long."
    // "Any unused space within the header should be padded with zeros."
    // So, the remaining 1024 - 500 = 524 bytes are padding.
    // We can represent this as a byte array.
    pub padding: [u8; 524],
}

impl StoryHeader {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 1024 {
            return Err(format!(
                "Header data too short. Expected 1024 bytes, got {}",
                bytes.len()
            ));
        }

        let mut cursor = Cursor::new(bytes);

        let version = cursor.read_u16::<BigEndian>().unwrap();
        let release_number = cursor.read_u16::<BigEndian>().unwrap();
        let story_id = cursor.read_u64::<BigEndian>().unwrap();
        let checksum = cursor.read_u64::<BigEndian>().unwrap();
        let code_section_start = cursor.read_u64::<BigEndian>().unwrap();
        let code_section_length = cursor.read_u64::<BigEndian>().unwrap();
        let static_data_section_start = cursor.read_u64::<BigEndian>().unwrap();
        let static_data_section_length = cursor.read_u64::<BigEndian>().unwrap();
        let dynamic_data_section_start = cursor.read_u64::<BigEndian>().unwrap();
        let dynamic_data_section_length = cursor.read_u64::<BigEndian>().unwrap();
        let globals_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let globals_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let abbreviations_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let abbreviations_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let objects_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let objects_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let properties_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let properties_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let attributes_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let attributes_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let string_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let string_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let dictionary_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let dictionary_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let system_functions_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let system_functions_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let opcodes_table_start = cursor.read_u64::<BigEndian>().unwrap();
        let opcodes_table_length = cursor.read_u64::<BigEndian>().unwrap();
        let flags = cursor.read_u64::<BigEndian>().unwrap();
        let reserved1 = cursor.read_u64::<BigEndian>().unwrap();
        let reserved2 = cursor.read_u64::<BigEndian>().unwrap();
        let reserved3 = cursor.read_u64::<BigEndian>().unwrap();

        let mut reserved_block = [0u64; 32];
        for i in 0..32 {
            reserved_block[i] = cursor.read_u64::<BigEndian>().unwrap();
        }

        let mut padding = [0u8; 524];
        cursor.read_exact(&mut padding).unwrap();

        Ok(StoryHeader {
            version,
            release_number,
            story_id,
            checksum,
            code_section_start,
            code_section_length,
            static_data_section_start,
            static_data_section_length,
            dynamic_data_section_start,
            dynamic_data_section_length,
            globals_table_start,
            globals_table_length,
            abbreviations_table_start,
            abbreviations_table_length,
            objects_table_start,
            objects_table_length,
            properties_table_start,
            properties_table_length,
            attributes_table_start,
            attributes_table_length,
            string_table_start,
            string_table_length,
            dictionary_table_start,
            dictionary_table_length,
            system_functions_table_start,
            system_functions_table_length,
            opcodes_table_start,
            opcodes_table_length,
            flags,
            reserved1,
            reserved2,
            reserved3,
            reserved_block,
            padding,
        })
    }
}

#[cfg(test)]
pub(crate) fn create_dummy_header_bytes() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1024);
    // Version: 0x0200
    bytes.extend_from_slice(&[0x02, 0x00]);
        // Release Number: 0x0001
        bytes.extend_from_slice(&[0x00, 0x01]);
        // Story ID: 0x00...00AA
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xAA]);
        // Checksum: 0x12...F0
        bytes.extend_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
        // Code Section Start Address: 1024 (0x400)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00]);
        // Code Section Length: 256 (0x100)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]);
        // Static Data Section Start Address: 1024+256 = 1280 (0x500)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00]);
        // Static Data Section Length: 128 (0x80)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
        // Dynamic Data Section Start Address: 1280+128 = 1408 (0x580)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x80]);
        // Dynamic Data Section Length: 64 (0x40)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40]);
        // Globals Table Start Address: 1408+64 = 1472 (0x5C0)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xC0]);
        // Globals Table Length: 32 (0x20)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20]);
        // Abbreviations Table Start Address: 1472+32 = 1504 (0x5E0)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xE0]);
        // Abbreviations Table Length: 16 (0x10)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10]);
        // Objects Table Start Address: 1504+16 = 1520 (0x5F0)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xF0]);
        // Objects Table Length: 8 (0x08)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08]);
        // Properties Table Start Address: 1520+8 = 1528 (0x5F8)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xF8]);
        // Properties Table Length: 4 (0x04)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04]);
        // Attributes Table Start Address: 1528+4 = 1532 (0x5FC)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFC]);
        // Attributes Table Length: 2 (0x02)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02]);
        // String Table Start Address: 1532+2 = 1534 (0x5FE)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFE]);
        // String Table Length: 1 (0x01)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        // Dictionary Table Start Address: 1534+1 = 1535 (0x5FF)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFF]);
        // Dictionary Table Length: 1 (0x01) - Making it small
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        // System Functions Table Start Address: 1535+1 = 1536 (0x600)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00]);
        // System Functions Table Length: 1 (0x01)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        // Opcodes Table Start Address: 1536+1 = 1537 (0x601)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x01]);
        // Opcodes Table Length: 1 (0x01)
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        // Flags: 0
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // Reserved 1: 0
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // Reserved 2: 0
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // Reserved 3: 0
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Reserved Block (256 bytes of 0s)
        for _ in 0..32 {
            bytes.extend_from_slice(&[0u8; 8]);
        }

        // Padding (524 bytes of 0s)
        bytes.extend_from_slice(&[0u8; 524]);

        assert_eq!(bytes.len(), 1024);
        bytes
    }

#[cfg(test)]
mod tests {
    use super::*;
    // Now use the function from the parent module scope (but still cfg(test))
    use crate::header::create_dummy_header_bytes;


    #[test]
    fn test_story_header_from_bytes_valid() {
        let header_bytes = create_dummy_header_bytes();
        let header = StoryHeader::from_bytes(&header_bytes).unwrap();

        assert_eq!(header.version, 0x0200);
        assert_eq!(header.release_number, 0x0001);
        assert_eq!(header.story_id, 0xAA);
        assert_eq!(header.checksum, 0x123456789ABCDEF0);
        assert_eq!(header.code_section_start, 1024);
        assert_eq!(header.code_section_length, 256);
        assert_eq!(header.static_data_section_start, 1280);
        assert_eq!(header.static_data_section_length, 128);
        assert_eq!(header.dynamic_data_section_start, 1408);
        assert_eq!(header.dynamic_data_section_length, 64);
        assert_eq!(header.globals_table_start, 1472);
        assert_eq!(header.globals_table_length, 32);
        assert_eq!(header.abbreviations_table_start, 1504);
        assert_eq!(header.abbreviations_table_length, 16);
        assert_eq!(header.objects_table_start, 1520);
        assert_eq!(header.objects_table_length, 8);
        assert_eq!(header.properties_table_start, 1528);
        assert_eq!(header.properties_table_length, 4);
        assert_eq!(header.attributes_table_start, 1532);
        assert_eq!(header.attributes_table_length, 2);
        assert_eq!(header.string_table_start, 1534);
        assert_eq!(header.string_table_length, 1);
        assert_eq!(header.dictionary_table_start, 1535);
        assert_eq!(header.dictionary_table_length, 1);
        assert_eq!(header.system_functions_table_start, 1536);
        assert_eq!(header.system_functions_table_length, 1);
        assert_eq!(header.opcodes_table_start, 1537);
        assert_eq!(header.opcodes_table_length, 1);
        assert_eq!(header.flags, 0);
        assert_eq!(header.reserved1, 0);
        assert_eq!(header.reserved2, 0);
        assert_eq!(header.reserved3, 0);
        assert_eq!(header.reserved_block, [0u64; 32]);
        assert_eq!(header.padding, [0u8; 524]);
    }

    #[test]
    fn test_story_header_from_bytes_too_short() {
        let bytes = vec![0u8; 512]; // Less than 1024
        let result = StoryHeader::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Header data too short. Expected 1024 bytes, got 512"
        );
    }

    #[test]
    fn test_story_header_from_bytes_invalid_version_parsing() {
        // This test is more for the Memory struct's validation,
        // but we can ensure from_bytes parses whatever is there.
        let mut header_bytes = create_dummy_header_bytes();
        header_bytes[0] = 0x03; // Invalid version (e.g., 0x0300)
        header_bytes[1] = 0x00;
        let header = StoryHeader::from_bytes(&header_bytes).unwrap();
        assert_eq!(header.version, 0x0300);
    }
}
