// zm2_vm/src/opcodes.rs

#![allow(dead_code)] // Allow dead code for now as not all opcodes are used yet

// 0OP Opcodes
pub const OP_RTRUE: u64 = 0x0000;
pub const OP_RFALSE: u64 = 0x0001;
// OP_PRINT (0x0002) - Not implemented yet
// OP_PRINT_RET (0x0003) - Not implemented yet
// OP_SAVE (0x0004) - Not implemented yet
// OP_RESTORE (0x0005) - Not implemented yet
pub const OP_QUIT: u64 = 0x0006;
pub const OP_NOP: u64 = 0x0007;
// OP_RESTART (0x0008) - Not implemented yet
// OP_RET_POPPED (0x0009) - Not implemented yet
// OP_POP (0x000A) - Not implemented yet
// OP_CATCH (0x000B) - Not implemented yet
// OP_THROW (0x000C) - Not implemented yet

// 1OP Opcodes
pub const OP_RET: u64 = 0x010A;
pub const OP_JUMP: u64 = 0x010B;

// VAROP Opcodes
pub const OP_CALL: u64 = 0x0300;
pub const OP_PUSH: u64 = 0x0308;
pub const OP_PULL: u64 = 0x0309;
pub const OP_STORE: u64 = 0x0301; // ZM2 VAROP list, corrected from 0x0319

// 1OP Opcodes (continued)
pub const OP_LOAD: u64 = 0x010D;

// 2OP Opcodes
pub const OP_ADD: u64 = 0x0203;
pub const OP_SUB: u64 = 0x0204;


// TODO: Add other opcode constants as they are implemented
// Example:
// pub const OP_JE: u64 = 0x0100; // Example for a 2OP/VAR opcode
