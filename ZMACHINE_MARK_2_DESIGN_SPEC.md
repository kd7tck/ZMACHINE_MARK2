# Z-Machine Mark 2 Design Specification

## 1. Overview

**Statement of Purpose:** The Z-Machine Mark 2 aims to revolutionize text-only interactive fiction by blending the classic Z-machine architecture with modern Large Language Model (LLM) capabilities. Its purpose is to create highly immersive, dynamic, and intuitively interactive experiences for players, moving beyond predefined commands and responses.

Z-Machine Mark 2 is a modernized version of the original Z-machine, designed for text-only interactive fiction games. It operates as a 64-bit virtual machine, allowing for large data handling and future scalability. It integrates with a Large Language Model (LLM) via Hugging Face's API for natural language understanding and generation, aiming to create a more immersive and interactive experience.

**Key Components and Interactions:**

The Z-Machine Mark 2 comprises the following core components:

1.  **64-bit Virtual Machine (VM):** The heart of the system, responsible for executing game logic, managing the game state, and handling memory operations. Its 64-bit architecture supports vast address spaces and data sizes.
2.  **Game Story File:** Contains the game's code, data (objects, text, etc.), and initial state, loaded by the VM.
3.  **Large Language Model (LLM):** An external AI model accessed via API (e.g., Hugging Face). It provides:
    *   **Natural Language Understanding (NLU):** Parses player's typed commands into structured actions that the VM can understand.
    *   **Natural Language Generation (NLG):** Creates rich, dynamic, and contextually relevant descriptive text for the game world and events.
4.  **Player Interface:** The means by which the player interacts with the game, typically a text input/output console.

**Interaction Flow:**

*   The **Player** inputs a command through the **Player Interface**.
*   The **VM** receives this input. Game logic, using LLM opcodes, can initiate an asynchronous request to the **LLM** for NLU, providing relevant game state information.
*   The game can periodically check the status of this request. Once completed, the **LLM**'s parsed structured action is retrieved by the **VM**.
*   The **VM** executes the action, updating the internal **Game State** (e.g., player location, inventory, world events).
*   For generating output, the **VM** may use LLM opcodes to initiate asynchronous requests to the **LLM** for NLG, providing prompts or context.
*   Once the LLM generates descriptive text and the VM retrieves it, this text is combined with the VM's own direct output and presented to the **Player** via the **Player Interface**.
This cycle, involving asynchronous LLM interactions, repeats, creating a continuous interactive loop.

## 2. Memory Model

-   **Word Size**: Each word is 64 bits (8 bytes).
-   **Address Space**: 64-bit addressing, providing 2<sup>64</sup> words of addressable memory.
-   **Memory Layout**:
    -   **Header**: Contains metadata (version, release number, memory map).
    -   **Code Section**: Stores game logic routines. Instructions can span multiple 64-bit words.
    -   **Data Sections**: Includes objects, text abbreviations, and dynamic memory.
-   **Byte-Addressable**: Remains byte-addressable, with 64-bit word granularity. Each 64-bit word consists of 8 bytes. Addresses refer to individual bytes, but operations like reads and writes are typically performed on whole words.

**Visual Representation of Memory Layout:**

```
+---------------------------------------+ 0x0000000000000000
| Header                                |
| (Fixed Size, e.g., 1024 Bytes)        |
+---------------------------------------+ Start of Code Section (from Header)
| Code Section                          |
| (Variable Size)                       |
| - Routines / Functions                |
| - Instructions (1 to N words each)    |
+---------------------------------------+ Start of Static Data Section (from Header)
| Static Data Section                   |
| - Object Table                        |
| - Property Tables                     |
| - Global Variables (Initial Values)   |
| - Text Abbreviation Table             |
| - Dictionary                          |
+---------------------------------------+ Start of Dynamic Data Section (from Header)
| Dynamic Data Section (Heap)           |
| - Game State Variables (Mutable)      |
| - Dynamic Object Data                 |
| - Buffers (e.g., for I/O, LLM comms)  |
| - Stack (if not CPU register based)   |
+---------------------------------------+ ... up to 2^64 - 1
| (Potentially Unused Address Space)    |
+---------------------------------------+
```

**Header Structure:**

The header is a fixed-size block at the beginning of the memory (e.g., the first 1024 bytes). All multi-byte values are stored in Big Endian format. Pointers within the header are absolute byte addresses.

| Offset (Bytes) | Size (Bytes) | Field Name                     | Description                                                                                                |
|----------------|--------------|--------------------------------|------------------------------------------------------------------------------------------------------------|
| 0              | 2            | `version`                      | Z-Machine Mark 2 version (e.g., 0x0200 for v2.0).                                                          |
| 2              | 2            | `release_number`               | Release number of the story file.                                                                          |
| 4              | 8            | `story_id`                     | Unique 64-bit identifier for the story.                                                                    |
| 12             | 8            | `checksum`                     | 64-bit checksum of the story file (excluding header beyond this field) for integrity verification.         |
| 20             | 8            | `code_section_start`           | Byte address of the start of the Code Section.                                                             |
| 28             | 8            | `code_section_length`          | Length of the Code Section in bytes.                                                                       |
| 36             | 8            | `static_data_section_start`    | Byte address of the start of the Static Data Section.                                                      |
| 44             | 8            | `static_data_section_length`   | Length of the Static Data Section in bytes.                                                                |
| 52             | 8            | `dynamic_data_section_start`   | Byte address of the start of the Dynamic Data Section (initial heap pointer).                              |
| 60             | 8            | `dynamic_data_section_length`  | Total available length for the Dynamic Data Section in bytes.                                              |
| 68             | 8            | `globals_table_start`          | Byte address of the global variables table within the Static Data Section.                                 |
| 76             | 8            | `object_table_start`           | Byte address of the object table within the Static Data Section.                                           |
| 84             | 8            | `dictionary_start`             | Byte address of the dictionary within the Static Data Section.                                             |
| 92             | 8            | `abbreviation_table_start`     | Byte address of the text abbreviation table within the Static Data Section.                                |
| 100            | 4            | `flags1`                       | Various flags. See details below:<br>\| Bit   \| Name                     \| Description (1 = ON/Required, 0 = OFF/Optional/Default) \|<br>\|-------\|--------------------------\|-----------------------------------------------------------\|<br>\| 0     \| `Transcripting`          \| VM attempts to record player input and game output.       \|<br>\| 1     \| `FixedPitchFont`         \| Game expects a fixed-pitch font for proper layout.        \|<br>\| 2     \| `StrictZSCIICompatMode`  \| VM attempts to transliterate Unicode to ZSCII. Default: Unicode. \|<br>\| 3     \| `DebugMode`              \| VM may provide verbose errors or debug features.          \|<br>\| 4     \| `LLMParseEnable`         \| `start_llm_parse` is active. Default: ON.                 \|<br>\| 5     \| `LLMGenerateEnable`      \| `start_llm_generate` is active. Default: ON.              \|<br>\| 6     \| `DivByZeroHalt`          \| Division by zero causes VM halt. Default: Halt. (Else returns 0) \|<br>\| 7     \| `SaveLoadEnable`         \| `save` and `restore` opcodes active. Default: ON.         \|<br>\| 8-31  \| `Reserved`               \| Must be 0.                                                \| |
| 104            | 4            | `flags2`                       | More flags. See details below:<br>\| Bit   \| Name                     \| Description (1 = ON, 0 = OFF/Default)                      \|<br>\|-------\|--------------------------\|------------------------------------------------------------\|<br>\| 0     \| `ForceLLMSync`           \| VM attempts to make LLM calls synchronous. Default: OFF.   \|<br>\| 1     \| `Enable拡張Opcodes`      \| Enables hypothetical extended, non-standard opcodes.       \|<br>\| 2     \| `BypassLLMModeration`    \| VM might bypass its own LLM content moderation. *Use with caution.* \|<br>\| 3-31  \| `Reserved`               \| Must be 0.                                                 \| |
| 108            | 8            | `llm_api_endpoint_ptr`         | Pointer to a null-terminated string within Static Data for the LLM API endpoint. 0 if not used.            |
| 116            | 8            | `llm_parameters_ptr`           | Pointer to a structure/string within Static Data for LLM parameters (e.g., model name). 0 if not used.<br>This pointer references a null-terminated JSON string within the Static Data Section. If this pointer is 0, the VM will use its own default LLM parameters.<br>The JSON string should contain an object with the following optional key-value pairs. If a key is omitted, the VM will use a sensible default for that parameter:<br><pre><code>{\n  "nlu_model_id": "string",     // Hugging Face model ID for Natural Language Understanding (parsing player input)\n  "nlg_model_id": "string",     // Hugging Face model ID for Natural Language Generation (descriptive text, NPC dialogue)\n  "default_nlu_temperature": "number", // 0.0 to 2.0, e.g., 0.5 (lower is more deterministic for parsing)\n  "default_nlg_temperature": "number", // 0.0 to 2.0, e.g., 0.9 (higher is more creative for generation)\n  "default_nlu_max_tokens": "integer", // Max tokens the NLU model should generate for the structured output, e.g., 100\n  "default_nlg_max_tokens": "integer", // Max tokens the NLG model should generate for descriptive text, e.g., 250\n  "api_base_url": "string"      // Optional: Override Hugging Face API base URL (e.g., for self-hosted inference endpoint)\n}</code></pre><br>**Example in Static Data:**<br><pre><code>// Somewhere in Static Data Section, pointed to by llm_parameters_ptr\nmy_llm_params:\n  DB "{",0\n  DB "  \"nlu_model_id\": \"mistralai/Mistral-7B-Instruct-v0.1\",",0\n  DB "  \"nlg_model_id\": \"gpt2\",",0\n  DB "  \"default_nlg_temperature\": 0.85,",0\n  DB "  \"default_nlg_max_tokens\": 300",0\n  DB "}",0\n  DB 0 // Null terminator for the string</code></pre><br>The VM reads this JSON string at startup. If the JSON is malformed or a parameter value is invalid, the VM should report an error or fall back to its internal defaults for the problematic parameter(s). |
| 124            | 8            | `context_globals_list_ptr`     | Pointer to a null-terminated list of 1-byte global variable indices (0-239) for `get_context_as_json`. 0 if not used. |
| 132            | 8            | `recent_events_buffer_ptr`     | Pointer to the start of a buffer for recent event strings (for `get_context_as_json`). 0 if not used.         |
| 140            | 8            | `recent_events_count_ptr`      | Pointer to a 1-byte variable holding the count of valid event strings in `recent_events_buffer_ptr`. 0 if not used. |
| 148            | 876          | `reserved`                     | Reserved for future expansion. Must be initialized to zero.                                                |
| **Total Size** | **1024**     |                                |                                                                                                            |

**Code and Data Section Organization and Access:**

*   **Code Section:**
    *   **Organization**: This section contains the executable game logic. It is primarily a sequence of routines (functions). Each routine consists of a series of instructions. Instructions themselves can vary in length, potentially spanning multiple 64-bit words, especially if they include immediate data or multiple operands. The first part of a routine might define the number of local variables.
    *   **Access**: The Program Counter (PC) register points to the current instruction being executed within this section. `CALL` opcodes store a return address and jump to a routine's entry point. `JUMP` opcodes modify the PC directly. Addresses for routines are typically obtained from the Header or other game data structures. Data embedded directly within instruction streams (immediate operands) is accessed relative to the PC.

*   **Static Data Section:**
    *   **Organization**: This section holds data that is generally fixed at compile time and does not change during gameplay (though some parts like global variables may be initialized here and then modified in dynamic memory or a save game area). It includes:
        *   **Object Table**: Defines all game objects, their attributes, parent-sibling-child relationships, and pointers to their property tables.
        *   **Property Tables**: Store properties for each object.
        *   **Global Variables**: Initial values for global game variables.
        *   **Dictionary**: A list of words recognized by the parser (less critical if LLM is primary parser, but useful for fallback or specific commands). Words are typically encoded.
        *   **Text Abbreviation Table**: Pointers to frequently used strings to save space (Z-encoded).
        *   Other static data like arrays, game configuration settings, etc.
    *   **Access**: Data in this section is accessed using absolute addresses derived from the header pointers (e.g., `object_table_start`) plus offsets, or via pointers stored in variables or object properties.

*   **Dynamic Data Section (Heap):**
    *   **Organization**: This is the writable area of memory used for data that changes during gameplay. It functions as a heap, where memory can be allocated and deallocated (though deallocation might be implicit or garbage collection based, TBD). It stores:
        *   **Mutable Game State Variables**: Current values of global variables (if copied here from static data), and local variables for active routines (if not on a separate stack).
        *   **Dynamic Object Data**: Changes to object properties, or dynamically created objects/data.
        *   **Buffers**: Temporary storage for player input, text being prepared for output, data being sent to/from the LLM.
        *   **Game Stack**: If a dedicated hardware stack is not part of the VM CPU design, the call stack (for routine calls, local variables, and temporary calculations) would reside here.
    *   **Access**: Accessed via absolute addressing. A heap pointer (managed by the VM) tracks the current top of allocated dynamic memory. Variables often act as pointers into this section. Stack operations (push/pop) would manipulate a stack pointer within this area.

## 3. Story File Format

A Z-Machine Mark 2 story file (typically with a `.zm2` or `.z64` extension) is a binary file composed of several contiguous sections. The structure and content of these sections are based on the memory model loaded by the VM. All multi-byte numerical data within the story file is stored in **Big Endian** format.

**File Structure:**

1.  **Header Section (Fixed Size, e.g., 1024 bytes)**:
    *   **Content**: Identical to the **Header Structure** defined in Section 2 (Memory Model). Contains metadata such as version, story ID, checksum, and pointers (absolute byte offsets from the start of the file) to the start and length of subsequent sections (`code_section_start`, `static_data_section_start`, etc.).
    *   **Encoding**: As per the Header Structure table in Section 2.

2.  **Code Section (Variable Size)**:
    *   **Content**: Contains all executable game logic, encoded as Z-Machine Mark 2 instructions (opcodes and operands). Routines are sequences of these instructions.
    *   **Encoding**: Raw binary sequence of 64-bit opcodes and their operands. Packed addresses (`PADDR`) within instructions are relative to specific bases (e.g., start of Code Section or Static Data Section) and are expanded by the VM at runtime. Details of instruction encoding are in Section 4 (Instruction Set).
    *   **Location in File**: Starts at the byte offset specified by `code_section_start` in the Header and continues for `code_section_length` bytes.

3.  **Static Data Section (Variable Size)**:
    *   **Content**: Contains game data that is generally fixed at compile time. This includes:
        *   **Object Table**: Defines all game objects. The table starts at the address specified by `object_table_start` in the Header. It's an array of object entries. The total number of objects can be inferred if there's a standard object 0 or a specific field for object count, or by dividing the relevant section of static data by object entry size. Object IDs are 1-based indices into this table (Object 0 is often unused or has special meaning). Each object entry has a fixed size of 40 bytes and is structured as follows (all multi-byte values are Big Endian):
            ```
            | Offset | Size (Bytes) | Field Name             | Description                                                                 |
            |--------|--------------|------------------------|-----------------------------------------------------------------------------|
            | 0      | 8            | `attributes`           | 64-bit bitfield for object attributes (e.g., portable, wearable, lit).      |
            | 8      | 8            | `parent_obj_id`        | 64-bit ID of the parent object. 0 if no parent (e.g., for a room object).   |
            | 16     | 8            | `sibling_obj_id`       | 64-bit ID of the next sibling object. 0 if no more siblings.                |
            | 24     | 8            | `child_obj_id`         | 64-bit ID of the first child object. 0 if no children.                      |
            | 32     | 8            | `property_table_ptr`   | Absolute byte address of this object's property table. 0 if no properties.  |
            ```
            Object IDs are 64-bit values. For ZM2, an object ID `N` typically refers to the Nth entry in the object table (1-indexed). So, to find object `N`, the address would be `object_table_start + ((N-1) * 40)`.
        *   **Property Tables**: Each object can have an associated property table, pointed to by its `property_table_ptr`. A property table contains a list of properties for that object. The format of a property table begins with a 64-bit text address for the object's 'short name' (a Z-encoded string). This is followed by a sequence of individual property entries. The end of the property list is typically indicated by a property block starting with a property ID of 0.
            Each individual property entry is structured as follows:
            ```
            | Size (Bytes)         | Field Name      | Description                                                                      |
            |----------------------|-----------------|----------------------------------------------------------------------------------|
            | 1 (or 2, see below)  | `id_and_length` | Combined property ID and length of property data. See details below.             |
            | Variable (`L`)       | `data`          | Property data itself (L bytes).                                                  |
            ```
            **Simplified and ZM2-Specific `id_and_length` Encoding:**
            *   Each property entry starts with one or two bytes defining its ID and length.
            *   Byte 1:
                *   Bits 0-5: Property ID (1-63). An ID of 0 marks the end of the property list.
                *   Bit 6: Reserved (0).
                *   Bit 7: Length specifier.
                    *   If 0: Length of data is 1 byte. Property data (1 byte) follows. Total: 2 bytes (byte1 + data).
                    *   If 1: A second byte follows Byte 1.
            *   Byte 2 (only if Bit 7 of Byte 1 is 1):
                *   Bits 0-7: Length of data `L` (1-255 bytes). Property data (`L` bytes) follows. Total: 2 + `L` bytes.
                *   (Note: A length of 0 in Byte 2 is invalid. Lengths from this byte are 1-255).
            *   Property data is always byte-aligned. Property values are interpreted according to game logic (e.g., as numbers, pointers, or Z-encoded strings).
            *   Default properties are not explicitly handled by this format; all properties for an object must be listed if this structure is used.
        *   **Global Variables Initial Values Table**: A table holding the initial 64-bit values for all 240 global variables. These are copied to the dynamic memory area for global variables upon game initialization.
        *   **Dictionary**: A list of ZSCII-encoded words recognized by the traditional parser (if used as a fallback). Each word is typically fixed-length, padded with nulls. Associated with data like word ID or pointers to grammar tokens.
        *   **Text Abbreviation Table**: A table of pointers to frequently used Z-encoded strings. Used by `print_abbrev` opcode.
        *   **Z-Encoded Strings**: All game text (room descriptions, messages, object names, etc.) is stored as ZSCII strings, potentially using Z-Machine string compression techniques (e.g., Huffman-like encoding of character pairs, references to abbreviation table).
        *   **LLM API Configuration Strings**: Null-terminated strings for `llm_api_endpoint_ptr` and `llm_parameters_ptr` if these are used.
        *   Other static arrays, tables, or data structures defined by the game authoring system.
    *   **Encoding**: Binary data. Object IDs are 64-bit. Pointers within this section (e.g., from object to its properties) are absolute byte offsets from the start of the Static Data Section or the start of the file.
    *   **Location in File**: Starts at `static_data_section_start` and continues for `static_data_section_length` bytes.

4.  **Dynamic Data Section (Initial State - Optional in File)**:
    *   **Content**: The story file *may* optionally include an initial snapshot of the Dynamic Data Section. However, more commonly, the Dynamic Data section in memory is initialized by the VM based on data from the Static Data section (e.g., copying global variable initial values, setting up initial object states).
    *   If included, it would represent the desired starting state of mutable game elements if it differs from what can be constructed from the Static Data section alone.
    *   **Encoding**: Raw binary image of the initial dynamic memory.
    *   **Location in File**: If present, its start and length would also be defined in the Header, potentially as `initial_dynamic_data_start` and `initial_dynamic_data_length`. Most often, the `dynamic_data_section_start` in the header refers to where this section will *reside in memory*, and its initial content is constructed by the VM, not bulk-loaded from the file. The `dynamic_data_section_length` in the header defines the *size* of this memory region to be allocated by the VM.

**Encoding of Game Data:**

*   **Scenes/Locations**: Represented as **Objects** in the Object Table. A location object might have properties for its description (a pointer to a Z-encoded string or a prompt for an LLM generation opcode), exits (pointers to other location objects or routines to handle movement), and lists of other objects currently present in it.
*   **Objects (Items, Scenery, etc.)**: Defined in the Object Table. Each object has:
    *   A unique 64-bit ID.
    *   Attributes (e.g., `portable`, `wearable`, `lit`, `container`).
    *   Parent, sibling, and child object ID pointers to define the object tree (e.g., what's in a room, what's in a container).
    *   A pointer to its property table. Properties might include description text, weight, value, or game-specific data.
*   **Characters (NPCs)**: Also represented as **Objects**. NPC-specific logic is handled by routines in the Code Section. Dialogue might be stored as Z-encoded strings, selected by game logic, or generated via LLM generation opcodes using prompts stored as strings in the Static Data section. NPC state (e.g., mood, knowledge) is stored in its object properties or related global variables.
*   **Game Logic (Puzzles, Rules, Event Handling)**: Encoded as routines (functions) in the **Code Section**. These routines manipulate game state (variables, object properties/locations) and interact with the player via I/O opcodes or the asynchronous LLM interaction model.
    *   Example: A puzzle might involve checking if the player has a specific object (`get_parent`) and is in a specific room (global variable for player location), then changing a property on another object (`put_prop`) to signify the puzzle is solved.
*   **Text**: All printable text is stored as Z-encoded strings (see Z-Machine Standard 1.1, Appendix C for ZSCII and string encoding details, adapted for potential Unicode characters via `print_unicode`). Abbreviations are used to save space. This adaptation means that standard Z-Machine text processing opcodes (e.g., `print`, `print_addr`) primarily handle ZSCII characters and Z-encoded abbreviations. For characters outside the ZSCII set, the `print_unicode (char_code)` opcode must be used. Story file text intended for direct printing via standard opcodes should primarily use ZSCII and Z-encoding. Text requiring broader Unicode support will typically be constructed or processed at runtime and printed character by character using `print_unicode` if necessary. Text generated by LLMs, which is typically UTF-8, will be processed by the VM before being made available to Z-code. This involves converting it into a sequence of ZSCII characters (for direct use with `print` opcodes if possible) and/or Unicode character codes (for use with `print_unicode`). See Section 7 (Player Interaction), subsection "Output Formatting and Presentation" for details on runtime handling.

The story file is essentially a serialized form of the static parts of the Z-Machine's memory, plus the executable code. The VM reads this file to populate its memory and then begins execution. The Dynamic Data Section in memory is where the live game state evolves from the initial state defined in the Static Data.

## 4. Instruction Set

-   **Command Size**: Each opcode is 64 bits long.
-   **Opcode Categories**:
    -   **Standard Opcodes**: Adapted from the original Z-machine for 64-bit operands (e.g., arithmetic, control flow, object manipulation, input/output). These opcodes will generally follow the naming and operational conventions of the Z-Machine Standard 1.1, but expanded to handle 64-bit addresses and data. Operands can be constants (small or large), or variables (local, global, or stack).
    -   **Extended Opcodes for LLM Integration (Asynchronous)**: These opcodes facilitate non-blocking interaction with an external LLM. The general flow involves starting an LLM task, periodically checking its status, and then retrieving the result once ready.
        -   `start_llm_parse`: Initiates an LLM parsing task.
        -   `start_llm_generate`: Initiates an LLM text generation task.
        -   `check_llm_status`: Polls the status of a pending LLM request.
        -   `get_llm_result`: Retrieves the result of a successful LLM operation.
-   **Version Support**: Designed for backward compatibility with earlier Z-machine concepts, extended for 64-bit operations. Opcodes will be version-flagged if behavior changes significantly from Z-Spec 1.1.

**Operand Types:**

Operands for opcodes are fetched according to type specifiers. Common types include:
*   **Large Constant (LC):** A 64-bit constant value embedded in the instruction stream.
*   **Small Constant (SC):** An 8-bit or 16-bit constant value embedded in the instruction stream (implementation detail, expanded to 64-bit internally).
*   **Variable (VAR):** A 64-bit value from a variable. The variable is specified by an 8-bit number:
    *   `0x00`: Top of stack (popped).
    *   `0x01-0x0F`: Local variable (L00-L15).
    *   `0x10-0xFF`: Global variable (G00-G239).
*   **Address (ADDR):** A 64-bit byte address. Can be a Large Constant or a Variable.
*   **Packed Address (PADDR):** A compressed address format for routines or strings within the code/static data sections, which is then expanded to a full 64-bit byte address by the VM.

**Standard Opcodes (Illustrative List - Not Exhaustive):**

This list provides examples. A full list will be maintained in a separate ZM2_Opcodes.md document. All operations are on 64-bit values unless specified.

*   **0OP (No Operands):**
    *   `rtrue`: Return true (1) from the current routine.
    *   `rfalse`: Return false (0) from the current routine.
    *   `print`: Prints the Z-encoded string at the current PC, then advances PC.
    *   `newline`: Prints a newline character.
    *   `quit`: Terminates the game.
    *   `save`: (Branch) Saves the game state. Branches if successful.
    *   `restore`: (Branch) Restores the game state. Branches if successful. (Note: restore does not return, unlike ZSpec 1.1)
    *   `nop`: No operation.

*   **1OP (One Operand):**
    *   `jz (value)`: Jump if value is zero. Operand is the value to test. Branch data follows.
    *   `get_sibling (object_id) -> (result)`: Stores sibling object ID.
    *   `get_child (object_id) -> (result)`: Stores child object ID.
    *   `get_parent (object_id) -> (result)`: Stores parent object ID.
    *   `get_prop_len (property_address) -> (result)`: Stores length of property data.
    *   `inc (variable_ref)`: Increments variable.
    *   `dec (variable_ref)`: Decrements variable.
    *   `print_addr (byte_address)`: Prints Z-encoded string at the given byte address.
    *   `print_obj (object_id)`: Prints short name of the object.
    *   `ret (value)`: Return value from current routine.
    *   `pop (variable_ref)`: Pops value from stack into variable (if variable_ref is 0, effectively discards top of stack).

*   **2OP (Two Operands):**
    *   `je (value1, value2)`: Jump if value1 equals value2. Branch data follows.
    *   `jl (value1, value2)`: Jump if value1 < value2. Branch data follows.
    *   `jg (value1, value2)`: Jump if value1 > value2. Branch data follows.
    *   `add (a, b) -> (result)`: a + b.
    *   `sub (a, b) -> (result)`: a - b.
    *   `mul (a, b) -> (result)`: a * b.
    *   `div (a, b) -> (result)`: a / b (integer division).
    *   `mod (a, b) -> (result)`: a % b.
    *   `loadw (array_addr, word_index) -> (result)`: Reads word from array. (array_addr is byte address).
    *   `storew (array_addr, word_index, value)`: Writes word to array.
    *   `loadb (array_addr, byte_index) -> (result)`: Reads byte from array (zero-extended to 64-bit).
    *   `storeb (array_addr, byte_index, value)`: Writes byte to array (lowest 8 bits of value).
    *   `get_prop (object_id, property_id) -> (result)`: Reads property value.
    *   `put_prop (object_id, property_id, value)`: Writes property value.
    *   `test_attr (object_id, attribute_id)`: (Branch) Tests if attribute is set.
    *   `set_attr (object_id, attribute_id)`: Sets an attribute.
    *   `clear_attr (object_id, attribute_id)`: Clears an attribute.
    *   `insert_obj (object_id, destination_id)`: Moves object into destination.
    *   `remove_obj (object_id)`: Removes object from its parent.

*   **VAROP (Variable Number of Operands):**
    *   `call (routine_paddr, arg1, ..., argN) -> (result)`: Calls a routine. `routine_paddr` is a packed address.
    *   `store (variable_ref, value)`: Stores value in variable.
    *   `print_unicode (char_code)`: Prints a Unicode character corresponding to the given code point. (Can be VAROP for multiple chars or a sequence).
    *   `check_unicode (char_code) -> (result)`: Checks if current output stream supports the Unicode character. (Returns 1 if supported, 0 if not, 2 if maybe/transliteration possible).
    *   `sread (text_buffer_addr, parse_buffer_addr)`: Reads player input from the console into `text_buffer_addr`. If `parse_buffer_addr` is non-zero, it may also perform a traditional dictionary-based tokenization similar to the original Z-Machine specification, storing results in `parse_buffer_addr`. This opcode **does not** directly interact with the LLM. Game logic is responsible for subsequently calling LLM parsing opcodes (e.g., `start_llm_parse`) with the content of `text_buffer_addr` if LLM-based parsing is desired.
    *   `aread (text_buffer_addr, parse_buffer_addr, timeout_routine_paddr, timeout_seconds) -> (result)`: Async read with timeout. (Similar to `sread` regarding LLM non-interaction).

**VM Utility Opcodes:**

1.  **`get_context_as_json (object_scope_flag, max_depth, output_buffer_addr, max_output_len) -> (status_code)`**
    *   **Functionality**: Collects relevant game state information and formats it as a JSON string in the specified buffer. This is intended to help populate the `context_data_addr` for `start_llm_parse` or `start_llm_generate` calls.
    *   **Operands**:
        *   `object_scope_flag (SC/VAR)`: A bitmask defining the scope of objects to include.
            *   Bit 0 (0x01): Include player inventory.
            *   Bit 1 (0x02): Include objects in current location.
            *   Bit 2 (0x04): Include visible objects in adjacent locations (1 level away).
            *   Bit 3 (0x08): Include a subset of global game variables. This subset is defined by a null-terminated list of 1-byte global variable indices (0-239) stored at an address pointed to by a new Header field: `context_globals_list_ptr` (type: ADDR, 8 bytes). If `context_globals_list_ptr` is 0, no globals are included by this flag. The JSON output will use 'gX' as keys, where X is the variable index (e.g., 'g5', 'g120').
            *   Bit 4 (0x10): Include recent event summaries. These are expected to be stored as a series of null-terminated Z-encoded strings. A new Header field, `recent_events_buffer_ptr` (type: ADDR, 8 bytes), points to the start of this buffer. Another Header field, `recent_events_count_ptr` (type: ADDR, 8 bytes), points to a 1-byte variable holding the current number of valid event strings in the buffer (acting as a circular buffer index or count). The VM reads up to a reasonable maximum (e.g., 5-10) of these strings. If `recent_events_buffer_ptr` or `recent_events_count_ptr` is 0, no events are included.
            *   (Further bits can be defined for more specific context elements like character states, quest logs, etc.)
        *   `max_depth (SC/VAR)`: Maximum depth for exploring object trees (e.g., contents of containers within containers).
        *   `output_buffer_addr (ADDR)`: Byte address of a buffer where the JSON string will be written by the VM.
        *   `max_output_len (LC/VAR)`: Maximum number of bytes for `output_buffer_addr`.
    *   **Stores**:
        *   `status_code (VAR)`:
            *   `0`: Success, JSON context written to `output_buffer_addr`.
            *   `1`: Output Buffer Too Small (JSON string generation was truncated or not attempted).
            *   `2`: Invalid Scope Flag (unrecognized bits set).
            *   `3`: Error during JSON generation (internal VM error).
    *   **Conceptual JSON Output (Example content in `output_buffer_addr`)**:
        ```json
        {
          "player_location_id": "obj_room10",
          "player_inventory": [
            {"name": "brass key", "id": "obj_key5", "attributes": ["portable"]},
            {"name": "lantern", "id": "obj_lantern2", "attributes": ["portable", "lit"]}
          ],
          "visible_objects": [
            {"name": "large oak table", "id": "obj_table1", "attributes": ["scenery"], "properties": {"description_ptr": "str_table_desc"}},
            {"name": "red book", "id": "obj_book3", "parent_id": "obj_table1", "attributes": ["portable"]}
          ],
          "global_vars_subset": {
            "g_quest_stage_dragon": 1,
            "g_time_of_day": 1800
          },
          "recent_events": [ // This part is highly game-dependent on how events are stored and made available
            "The north door creaked open.",
            "You heard a distant roar."
          ]
        }
        ```
    *   **Notes**:
        *   The VM would need a robust internal JSON generation capability.
        *   The exact structure of the JSON can be standardized by the ZM2 specification or be somewhat flexible, with the LLM prompts engineered to understand the provided structure. The provided JSON structure in the "Conceptual JSON Output" is an illustrative example. While the keys shown (e.g., `player_location_id`, `player_inventory`, `visible_objects`, `global_vars_subset`, `recent_events`) are recommended for consistency if the corresponding `object_scope_flag` bits are set, LLM prompts should be engineered to be flexible. The VM guarantees well-formed JSON if the operation is successful.
        *   This opcode helps abstract away complex Z-code for gathering common contextual data.

**Extended Opcodes for LLM Integration (Asynchronous):**

These opcodes facilitate non-blocking interaction with an external LLM. They require the VM to have networking capabilities and access to the LLM API endpoint and parameters defined in the header. The general flow is to initiate a request, receive a handle, periodically check the status using the handle, and then retrieve the result once processing is complete.

1.  **`start_llm_parse (input_text_addr, context_data_addr, result_buffer_addr, max_result_len) -> (handle)`**
    *   **Functionality**: Initiates an asynchronous LLM parsing task. This opcode is non-blocking; it submits the request and returns a handle immediately.
    *   **Operands**:
        *   `input_text_addr (ADDR)`: Byte address of a null-terminated Z-encoded string containing the player's raw input.
        *   `context_data_addr (ADDR)`: Byte address of a structured data block (e.g., JSON string or custom binary format) containing relevant game state context.
        *   `result_buffer_addr (ADDR)`: Byte address of a buffer where the LLM's structured response will eventually be written by the VM upon successful completion.
        *   `max_result_len (LC/VAR)`: Maximum number of bytes anticipated for `result_buffer_addr`.
    *   **Stores**:
        *   `handle (VAR)`: A unique 64-bit identifier for this asynchronous request. This handle is used with `check_llm_status` and `get_llm_result`. A value of 0 could indicate an immediate failure to start the task (e.g., invalid parameters).
    *   **Conceptual API Call Parameters (Prepared by VM for later transmission)**:
        *   Endpoint: `llm_api_endpoint_ptr` from header.
        *   Payload: Typically JSON. Example:
            ```json
            {
              "model": "story_parser_v1", // from llm_parameters_ptr
              "prompt": "Parse the following player command based on the game context.",
              "player_input": "...", // from input_text_addr
              "game_context": { ... }, // from context_data_addr
              "output_format": "structured_action_v1"
            }
            ```

2.  **`start_llm_generate (prompt_text_addr, context_data_addr, result_buffer_addr, max_result_len, creativity_level) -> (handle)`**
    *   **Functionality**: Initiates an asynchronous LLM text generation task. This opcode is non-blocking.
    *   **Operands**:
        *   `prompt_text_addr (ADDR)`: Byte address of a null-terminated Z-encoded string containing the prompt for the LLM.
        *   `context_data_addr (ADDR)`: Byte address of a structured data block for context.
        *   `result_buffer_addr (ADDR)`: Byte address of a buffer where the LLM's generated Z-encoded text will eventually be written.
        *   `max_result_len (LC/VAR)`: Maximum number of bytes anticipated for `result_buffer_addr`.
        *   `creativity_level (SC/VAR)`: A value (e.g., 0-100) suggesting the desired creativity/randomness (temperature) for the LLM.
    *   **Stores**:
        *   `handle (VAR)`: A unique 64-bit identifier for this asynchronous request. A value of 0 could indicate an immediate failure.
    *   **Conceptual API Call Parameters (Prepared by VM for later transmission)**:
        *   Endpoint: `llm_api_endpoint_ptr` from header.
        *   Payload: Typically JSON. Example:
            ```json
            {
              "model": "story_generator_v1", // from llm_parameters_ptr
              "prompt_text": "...", // from prompt_text_addr
              "game_context": { ... }, // from context_data_addr
              "max_tokens": N, // derived from max_result_len
              "temperature": X // from creativity_level
            }
            ```

3.  **`check_llm_status (handle) -> (status_code)`**
    *   **Functionality**: Polls the status of a pending LLM request initiated by `start_llm_parse` or `start_llm_generate`. This opcode is non-blocking. The VM manages the actual HTTP request and response handling internally based on the handle.
    *   **Operand**:
        *   `handle (VAR)`: The unique identifier returned by a `start_llm_*` opcode.
    *   **Stores**:
        *   `status_code (VAR)`:
            *   `0`: In Progress (request is still being processed by the LLM or network).
            *   `1`: Success (LLM processing complete, result is ready in the designated `result_buffer_addr`).
            *   `2`: Failed (network error, API error, timeout during HTTP request).
            *   `3`: Invalid Handle (the provided handle does not correspond to an active request).
            *   `4`: LLM Processing Error (LLM service returned an error or could not parse/generate meaningfully).
            *   `5`: Result Buffer Too Small (if VM can detect this before writing the full result, otherwise this might be caught by `get_llm_result` or be truncated).

4.  **`get_llm_result (handle, result_buffer_addr) -> (status_code)`**
    *   **Functionality**: Retrieves or confirms the result of a successful LLM operation. This opcode is called after `check_llm_status` returns `1` (Success). The primary purpose is to formally complete the asynchronous pattern and ensure the Z-code acknowledges the result is ready in the buffer specified during the `start_llm_*` call. The VM would have already written the data to `result_buffer_addr` upon successful completion of the LLM task.
    *   **Operands**:
        *   `handle (VAR)`: The unique identifier of the completed LLM request.
        *   `result_buffer_addr (ADDR)`: Byte address of the buffer where the result was written (should match the one provided to the `start_llm_*` call). This operand is for confirmation and to maintain consistency in the opcode pattern.
    *   **Stores**:
        *   `status_code (VAR)`:
            *   `0`: Success (result is confirmed to be in `result_buffer_addr`).
            *   `1`: Error Retrieving Result (e.g., handle was valid and indicated success, but data is unexpectedly missing or `result_buffer_addr` doesn't match internal record; this should be rare).
            *   `2`: Invalid Handle.
    *   **Expected LLM Response data (already written to `result_buffer_addr` by the VM when `check_llm_status` first indicated success)**:
        *   For `start_llm_parse`: JSON string as described previously.
        *   For `start_llm_generate`: Z-encoded string as described previously.
        *   The VM handles the conversion from LLM output (e.g., plain text UTF-8) to the Z-Machine storable format (Z-encoded string or structured JSON string) *before* `check_llm_status` reports success.

**Error Handling for Invalid Opcodes or Operands:**

*   **Invalid Opcode**: If the VM encounters an opcode number that is not defined for the current Z-Machine version:
    *   The VM SHOULD halt with a fatal error message "Undefined opcode X at address Y".
    *   Alternatively, it MAY treat it as a `nop` and continue, but this is generally discouraged as it indicates a corrupted or incompatible story file. A flag in the header might control this behavior for debugging.
*   **Invalid Operand Type**: If an opcode expects a variable reference but gets a constant, or vice-versa, where the type is unambiguous from the opcode definition itself:
    *   The VM SHOULD halt with a fatal error message "Invalid operand type for opcode X at address Y".
*   **Invalid Variable Reference**: If an operand specifies a variable L00-L15 but the current routine defines fewer local variables, or a global G00-G239 that is out of the defined range:
    *   The VM SHOULD halt with a fatal error message "Invalid variable reference for opcode X at address Y".
*   **Address Out of Bounds**: If an opcode attempts to read or write memory (e.g., `loadw`, `storew`, `print_addr`) using an address that is outside the valid memory map (0 to `dynamic_data_section_start + dynamic_data_section_length - 1`, or specific section bounds if applicable):
    *   The VM SHOULD halt with a fatal error message "Memory access violation at address Z by opcode X at address Y".
*   **Division by Zero**: For `div` or `mod` opcodes where the divisor is zero:
    *   The VM SHOULD halt with a fatal error message "Division by zero for opcode X at address Y".
    *   Alternatively, it MAY return a specific value (e.g., 0 or MAX_INT) and set a status flag, if game recovery is prioritized. This behavior should be specified by a header flag.
*   **Stack Underflow/Overflow**: If a `pop` or `ret` occurs on an empty stack, or a `push` or `call` occurs on a full stack (if the stack has a fixed size within dynamic memory):
    *   The VM SHOULD halt with a fatal error "Stack underflow/overflow by opcode X at address Y".

Error messages should, where possible, include the current Program Counter (address of the faulting instruction) and any relevant operand values to aid debugging.
The general philosophy is to fail fast and clearly for programmer errors (invalid opcodes, bad variable refs) but provide mechanisms for game logic to handle runtime issues from LLM interactions (via status codes).

## 4.A Detailed Opcode Specification

### 4.A.0 Packed Address (PADDR) Encoding and Expansion

Before detailing individual opcodes, it's crucial to understand Packed Addresses (PADDRs), a mechanism used to efficiently encode addresses within instruction streams.

**1. Purpose of PADDRs:**
PADDRs are primarily used to save space in the instruction stream when an opcode needs to refer to an address within the **Code Section** (e.g., for a routine call) or the **Static Data Section** (e.g., for printing a string). Instead of embedding a full 64-bit absolute address directly as an operand, a shorter, packed representation is used.

**2. PADDR Encoding in Instructions:**
A PADDR is a 32-bit value embedded directly in the instruction stream as part of an opcode's operands. This 32-bit value is read as an unsigned integer, adhering to the Z-Machine Mark 2's general rule of Big Endian byte order for multi-byte values within instruction streams.

**3. PADDR Expansion Formula:**
The VM expands a PADDR into a full 64-bit absolute byte address at runtime using the following general formula:

`AbsoluteByteAddress = BaseAddress + PackedValue`

Where:
*   `PackedValue`: The 32-bit unsigned integer value read from the instruction stream.
*   `BaseAddress`: A 64-bit absolute byte address determined by the type of PADDR or the context of the opcode using it.

**4. Base Addresses and PADDR Types:**
The `BaseAddress` used in the expansion formula depends on the type of resource the PADDR refers to. For Z-Machine Mark 2, the following PADDR types are defined:

*   **`RoutinePADDR`**:
    *   Used by opcodes like `call` to refer to the starting address of a routine within the Code Section.
    *   `BaseAddress`: The value of `code_section_start` (obtained from the Header).
    *   Expansion: `AbsoluteByteAddress = code_section_start + PackedValue`
    *   `PackedValue` is the 32-bit unsigned offset from the start of the Code Section.

*   **`StringPADDR`**:
    *   Used by opcodes like `print_paddr` (a hypothetical opcode for printing packed address strings) to refer to a Z-encoded string within the Static Data Section.
    *   `BaseAddress`: The value of `static_data_section_start` (obtained from the Header).
    *   Expansion: `AbsoluteByteAddress = static_data_section_start + PackedValue`
    *   `PackedValue` is the 32-bit unsigned offset from the start of the Static Data Section.

The type of PADDR (RoutinePADDR vs. StringPADDR) is typically implicitly determined by the opcode that consumes it. For example, a `call` opcode always expects a `RoutinePADDR`.

**5. Scale Factor:**
For the current specification, the `ScaleFactor` in the conceptual formula `BaseAddress + (PackedValue * ScaleFactor)` is **1** for both `RoutinePADDR` and `StringPADDR`. This means the `PackedValue` is a direct byte offset from the respective base address.

*Note: Future revisions may introduce different scale factors or additional PADDR types if finer granularity (e.g., word-addressing for specific data structures) or larger relative packed address ranges (beyond 4GB with a 32-bit value) are needed for specific purposes. For now, PADDRs are 32-bit byte offsets from their respective section bases.*

**6. Limitations:**
The use of a 32-bit `PackedValue` with a `ScaleFactor` of 1 imposes an addressing limitation:
*   A `RoutinePADDR` can only address locations up to 2<sup>32</sup> - 1 bytes (approximately 4GB) from the `code_section_start`.
*   A `StringPADDR` can only address locations up to 2<sup>32</sup> - 1 bytes (approximately 4GB) from the `static_data_section_start`.

Story files where targeted routines or strings reside beyond this 4GB offset from their respective section base addresses cannot use these PADDR types to refer to them and would require full 64-bit address operands or alternative mechanisms.

### 1. Introduction to Opcode Specification

This section provides the detailed specification for each Z-Machine Mark 2 opcode. Opcode numbers (values) will be assigned systematically. For opcodes adapted from the Z-Machine Standard 1.1, the original numbers may be used if they fit within a new, coherent numbering scheme designed for ZM2; otherwise, a clear mapping or new number will be provided. New ZM2-specific opcodes, particularly those for LLM integration or 64-bit specific operations, will have distinct numbers.

All multi-byte values within instruction streams, including the opcode itself and any literal operands, are stored in **Big Endian** format, consistent with the overall memory model of the Z-Machine Mark 2.

### 2. Opcode Definition Template

Each opcode will be defined using the following structure:

*   **`Mnemonic`**: The textual representation of the opcode (e.g., `add`).
*   **`Opcode Value`**: The unique hexadecimal value identifying the opcode (e.g., `0x0120`). Specific values are illustrative examples in this document.
*   **`Form`**: The general classification of the opcode based on its operand structure:
    *   `0OP`: No operands.
    *   `1OP`: One operand.
    *   `2OP`: Two operands.
    *   `VAROP`: Variable number of operands.
    *   `EXT`: Extended opcodes, typically for new ZM2 features like LLM interaction or specialized utilities.
*   **`Operand Types`**: A detailed list of operands the opcode takes. This includes:
    *   `type_byte(s)`: Description of byte(s) that specify the types of subsequent operands, if applicable (common in 2OP and VAROP).
    *   `operand1 (type)`: Name and type of the first operand (e.g., `value (Small Constant/Variable)`). Types can be Small Constant (SC), Large Constant (LC), Variable Reference (VAR), Packed Address (PADDR), Address (ADDR - which itself could be LC or VAR).
    *   `operand2 (type)`, ... `operandN (type)`: Subsequent operands.
    *   `store_variable_ref? (Variable Reference)`: Indicates if a variable reference for storing the result follows.
    *   `branch_data?`: Indicates if branch information follows the opcode and its operands.
*   **`Description`**: A concise summary of what the opcode does.
*   **`Operation Details`**: A step-by-step description of the opcode's logic. This includes:
    *   How operands are read from the instruction stream or memory.
    *   The computation or action performed.
    *   How the stack is affected (push, pop, frame creation/destruction).
    *   How the Program Counter (PC) is updated (e.g., standard advance, jump, call).
*   **`Stores Result To`**: Specifies where the primary result of the operation is stored (e.g., a specific variable, top of stack, an implicit location).
*   **`Branches If`**: For branching opcodes, the condition that causes a branch, and how the branch offset is determined and applied.
*   **`Side Effects`**: Any other changes to the VM state not covered by result storage or branching (e.g., modification of status flags, turn counters, I/O operations).
*   **`Error Conditions`**: Specific errors the opcode might trigger (e.g., division by zero, invalid object ID, stack overflow/underflow, address out of bounds).

### 3. Example Opcode Definitions

#### a. `rtrue` (0OP)

*   **`Mnemonic`**: `rtrue`
*   **`Opcode Value`**: `0x00B0` (Example)
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Returns true (1) from the current routine.
*   **`Operation Details`**:
    1.  The current routine's stack frame is validated and prepared for removal.
    2.  The Program Counter (PC) is set to the return address stored in the current stack frame.
    3.  The current stack frame is popped from the call stack, restoring the previous frame pointer.
*   **`Stores Result To`**: The 64-bit integer value `1` is stored into the variable indicated by the `store_variable_ref` operand of the `call` opcode that invoked the current routine.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Call stack depth decreases by one.
*   **`Error Conditions`**:
    *   Stack underflow if called when no routine call is active (e.g., from top-level execution).

#### b. `jz` (1OP)

*   **`Mnemonic`**: `jz`
*   **`Opcode Value`**: `0x0181` (Example)
*   **`Form`**: `1OP`
*   **`Operand Types`**: `value (Small Constant/Variable)`
*   **`Description`**: Jump if value is zero.
*   **`Operation Details`**:
    1.  Reads the `value` operand. If it's a Small Constant, it's used directly. If it's a Variable, its content is read.
    2.  If the (read) `value` is equal to 0, a branch is performed.
    3.  If the value is not 0, execution continues with the instruction immediately following the branch data.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `value == 0`.
    *   The branch information follows the opcode and its `value` operand.
    *   It consists of a single byte. Bit 7 determines if the branch is "on true" (1) or "on false" (0) - for `jz` this implies the condition itself determines if we branch. Bit 6 determines if the offset is 1 or 2 bytes. Bits 0-5 of this byte contain the top 6 bits of the offset.
    *   If bit 6 is 0, the next byte contains the lower 8 bits of the offset, forming a 14-bit signed offset (`((byte1 & 0x3F) << 8) | byte2`).
    *   If bit 6 is 1, the offset is implicitly short and the jump is to fixed locations (details TBD, or this form is simplified to always use 14-bit offset).
    *   For this example, let's assume a common Z-Machine branch: a 14-bit signed offset. The offset is calculated from the address *after* the branch data itself. A branch to offset 0 means "return false from current routine", offset 1 means "return true from current routine". Other values are added to PC.
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**: Invalid variable reference if `value` specifies an invalid variable.

#### c. `add` (2OP)

*   **`Mnemonic`**: `add`
*   **`Opcode Value`**: `0x0220` (Example)
*   **`Form`**: `2OP`
*   **`Operand Types`**: `operand1 (Small Constant/Variable)`, `operand2 (Small Constant/Variable)`, `store_variable_ref (Variable Reference)`
    *   Operand types for `operand1` and `operand2` are determined by type bytes following the opcode itself.
*   **`Description`**: Adds `operand1` and `operand2`, stores the 64-bit result in `store_variable_ref`.
*   **`Operation Details`**:
    1.  Reads `operand1`.
    2.  Reads `operand2`.
    3.  Computes `result = (operand1 + operand2) & 0xFFFFFFFFFFFFFFFF` (unsigned 64-bit addition, wraps on overflow).
    4.  Stores `result` into the variable specified by `store_variable_ref`.
    5.  PC advances past this instruction and its operands.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid variable reference for operands or `store_variable_ref`.

#### d. `call` (VAROP)

*   **`Mnemonic`**: `call`
*   **`Opcode Value`**: `0x03E0` (Example)
*   **`Form`**: `VAROP`
*   **`Operand Types`**: `routine_paddr (PADDR)`, `arg0 (Small Constant/Variable)`, ..., `argN (Small Constant/Variable)` (up to 7 args), `store_variable_ref (Variable Reference)`
    *   Operand types for `arg0` through `argN` are determined by type byte(s) following the opcode.
    *   The `routine_paddr` is the first operand read.
*   **`Description`**: Calls the routine at the packed address `routine_paddr`, passing up to 7 arguments. Stores the routine's return value into `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `routine_paddr` operand. Resolve it to an absolute byte address in the Code Section. This is the target routine address.
    2.  Read the type byte(s) for arguments. For each argument supplied:
        *   Read the argument value according to its type (Small Constant or Variable).
    3.  At the target routine address, the first byte specifies the number of local variables (0-15) for that routine.
    4.  Push a new stack frame onto the call stack. The frame contains:
        *   Return PC (address of the instruction after this `call` opcode and all its operands).
        *   Previous frame pointer.
        *   The `store_variable_ref` where the routine's result will be stored.
        *   Number of arguments supplied to the call.
        *   Arguments themselves (arg0, arg1, ... argN).
        *   Space for local variables (initialized to 0), count taken from routine header.
    5.  Set PC to the target routine address, immediately after the byte specifying number of locals.
*   **`Stores Result To`**: The routine's return value (from `ret`, `rtrue`, `rfalse`) will be stored in `store_variable_ref` when the called routine returns.
*   **`Branches If`**: Does not branch itself, but transfers control to the called routine.
*   **`Side Effects`**: Call stack depth increases. A new stack frame is created.
*   **`Error Conditions`**:
    *   Invalid `routine_paddr` (e.g., points outside code section, not to a valid routine).
    *   Stack overflow if call stack exceeds its maximum depth.
    *   Invalid variable references for arguments or `store_variable_ref`.
    *   Calling a routine with more arguments than it's designed to handle (behavior TBD, may truncate or error).

#### e. `start_llm_parse` (EXT)

*   **`Mnemonic`**: `start_llm_parse`
*   **`Opcode Value`**: `0xEE00` (Example, assuming `EE` prefix for EXT LLM opcodes)
*   **`Form`**: `EXT`
*   **`Operand Types`**: `input_text_addr (Large Constant/Variable Address)`, `context_data_addr (Large Constant/Variable Address)`, `result_buffer_addr (Large Constant/Variable Address)`, `max_result_len (Large Constant/Variable)`, `store_handle_variable_ref (Variable Reference)`
    *   Operand types determined by a type byte following the opcode.
*   **`Description`**: Initiates an asynchronous LLM parsing task for the ZSCII string at `input_text_addr` using context from `context_data_addr`. The LLM's structured response will eventually be placed in `result_buffer_addr`.
*   **`Operation Details`**:
    1.  Read all operands according to their types.
    2.  Validate `input_text_addr`, `context_data_addr`, and `result_buffer_addr` (e.g., check if they are valid memory regions).
    3.  Validate `max_result_len` (e.g., must be > 0).
    4.  Check `flags1.LLMParseEnable`. If this flag is OFF (0):
        *   Store 0 (or a specific error code like `LLM_ERR_DISABLED`) into `store_handle_variable_ref`.
        *   Return (PC advances past this instruction and its operands).
    5.  Generate a new, unique 64-bit handle for this LLM request. This handle should not be 0.
    6.  The VM internally records the request details: handle, input text address, context data address, result buffer address, max result length, and the type of request (parse). This information is queued for the VM's asynchronous LLM task manager.
    7.  The actual HTTP API call to the LLM service is *not* made synchronously by this opcode. It is managed by the VM's internal processes, triggered by game calls to `check_llm_status`.
*   **`Stores Result To`**: The newly generated 64-bit `handle` (or 0/error code if `LLMParseEnable` is false or immediate validation fails) is stored in `store_handle_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: An LLM request is queued within the VM.
*   **`Error Conditions`**:
    *   Invalid memory address for `input_text_addr`, `context_data_addr`, or `result_buffer_addr`.
    *   `max_result_len` is zero or unreasonably small.
    *   `LLMParseEnable` flag in `flags1` is false.
    *   Failure to generate a unique handle (highly unlikely).
    *   Invalid variable reference for `store_handle_variable_ref` or any variable operands.

#### f. `get_context_as_json` (EXT)

*   **`Mnemonic`**: `get_context_as_json`
*   **`Opcode Value`**: `0xEE02` (Example)
*   **`Form`**: `EXT`
*   **`Operand Types`**: `object_scope_flag (Small Constant/Variable)`, `max_depth (Small Constant/Variable)`, `output_buffer_addr (Large Constant/Variable Address)`, `max_output_len (Large Constant/Variable)`, `store_status_variable_ref (Variable Reference)`
    *   Operand types determined by a type byte following the opcode.
*   **`Description`**: Collects relevant game state information based on `object_scope_flag` and `max_depth`, formats it as a JSON string, and writes it to `output_buffer_addr`.
*   **`Operation Details`**:
    1.  Read all operands according to their types.
    2.  Validate `output_buffer_addr` and `max_output_len`.
    3.  Based on `object_scope_flag` bits and `max_depth`:
        *   Gather player inventory.
        *   Gather objects in the current location.
        *   Gather specified global variables.
        *   (Other context elements as defined by the flag).
    4.  Construct a JSON string representing this gathered information.
    5.  Get the length of the generated JSON string.
    6.  If JSON string length (including null terminator) > `max_output_len`:
        *   Store status code `1` (Output Buffer Too Small) into `store_status_variable_ref`.
        *   Optionally, write a truncated (but still valid, if possible) JSON or an error JSON to `output_buffer_addr`.
        *   Return.
    7.  Write the complete JSON string (including null terminator) to `output_buffer_addr`.
    8.  Store status code `0` (Success) into `store_status_variable_ref`.
*   **`Stores Result To`**: A `status_code` is stored in `store_status_variable_ref`:
    *   `0`: Success.
    *   `1`: Output Buffer Too Small.
    *   `2`: Invalid Scope Flag (unrecognized bits set in `object_scope_flag`).
    *   `3`: Internal JSON Generation Error (e.g., VM failed to serialize data).
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Memory at `output_buffer_addr` is overwritten.
*   **`Error Conditions`**:
    *   Invalid `output_buffer_addr`.
    *   `max_output_len` is too small to hold even a minimal JSON structure (e.g., `{}`).
    *   Invalid variable references for operands or `store_status_variable_ref`.

#### g. `sread` (VAROP)

*   **`Mnemonic`**: `sread`
*   **`Opcode Value`**: `0x03E8` (Example)
*   **`Form`**: `VAROP`
*   **`Operand Types`**: `text_buffer_addr (Large Constant/Variable Address)`, `parse_buffer_addr (Large Constant/Variable Address)` (Optional, 0 if not used)
    *   Operand types determined by a type byte following the opcode.
*   **`Description`**: Reads player input into `text_buffer_addr`. If `parse_buffer_addr` is non-zero, tokenizes the input using the game's dictionary and stores the parse information in `parse_buffer_addr`. This opcode does not directly interact with the LLM.
*   **`Operation Details`**:
    1.  Read `text_buffer_addr` and `parse_buffer_addr` operands.
    2.  Display an input prompt to the player (e.g., "> ").
    3.  Read a line of text from the player. The maximum number of characters to read should be taken from the first byte of `text_buffer_addr` (similar to Z-Spec 1.1, S13.3 for V1-V4, or a fixed reasonable maximum for V5+ like 255 characters, adapted for ZM2). Store the actual number of characters read into the second byte of `text_buffer_addr`, followed by the ZSCII characters themselves. The text is null-terminated if space allows.
    4.  **Parse Buffer Processing**: If `parse_buffer_addr` is non-zero:
        *   The traditional Z-Machine parsing routine is invoked.
        *   **Reference Z-Machine Standard:** The format of the data written to `parse_buffer_addr` closely follows the Z-Machine Standard 1.1, Section 13 ('The Dictionary and Lexical Analysis'). However, adaptations for Z-Machine Mark 2's 64-bit architecture apply:
        *   **Maximum Words:** The first byte of `parse_buffer_addr` specifies the maximum number of textual words to parse from the input.
        *   **Number of Parsed Words:** The second byte of `parse_buffer_addr` will store the actual number of words parsed by the routine.
        *   **Word Entry Structure:** Each subsequent word entry in the parse buffer, for each parsed word, will consist of:
            *   `dictionary_address` (64-bit): The byte address of the matched word in the dictionary (pointed to by `dictionary_start` in the Header), or 0 if the word is not found in the dictionary.
            *   `length_of_word` (8-bit): The number of ZSCII characters in the textual word from the input.
            *   `start_offset_in_text_buffer` (32-bit): The byte offset of this word's first character within the `text_buffer_addr` (where the input ZSCII string is stored).
        *   Each word entry will thus be 8 + 1 + 4 = 13 bytes. (Note: For alignment or simplicity, entries could be padded to 16 bytes in a final implementation, with the last 3 bytes reserved. This specification currently assumes unpadded 13-byte entries.)
        *   **Dictionary Format:** The dictionary itself (pointed to by `dictionary_start` in the Header) follows Z-Spec 1.1 structure (e.g., word separator characters, entry length, number of entries, Z-encoded words), but all dictionary entry addresses are byte addresses. Word separation is done using the defined word separator characters from the dictionary header.
    5.  PC advances past this instruction and its operands.
*   **`Stores Result To`**: Does not directly store a result in a variable (unlike `aread`). Status of read (e.g., characters read) is in `text_buffer_addr`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Memory at `text_buffer_addr` and `parse_buffer_addr` (if provided) is overwritten. Player I/O occurs.
*   **`Error Conditions`**:
    *   Invalid `text_buffer_addr` or `parse_buffer_addr`.
    *   Buffer sizes specified in `text_buffer_addr` or `parse_buffer_addr` being too small.

**Note on `aread`:** The `aread` opcode, when implemented with parsing enabled (via a non-zero `parse_buffer_addr`), uses the same `parse_buffer_addr` format as described for `sread`.

### 4. Note on Completeness

The opcodes listed above are examples provided to illustrate the required level of detail and format for specification. A complete Z-Machine Mark 2 implementation would necessitate defining *all* opcodes from the Z-Machine Standard 1.1 in this manner. This includes adapting them for 64-bit operations, 64-bit addresses, ensuring correct handling of Packed Addresses (PADDRs) where appropriate (especially for routine calls and string addresses), and verifying operand types. Furthermore, all new ZM2-specific opcodes, such as the full suite of LLM interaction opcodes (`start_llm_generate`, `check_llm_status`, `get_llm_result`) and any other utility or extended opcodes, must also be defined with this level of rigor.

The final assignment of numerical opcode values needs to be done systematically to avoid clashes across different opcode forms (0OP, 1OP, 2OP, VAROP, EXT) and to allow for logical grouping and potential future expansion. The example values used herein are illustrative and subject to a final, comprehensive numbering scheme.

## 5. LLM Integration

-   **Role of the LLM**: The LLM serves two primary functions within the Z-Machine Mark 2:
    -   **Natural Language Understanding (NLU)**: Interpreting potentially complex and nuanced player commands into a structured format that the Z-Machine's game logic can execute. This moves beyond simple verb-noun parsing to understand intent, multiple actions in one command, and context-dependent references.
    -   **Natural Language Generation (NLG)**: Creating rich, varied, and contextually aware descriptive text for locations, objects, characters, and events. This allows for more dynamic and less repetitive game experiences than traditionally hand-authored static text.

-   **Interface with Hugging Face API**: The Z-Machine Mark 2 will primarily use Hugging Face's Inference API for accessing LLMs. This approach allows for flexibility in choosing models and avoids the need to host models locally, though local model inference is a potential future extension.

    **Step-by-Step Hugging Face API Integration Guide:**

    1.  **Account Setup**: Create an account on [Hugging Face](https://huggingface.co/).
    2.  **API Token Generation**:
        *   Navigate to your Hugging Face account settings.
        *   Go to "Access Tokens" and generate a new token with "read" or "write" permissions depending on whether you plan to use models that require fine-tuning or specific gated access. For inference with public models, "read" is usually sufficient.
        *   Securely store this API token. It will be required by the Z-Machine VM to authenticate API requests. *The method for providing this token to the VM (e.g., environment variable, configuration file read by the interpreter) is an implementation detail of the VM interpreter, not the story file.*
    3.  **Model Selection**:
        *   Browse the [Hugging Face Model Hub](https://huggingface.co/models) to find suitable models.
            *   For NLU (Parsing): Look for models fine-tuned for instruction following, question answering, or text-to-SQL/code tasks if a highly structured output is desired. Examples: `distilbert-base-uncased-finetuned-sst-2-english` (for sentiment, adaptable), or more general models like `gpt2` or `Llama-2` that can be prompted for structured output.
            *   For NLG (Generation): Look for models fine-tuned for text generation, creative writing, or dialogue. Examples: `gpt2`, `Llama-2`, `mistralai/Mistral-7B-Instruct-v0.1`.
        *   The chosen model's identifier (e.g., "gpt2" or "mistralai/Mistral-7B-Instruct-v0.1") will be part of the `llm_parameters_ptr` data in the story file header, or a default can be used by the interpreter.
    4.  **API Endpoint Construction**:
        *   The base URL for Inference API is typically `https://api-inference.huggingface.co/models/<MODEL_ID>`.
        *   The `llm_api_endpoint_ptr` in the story file header should point to this base URL or a custom URL if using a self-hosted inference endpoint or a different service compatible with Hugging Face's API structure.
    5.  **Making API Calls (from VM, triggered by `check_llm_status` for a new handle or as needed)**:
        *   When `check_llm_status` is called for a handle for the first time, or if the request is genuinely pending, the VM's HTTP client will construct and send a POST request if it hasn't already.
        *   **Headers**:
            *   `Authorization: Bearer YOUR_API_TOKEN`
            *   `Content-Type: application/json`
        *   **Body**: A JSON payload specific to the task (NLU or NLG) and model, using data referenced by addresses provided in the `start_llm_*` call. See "Data Structures for LLM Communication" below.
    6.  **Handling Responses (internally by VM)**:
        *   The VM asynchronously receives and parses the JSON response from the API.
        *   For NLU: Extracts the structured action.
        *   For NLG: Extracts the generated text and converts it to Z-encoded format.
        *   The VM updates the internal state associated with the handle (e.g., to "Success", "Failed"). If successful, it writes the processed data to the `result_buffer_addr` specified in the `start_llm_*` call.
        *   Subsequent calls to `check_llm_status` for that handle will then return the updated status. If the status is "Success", `get_llm_result` can be called.

-   **Data Structures for LLM Communication**: The `start_llm_parse` and `start_llm_generate` opcodes define operands for `input_text_addr`, `context_data_addr`, and `result_buffer_addr`. The data at these addresses will typically be formatted as JSON strings, though the design allows for other structured binary formats if optimized. To ensure interoperability and simplify LLM fine-tuning, the following JSON structures are defined as the **normative standard** for communication between the Z-Machine Mark 2 VM and the LLM. While the VM's JSON parser for LLM responses should be somewhat robust to minor structural variations (e.g., extra fields returned by the LLM), the VM will always *construct* its requests to the LLM adhering to this defined structure. LLMs fine-tuned for Z-Machine Mark 2 should be trained to expect input in this format and produce output that aligns with it.

    *   **Sending Information to LLM (NLU - initiated by `start_llm_parse`)**:
        *   `input_text_addr`: Points to a Z-encoded string (e.g., "take the red key from the oak table"). This is converted to a plain string for the JSON payload by the VM.
        *   `context_data_addr`: Points to a JSON string providing context.
            ```json
            // Conceptual content at context_data_addr for NLU
            {
              "current_location": "Dusty Library",
              "visible_objects": [
                {"name": "oak table", "id": "obj12", "description": "A sturdy oak table."},
                {"name": "small brass key", "id": "obj15", "location": "on oak table"},
                {"name": "book of spells", "id": "obj22", "location": "on shelf"}
              ],
              "player_inventory": ["rusty lantern", "note"],
              "recent_events": ["The door to the north creaked open."],
              "task_prompt": "Parse the player's command into a structured action: {verb, noun1, preposition, noun2}. Noun phrases should match visible objects or inventory if possible."
            }
            ```
        *   **Hugging Face API Payload (constructed by VM):**
            ```json
            {
              "inputs": "Player command: 'take the red key from the oak table'. Context: Current location is Dusty Library. Visible objects: oak table (obj12), small brass key (obj15) on oak table, book of spells (obj22) on shelf. Player inventory: rusty lantern, note. Recent events: The door to the north creaked open. Task: Parse the player's command into a structured action: {verb, noun1, preposition, noun2}. Noun phrases should match visible objects or inventory if possible.",
              "parameters": { // Model-specific parameters
                "max_new_tokens": 50,
                "return_full_text": false
                // Other parameters like temperature, top_k, etc. can be added
              }
            }
            ```
            This payload structure, particularly the content of the `inputs` field combining player command, context, and task prompt, is the standard format the VM will send for NLU requests.

    *   **Receiving Information from LLM (NLU - populated in `result_buffer_addr` by VM before `get_llm_result` confirms it)**:
        *   The LLM is expected to return a JSON structure that corresponds to the 'Task: Parse the player's command...' part of the prompt. The standard format for this structured action, which the VM will parse from the LLM's `generated_text` field, is:
            ```json
            // Conceptual content written to result_buffer_addr by VM after successful LLM NLU task
            // This is the JSON part of the Hugging Face API response
            [ // Some models return a list, with the actual JSON object as the value of a field.
              {
                "generated_text": { // The LLM directly returns a JSON object here
                    "action": "take",
                    "noun1": "small brass key", // or "obj15" if IDs are preferred
                    "preposition": "from",
                    "noun2": "oak table"   // or "obj12"
                }
              }
            ]
            // Or, if the API can return the JSON directly:
            // { "action": "take", "noun1": "small brass key", ... }
            // The VM's JSON parser should be robust enough to handle the Hugging Face API's response structure
            // and directly parse the meaningful JSON object from the relevant field (e.g., `generated_text`).
            // The goal is to avoid requiring Z-code to perform a second parsing step on a stringified JSON.
            ```
            This structured action format is the standard the VM anticipates for NLU results. Game logic will rely on these field names (`action`, `noun1`, `preposition`, `noun2`, etc.) being present in the JSON stored in `result_buffer_addr`.
        *   The Z-Machine game logic then uses this structured data (e.g., verb "take", noun1 "small brass key", etc.) from `result_buffer_addr` after `get_llm_result` confirms success.

    *   **Sending Information to LLM (NLG - initiated by `start_llm_generate`)**:
        *   `prompt_text_addr`: Points to a Z-encoded string (e.g., "The player enters the Dragon's Lair. Describe it vividly.").
        *   `context_data_addr`: Points to a JSON string providing context.
            ```json
            // Conceptual content at context_data_addr for NLG
            {
              "current_location": "Dragon's Lair",
              "location_description_style": "ominous, ancient",
              "key_elements_present": ["hoard of gold", "sleeping dragon", "flickering torches"],
              "player_mood": "apprehensive"
            }
            ```
        *   **Hugging Face API Payload (constructed by VM):**
            ```json
            {
              "inputs": "Prompt: 'The player enters the Dragon's Lair. Describe it vividly.' Context: Style should be ominous and ancient. Key elements to include: hoard of gold, sleeping dragon, flickering torches. Player is apprehensive.",
              "parameters": {
                "max_new_tokens": 200, // Derived from max_result_len
                "temperature": 0.8, // Derived from creativity_level operand
                "do_sample": true
                // Other generation parameters
              }
            }
            ```
            This payload structure, particularly the content of the `inputs` field combining the prompt and context, is the standard format the VM will send for NLG requests.

    *   **Receiving Information from LLM (NLG - `result_buffer_addr`)**:
        *   The LLM returns generated text.
            ```json
            // Conceptual content written to result_buffer_addr by VM after LLM NLG call
            [
              {
                "generated_text": "The air in the Dragon's Lair is thick with the smell of sulfur and ancient dust. Mountains of glittering gold coins and jewels rise in shimmering heaps, catching the unsteady light of flickering torches. A colossal red dragon, scales like obsidian shields, lies sleeping atop the largest hoard, its chest rising and falling with a sound like distant thunder..."
              }
            ]
            // The VM extracts "generated_text", then converts it to Z-encoded format.
            ```
            The VM expects the LLM's response for NLG tasks to be in this format, specifically looking for the `generated_text` field within the first element of the top-level array (as is common with Hugging Face Inference API). The VM will then extract and process this text.

-   **Strategies for Fine-Tuning the LLM**:
    While pre-trained models can be powerful, fine-tuning an LLM on domain-specific data (interactive fiction commands, responses, and descriptive text) can significantly improve its performance and relevance.

    1.  **Dataset Preparation**:
        *   **NLU (Parsing)**: Create a dataset of `(player_input, game_context, structured_output)` triples.
            *   `player_input`: "go north", "take the sword", "ask the wizard about the amulet"
            *   `game_context`: Simplified JSON representation of the relevant game state at the time of the command.
            *   `structured_output`: The desired parsed command, e.g., `{"verb": "go", "direction": "north"}`, `{"verb": "take", "object": "sword"}`, `{"verb": "ask", "character": "wizard", "topic": "amulet"}`.
            *   Sources: Existing IF game logs, manually authored examples, or data from IF development systems like Inform 7.
        *   **NLG (Generation)**: Create a dataset of `(prompt, game_context, generated_text)` triples.
            *   `prompt`: "Describe the forest clearing.", "The NPC offers you a quest."
            *   `game_context`: Relevant game state.
            *   `generated_text`: High-quality, well-styled text appropriate for the prompt and context.
            *   Sources: High-quality IF game text, descriptive writing exercises.

    2.  **Fine-Tuning Process (using Hugging Face tools like `transformers` library and `AutoTrain` or custom scripts)**:
        *   **Choose a Base Model**: Select a suitable pre-trained model from the Hugging Face Hub (e.g., GPT-2, Llama 2, Mistral, T5). Smaller models are faster and cheaper to fine-tune but may be less capable.
        *   **Format Data**: Convert your dataset into a format required by the chosen fine-tuning script/service (often JSONL or CSV).
        *   **Training**:
            *   Use Hugging Face's `Trainer` API in the `transformers` library for more control.
            *   Alternatively, use Hugging Face AutoTrain for a no-code solution if your data fits its requirements.
            *   Set hyperparameters (learning rate, batch size, number of epochs).
            *   Monitor training progress (loss, evaluation metrics).
        *   **Evaluation**: Evaluate the fine-tuned model on a held-out test set to measure its performance on unseen data. Metrics could include accuracy for NLU (exact match of structured output) or perplexity/BLEU scores for NLG, as well as qualitative human evaluation.
        *   **Deployment**: Once satisfied, the fine-tuned model can be pushed to the Hugging Face Model Hub (private or public) and then accessed via the Inference API using its new model ID.

    3.  **Example Datasets & Resources**:
        *   **IF Datasets (may require adaptation)**:
            *   Jericho-QA: A dataset of question-answering based on text adventure games. ([https://github.com/google-research-datasets/jericho-qa](https://github.com/google-research-datasets/jericho-qa))
            *   CLUB (Command Language Understanding Benchmark): ([https://github.com/lil-lab/club](https://github.com/lil-lab/club)) - More general, but principles apply.
            *   TextWorld Commons: A collection of text games and an engine for generating new ones, useful for creating training data. ([https://www.microsoft.com/en-us/research/project/textworld/](https://www.microsoft.com/en-us/research/project/textworld/))
        *   **Hugging Face Fine-Tuning Resources**:
            *   [Fine-tuning with AutoTrain](https://huggingface.co/docs/autotrain/index)
            *   [Transformers `Trainer` API Documentation](https://huggingface.co/docs/transformers/main_classes/trainer)
            *   [Hugging Face Course, Chapter on Fine-tuning](https://huggingface.co/course/chapter7/1)
            *   Numerous blog posts and examples available on the Hugging Face website and community forums.

    4.  **Iterative Refinement**: Fine-tuning is often an iterative process. Collect more data, experiment with different base models and hyperparameters, and continuously evaluate to improve the LLM's capabilities within the Z-Machine Mark 2 context. Consider techniques like few-shot prompting or prompt engineering even with fine-tuned models to guide their behavior at runtime.

## 6. Game State Management

-   **State Representation**: The complete game state is encapsulated within the Z-Machine's memory, primarily within its Dynamic Data Section. This includes:
    *   **Global Variables**: Current values of all 240 global variables (G00-G239), each being a 64-bit word.
    *   **Object Data**: The current state of all game objects, including their parent-sibling-child relationships, attribute flags (e.g., `worn`, `lit`), and property values. While object definitions are in static memory, their dynamic state (e.g., current location, contents if a container, specific property values that can change) is part of the game state.
    *   **Player Status**: Key information about the player, such as current location (an object ID), inventory (list of object IDs), score, and turns taken. Some of these might be stored in dedicated global variables.
    *   **Program Counter (PC)**: The address of the next instruction to be executed. This is crucial for saving the exact point of execution.
    *   **Call Stack**: A record of active routine calls, including the return address for each call, local variables for each routine, and temporary values pushed during expression evaluation. The stack itself resides in the Dynamic Data Section.
    *   **Dynamic Memory Allocations**: Any memory dynamically allocated by the game (e.g., for custom data structures) within the Dynamic Data Section. The boundaries and contents of these allocations are part of the state.
    *   **LLM Interaction State (Optional but Recommended)**: Potentially a transcript or summary of recent interactions with the LLM, or flags indicating LLM availability/status, if this influences game logic beyond a single turn.

-   **Initialization from Story File**:
    1.  **Memory Allocation**: The VM interpreter allocates a contiguous block of RAM to hold the entire Z-Machine memory (Header, Code, Static Data, Dynamic Data). The total size is determined by the sum of lengths specified in the header, with `dynamic_data_section_length` being the initial size for the heap.
    2.  **Header Loading and Parsing**: The first part of the story file (e.g., 1024 bytes) is read into the start of the allocated memory. The VM parses the header fields to understand memory layout (section start addresses, lengths), version, story ID, etc.
    3.  **Code and Static Data Loading**: The VM reads the Code Section and Static Data Section from the story file directly into their respective memory locations, as specified by `code_section_start`, `code_section_length`, `static_data_section_start`, and `static_data_section_length` from the header.
    4.  **Dynamic Data Initialization**:
        *   The Dynamic Data Section is initialized according to the Z-Machine standard. Typically, this means it's largely zeroed out or set to a defined initial state.
        *   Initial values for global variables are copied from the `globals_table_start` (often part of the Static Data section or an "initial values" table for globals) into the actual global variable slots at the beginning of the Dynamic Data section (or wherever the VM designates global variable storage).
        *   The object table in the Static Data section provides the initial state of all objects (attributes, properties, relationships). This is considered the starting dynamic state.
    5.  **Setting Initial VM Registers/Pointers**:
        *   The Program Counter (PC) is set to the initial execution address, typically the start of the "main" game routine (this address is usually found at a specific offset in the Header or is a known convention).
        *   Stack Pointer (SP) is initialized to the base of the stack area within the Dynamic Data Section.
        *   Other internal VM registers are reset to their default states.
    6.  **Flags**: Header flags are interpreted, and VM behavior is adjusted accordingly (e.g., transcripting, font requirements).

-   **Game State Variable Storage and Modification**:
    *   **Global Variables (G00-G239)**: Stored as an array of 240 64-bit words at a fixed location, typically at the beginning of the Dynamic Data Section or referenced via a pointer from the header. They are accessed directly by opcodes like `store Gx, value` and `load Gx -> (result)`.
    *   **Local Variables (L00-L15)**: Stored on the game's call stack. When a routine is called, space for its local variables (as defined by the routine) is allocated on the stack. They are accessed relative to the current stack frame pointer. Opcodes like `store Lx, value` and `load Lx -> (result)` handle these. Local variables cease to exist when a routine returns.
    *   **Stack Variables (Temporary)**: The top of the stack can be used as a temporary variable, implicitly by some opcodes or explicitly using `push` and `pop`.
    *   **Object Attributes**: Stored as bitfields within each object's entry in the object table. Opcodes like `set_attr`, `clear_attr`, and `test_attr` manipulate these. The object table itself might be in static memory, but a copy of attributes (or the parts that can change) might be in dynamic memory if objects can be created/destroyed or attributes are highly dynamic. For simplicity, we can assume the primary object table attributes are part of the dynamic state that needs saving.
    *   **Object Properties**: Stored in property lists associated with each object. `get_prop`, `put_prop`, `get_prop_addr`, `get_prop_len` opcodes access these. Property data can be variable length. Changes to properties are direct modifications in the Dynamic Data Section (or a dynamic copy of object data).

-   **State Updates**:
    *   Game logic, executed as a sequence of Z-Machine opcodes, directly modifies these variables, object states, and stack contents.
    *   Player actions, once parsed (potentially by the LLM via the `start_llm_parse` / `check_llm_status` / `get_llm_result` sequence and then interpreted by game code), trigger routines that change the game state.
    *   For example, `insert_obj` changes parent-child relationships, `put_prop` changes an object's property, `store` changes a variable.
    *   The VM ensures that these changes are persistent within its current memory session until a `save` or `restore` operation.

-   **Serialization and Deserialization (Saving and Loading Games)**:
    The `save` and `restore` opcodes manage this process. The standard format for saved games is Quetzal, but with 64-bit adaptations.

    *   **Serialization (`save` opcode)**:
        1.  **Identify Dynamic Memory**: The core principle is to save the portions of memory that can change during gameplay. This primarily includes:
            *   The entire Dynamic Data Section (from `dynamic_data_section_start` for `dynamic_data_section_length` bytes). This implicitly saves global variables, current object property values if they are stored interleaved, the game stack, and any heap allocations.
            *   If object attributes or other parts of the object table are modified directly in the Static Data section (not recommended for clarity, but possible), those changed portions would also need to be identified and saved. A cleaner approach is to have a dynamic copy of mutable object aspects.
        2.  **Capture VM Registers**: Key VM registers must be saved:
            *   Program Counter (PC).
            *   Stack Pointer (SP).
            *   Current routine's frame pointer (if applicable).
            *   Potentially other internal state registers of the VM relevant for resuming execution.
        3.  **Quetzal Adaptation (Conceptual)**: A ZM2 Quetzal-like format would be used.
            *   **Header Chunk (`IFhd`)**: Contains the story file's release number, serial number (story ID), checksum, and initial PC. This is used to verify compatibility with the current story file when loading.
            *   **Memory Chunk (`CMem` or `UMem`)**:
                *   For `CMem`: The dynamic memory area is compressed (e.g., using a simple XOR difference from the initial state of dynamic memory, or a more complex algorithm like LZW). The chunk stores the compressed data.
                *   For `UMem`: The dynamic memory area is stored uncompressed. Given the potential size with 64-bit words, compression is recommended.
                *   The chunk needs to specify the start address and length of the memory block being saved.
            *   **Stack Chunk (`Stks`)**: Contains a snapshot of the call stack, including return addresses, local variables, and temporary values for each frame.
            *   **Program Counter Chunk (`PC__`)**: Explicitly stores the PC (though it might also be part of a general CPU state chunk).
            *   **(Optional) LLM State Chunk (`LLMs`)**: If recent LLM interactions need to be saved (e.g., conversation history snippets to maintain context across save/load), a dedicated chunk could store this.
        4.  **Output**: The VM writes these chunks to a file, forming the save game file. The `save` opcode would typically branch if the operation is successful.

    *   **Deserialization (`restore` opcode)**:
        1.  **File Selection**: The player or VM environment provides the save game file.
        2.  **Header Verification (`IFhd`)**: The VM reads the `IFhd` chunk from the save file. It compares the story ID, release number, and checksum with the currently loaded story file. If they don't match, the restore operation fails (and typically branches accordingly or halts with an error).
        3.  **Memory Restoration**:
            *   The VM reads the memory chunk (`CMem` or `UMem`).
            *   If compressed, it's decompressed.
            *   The data is written back into the Z-Machine's Dynamic Data Section, overwriting its current content.
        4.  **Stack Restoration (`Stks`)**: The call stack is cleared, and the saved stack frames are pushed back onto it. This restores local variables and return addresses.
        5.  **Register Restoration**: The saved Program Counter is loaded into the PC register. Other saved VM registers (like SP, frame pointers) are also restored.
        6.  **(Optional) LLM State Restoration (`LLMs`)**: Restore any saved LLM context.
        7.  **Resuming Execution**: Unlike Z-Spec 1.1 `restore` which returns a value, ZM2 `restore` typically does not return to the caller. Instead, after successfully restoring the state, the VM immediately resumes execution from the restored Program Counter, effectively continuing the game from the exact point it was saved. If the restore fails, it branches.

    *   **Considerations for 64-bit**:
        *   **File Size**: Uncompressed dynamic memory can be large, though less so than with 128-bit. Efficient compression for `CMem` is still crucial.
        *   **Atomicity**: Save/restore operations should ideally be atomic to prevent corrupted state if an error occurs midway. This is an interpreter implementation concern.
        *   **Endianness**: Ensure consistent endianness handling when writing/reading multi-word values (like 64-bit addresses or data) in the save file. The header structure already specifies Big Endian for internal memory; this should carry over to save files.

## 7. Player Interaction

-   **Input Process**:
    1.  **Raw Input Acquisition**: The Z-Machine interpreter captures the player's typed command from the input stream (e.g., console). This is typically done using an opcode like `sread` or `aread`. The raw text is stored in a designated buffer in Z-Machine memory (`text_buffer_addr`).
    2.  **Pre-processing (Optional but Recommended)**: Before sending the raw input to the LLM, the VM or game logic can perform several pre-processing steps:
        *   **Case Normalization**: Convert input to lowercase to ensure consistency, unless case sensitivity is specifically desired for certain commands or puzzles.
        *   **Whitespace Trimming**: Remove leading/trailing whitespace and normalize multiple internal spaces to single spaces.
        *   **Typo Correction (Basic)**: Implement a simple typo correction algorithm (e.g., Levenshtein distance against a common command vocabulary or game-specific terms) or use a more sophisticated local library if available to the interpreter. This can reduce LLM workload for obvious typos. *This step is optional and adds complexity; relying on the LLM's robustness to typos is also an option.*
        *   **Command History Expansion**: Allow players to use shortcuts like "again" (g) or references to previous commands/objects (e.g., "it", "him", "her", "them"). The VM can expand these locally before sending a more complete command to the LLM. For example, if the player types "take key" then "unlock door with it", "it" would be expanded to "key".
        *   **Profanity/Harmful Content Filtering (Basic Local Check)**: A local pre-filter for obviously problematic content might be implemented before an API call, though the primary responsibility for robust filtering often lies with the LLM service.
        *   **Concatenating with Turn Context**: The pre-processed player command is then combined with contextual information (see `context_data_addr` in `start_llm_parse` opcode details) to form the full prompt for the LLM. This context is crucial for the LLM to understand ambiguous commands.
    3.  **LLM Submission**: The (optionally pre-processed) input text along with the context data is used to initiate an LLM parsing task via the `start_llm_parse` opcode. The game receives a handle.
    4.  **LLM Response Handling**: The game logic will then need to periodically call `check_llm_status(handle)`.
        *   If status is "In Progress", the game can continue other tasks or enter a brief wait.
        *   If status is "Success", the game calls `get_llm_result(handle, result_buffer_addr)` to confirm. The structured response from the LLM (e.g., JSON) is now available in `result_buffer_addr`. Game logic then interprets this structure to take actions.
        *   If status indicates an error (Failed, LLM Processing Error, etc.), the game might:
            *   Print a generic "I didn't understand that." or "There was a problem with the LLM." message.
            *   Use a fallback traditional Z-Machine dictionary-based parser on the raw input.
            *   Query the player for clarification.

-   **Output Formatting and Presentation**:
    The Z-Machine Mark 2 must clearly distinguish between text generated directly by the game's internal logic (VM responses) and text generated by the LLM. This can be managed through conventions in game programming and interpreter capabilities.

    1.  **Direct VM Responses**:
        *   **Source**: These are texts embedded directly in the story file's Z-code (e.g., using `print` or `print_addr` opcodes) or constructed by game logic (e.g., "You take the brass key."). They are typically for:
            *   Standard messages: "Taken.", "Dropped.", "You can't go that way."
            *   Object short names (`print_obj`).
            *   Scores, turn counts, status lines.
            *   Fixed parts of room descriptions (if not fully LLM-generated).
            *   Debug messages or specific formatted outputs.
        *   **Formatting**: Standard Z-Machine text formatting applies (ZSCII encoding, potential for font styles if supported by the interpreter). Output is typically sent to the main game window. The game author controls this text directly.

    2.  **LLM-Generated Text**:
        *   **Source**: Generated by the LLM, initiated via the `start_llm_generate` opcode in response to a prompt from the game logic, and retrieved via `check_llm_status` and `get_llm_result`. This is used for:
            *   Dynamic room descriptions.
            *   Detailed object descriptions.
            *   NPC dialogue.
            *   Narrative flavor text, responses to unusual actions, or creative elaborations.
        *   **Formatting and Processing (handled by VM before `get_llm_result` confirms success)**:
            *   **Conversion to Z-Encoding and Unicode Handling**: The LLM typically returns plain text (e.g., UTF-8). The VM processes this text before Z-code can use it, especially for LLM result buffers intended for display. The exact behavior is influenced by the `StrictZSCIICompatMode` flag (Flag bit 2 in `flags1` of the header).
                *   **If `StrictZSCIICompatMode` is OFF (0, default Unicode mode):**
                    *   The VM converts the LLM's UTF-8 string into a sequence of 32-bit Unicode code points within the designated result buffer.
                    *   The Z-code routine responsible for printing this buffer would then iterate through these code points, using `print_unicode(char_code)` for each character. Standard Z-Machine string opcodes (like `print`) would not be directly applicable to such a buffer of raw Unicode code points.
                *   **If `StrictZSCIICompatMode` is ON (1):**
                    *   The VM will convert the LLM's UTF-8 text, attempting to transliterate or substitute all characters to their closest ZSCII equivalents. For characters with no reasonable ZSCII single-character substitute, a placeholder character '?' will be used.
                    *   The resulting text in the LLM result buffer will be a pure ZSCII string, suitable for direct use with standard Z-Machine output opcodes like `print` or `print_addr`.
                    *   In this mode, if Z-code calls `print_unicode(char_code)` where `char_code` is outside the ZSCII range, the VM will itself attempt to print the ZSCII substitute or the '?' placeholder instead of rendering the direct Unicode character.
            *   **Content Moderation/Filtering**: Before making the text available via `get_llm_result` (regardless of mode), the VM or game logic should ideally apply another layer of filtering to the LLM's output.
            *   **Stylistic Consistency**: The game author should prompt the LLM to maintain a consistent tone and style. The `creativity_level` parameter in `start_llm_generate` can help.
            *   **Attribution/Distinction (Optional)**: The interpreter or game could optionally prepend a subtle indicator for LLM-generated text if desired for transparency (e.g., a slightly different text color, a small icon, or a style like italics, if the output stream supports it). However, the goal is often seamless integration.
            *   **Line Wrapping and Paging**: The VM's standard output routines will handle line wrapping according to the current window width. If the LLM text is very long, the game might need to explicitly break it into manageable chunks or use "more..." prompts.
        *   **Display**: The processed, Z-encoded text from the LLM is then printed to the game window using standard Z-Machine output opcodes.

    3.  **Combining Outputs**:
        *   Game turns often involve a mix of VM-direct and LLM-generated text. For example:
            *   Player: `> look at table`
            *   VM (direct): `(First, checking to see if you can see the table...)` (internal thought or debug if verbose mode is on)
            *   Game Logic: Calls `start_llm_generate` with a prompt for the table description, gets a handle.
            *   Game Logic: (Optionally, does other quick tasks or enters a wait loop) Periodically calls `check_llm_status(handle)`.
            *   VM (internally): Once LLM responds and `check_llm_status` returns success, `get_llm_result` is called. The description is now in the buffer.
            *   Game Logic: Prints the LLM-generated description from the buffer: `"The old wooden table is covered in a fine layer of dust. Scratches and a few dark stains mar its surface, hinting at years of use. A half-open drawer on one side suggests something might be inside."`
            *   VM (direct, if a specific game mechanic is tied to it): `You notice a small, almost invisible inscription near the leg.`
        *   The game author orchestrates this sequence of calls to `print`, LLM opcodes, etc., to produce the final output for the turn.

## 8. Workflow (Game Loop)

The Z-Machine Mark 2 operates on a continuous game loop, processing player inputs, updating the game state, and generating outputs. This loop integrates traditional Z-Machine operations with LLM calls for enhanced interactivity.

**A. Initialization Phase (Executed Once at Game Start):**

1.  **VM Interpreter Startup**: The platform-specific Z-Machine Mark 2 interpreter is launched.
2.  **Story File Loading**:
    *   **VM Action**: The interpreter prompts the user for a story file or loads a default one.
    *   **Interaction (Memory)**: The VM reads the story file. It parses the **Header Section** to determine memory map (addresses and lengths of code, static data, dynamic data), story ID, version, checksum, initial Program Counter (PC), etc.
    *   **Interaction (Memory)**: The VM loads the **Code Section** and **Static Data Section** from the story file into their designated memory locations.
    *   **Interaction (Memory)**: The VM initializes the **Dynamic Data Section**. This includes setting up the initial state of global variables (copied from static data), preparing the object table with initial attributes and properties, and initializing the game stack.
3.  **Initial Game State Setup**:
    *   **VM Action**: The Program Counter (PC) is set to the game's starting routine address (from the header). Stack pointer and other necessary CPU registers are initialized.
    *   **Interaction (Memory/CPU)**: VM is ready to execute the first game instruction.
4.  **Initial Output (Optional)**:
    *   **VM Action**: The game may execute initial Z-code routines to print a welcome message, title screen, or the first room description. This might involve `print`, `print_addr`, or use the asynchronous LLM opcodes (`start_llm_generate`, `check_llm_status`, `get_llm_result`) if the initial description is dynamic.
    *   **Interaction (I/O, LLM, Memory)**: Output is sent to the player. LLM operations are initiated and managed asynchronously.

**B. Main Game Loop (Repeats Each Turn):**

1.  **Display Prompt & Read Player Input**:
    *   **VM Action**: The VM displays the input prompt (e.g., "> "). It then executes an input opcode (e.g., `sread` or `aread`).
    *   **Interaction (I/O)**: The interpreter waits for player to type a command and press Enter. The raw command string is read from the input stream.
    *   **Interaction (Memory)**: The raw command is stored in the `text_buffer_addr` in the Dynamic Data Section. Any specified parse buffer (`parse_buffer_addr`) is also prepared or cleared.

2.  **Input Pre-processing & Asynchronous LLM Parse Initiation**:
    *   **VM Action**: Game logic (Z-code) may perform pre-processing on the raw input text (case normalization, typo correction, command history expansion as detailed in Section 6).
    *   **VM Action**: The game logic formulates context data (player location, visible objects, etc.) to assist the LLM. This can be done through Z-code string operations or by using the `get_context_as_json` opcode.
    *   **Interaction (Memory)**: Pre-processed text and context data are prepared in memory buffers.
    *   **VM Action**: The `start_llm_parse` opcode is executed.
        *   **Interaction (Memory)**: The VM records the request, associates it with a new handle, and stores this handle in the designated result variable. No immediate HTTP call is necessarily made here; the VM might queue it.
    *   **Flowchart: Asynchronous LLM Parse Interaction**
        ```
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        | Z-Code: Prepare input &   |----->| VM: start_llm_parse             |----->| Z-Code: Stores handle, can perform   |
        | context data in memory    |      | (input_*, context_*, result_*)  |      | other operations / wait              |
        |                           |      | (Returns handle)                |      |                                      |
        +---------------------------+      +---------------------------------+      +--------------------------------------+
                                                                                              |
                                                                                              V
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        | Z-Code: Loop:             |----->| VM: check_llm_status(handle)    |----->| Z-Code: Checks status_code.          |
        | Calls check_llm_status    |      | (Manages HTTP to LLM if needed, |      | If 0 (In Progress), loop/wait.       |
        |                           |      | updates internal status)        |      | If 1 (Success), proceed to get.    |
        |                           |      | (Returns status_code)           |      | If error (2,3,4,5), handle error.    |
        +---------------------------+      +---------------------------------+      +--------------------------------------+
                                                                                              | (If status_code == 1)
                                                                                              V
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        | Z-Code: Process LLM       |<-----| VM: get_llm_result(handle,      |<-----| Z-Code: Calls get_llm_result         |
        | response from             |      | result_buffer_addr)             |      | to confirm data.                     |
        | result_buffer_addr        |      | (Confirms data in buffer)       |      |                                      |
        | or handle error           |      | (Returns status_code)           |      |                                      |
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        ```
    *   **VM Action (Error/Fallback Handling for Parsing, based on `check_llm_status` and `get_llm_result`)**:
        *   If `check_llm_status` returns `1` (Success) and `get_llm_result` returns `0` (Success): The game logic uses the structured data from `result_buffer_addr`.
        *   If `check_llm_status` returns error codes (2, 3, 4, 5) or `get_llm_result` returns error codes:
            *   The game logic MAY attempt a traditional Z-Machine dictionary-based parse on the original input (using `read` with dictionary, `tokenise`).
            *   The game logic MAY print an error message (e.g., "I didn't understand that," or "LLM unavailable.") or ask for clarification.

3.  **Execute Action & Update Game State**:
    *   **VM Action**: Based on the (successfully) parsed action (either from LLM or traditional parser), the VM executes the corresponding game logic routines (Z-code).
    *   **Interaction (CPU, Memory)**: The PC moves through instructions in the Code Section. Opcodes read from and write to global variables, object properties/attributes, the stack, and other parts of the Dynamic Data Section. This is where the game world changes.
        *   Examples: `set_attr` to mark a key as taken, `put_prop` to change an NPC's state, `insert_obj` to move the player to a new room.
    *   **Interaction (Memory)**: The game state (variables, object states, player location, etc.) is updated in the Dynamic Data Section.

4.  **Generate Text Output (Potentially Asynchronous)**:
    *   **VM Action**: Game logic determines what text to display. This can be a combination of:
        *   Directly printing pre-defined strings (e.g., "You open the door.").
        *   Printing object names or properties.
        *   Initiating dynamic text generation using `start_llm_generate`.
    *   **Flowchart: Asynchronous LLM Generate Interaction (Similar to Asynchronous Parse)**
        ```
        +---------------------------+      +-----------------------------------+      +--------------------------------------+
        | Z-Code: Prepare prompt &  |----->| VM: start_llm_generate            |----->| Z-Code: Stores handle, can perform   |
        | context data in memory    |      | (prompt_*, context_*, result_*,etc)|      | other operations / wait              |
        |                           |      | (Returns handle)                  |      |                                      |
        +---------------------------+      +-----------------------------------+      +--------------------------------------+
                                                                                                |
                                                                                                V
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        | Z-Code: Loop:             |----->| VM: check_llm_status(handle)    |----->| Z-Code: Checks status_code.          |
        | Calls check_llm_status    |      | (Manages HTTP to LLM if needed, |      | If 0 (In Progress), loop/wait.       |
        |                           |      | updates internal status)        |      | If 1 (Success), proceed to get.    |
        |                           |      | (Returns status_code)           |      | If error (2,3,4,5), handle error.    |
        +---------------------------+      +---------------------------------+      +--------------------------------------+
                                                                                                | (If status_code == 1)
                                                                                                V
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        | Z-Code: Process LLM text  |<-----| VM: get_llm_result(handle,      |<-----| Z-Code: Calls get_llm_result         |
        | from result_buffer_addr   |      | result_buffer_addr)             |      | to confirm data.                     |
        | (Z-encode, filter), print |      | (Confirms data in buffer)       |      |                                      |
        | or handle error           |      | (Returns status_code)           |      |                                      |
        +---------------------------+      +---------------------------------+      +--------------------------------------+
        ```
    *   **Interaction (Memory)**: If using LLM generation, the VM handles writing the Z-encoded and filtered text into `result_buffer_addr` upon successful completion.
    *   **Interaction (I/O)**: All output text (VM-direct and retrieved LLM-generated) is sent to the player's screen, formatted according to Z-Machine rules.

5.  **Check for Game End Conditions**:
    *   **VM Action**: Game logic checks if any end-game conditions have been met (e.g., player death, puzzle solved, `quit` command).
    *   **Interaction (Memory)**: Reads variables or game state flags.
    *   If game ends: Print final message, score, and halt or offer restart. Loop terminates.
    *   If game continues: Increment turn counter (if used), run any end-of-turn daemons/rules.

6.  **Loop**: Return to Step B.1.

-   **Error Handling within the Loop**:
    -   **LLM Errors**: Failures in LLM communication or content generation are handled by game logic based on `status_code` from `check_llm_status` and `get_llm_result`. This might involve falling back to simpler responses, traditional parsing, or informing the player of an issue.
    -   **VM Errors**: Invalid opcodes, memory access violations, stack overflows, etc., will typically halt the VM with an error message (see Section 4 on Instruction Set error handling). These are usually indicative of bugs in the story file Z-code or the interpreter itself.
    -   **Input Errors**: The game should be robust to nonsensical player input, relying on the LLM's parsing capabilities or fallback mechanisms.
-   **Asynchronous Operations (Consideration for `aread`)**:
    -   If `aread` (asynchronous read with timeout) is used, the loop structure might be more complex, involving event checking for input arrival or timeout events. The fundamental sequence of parse-execute-output for a command still applies once input is received.

## 9. Advantages

-   **Enhanced Interactivity**: More natural and flexible player inputs.
-   **Dynamic Text Generation**: Varied and engaging descriptions.
-   **Future-Proofing**: 64-bit architecture provides vast scalability for the foreseeable future.
-   **Portability**: Runs on any platform with a compatible interpreter, plus AI features.

## 10. Challenges and Considerations

-   **LLM Reliability**: Potential for inconsistent outputs; requires safeguards.
-   **Performance**: API calls may introduce latency.
-   **Training and Fine-Tuning**: LLM needs domain-specific training.
-   **Complexity**: While 64-bit is a significant step up, it aligns well with modern hardware, making it less complex to implement than 128-bit.

## 11. Implementation Notes

-   **Choosing an LLM**: Use a pre-trained model from Hugging Face, fine-tuned on interactive fiction data.
-   **API Integration**: Leverage Hugging Face's Inference API.
-   **Game Development Tools**: Extend tools like Inform for 64-bit operations and LLM integration.
-   **Testing**: Test with existing games for compatibility and performance.

## 12. Building the VM

This section outlines considerations for developing a Z-Machine Mark 2 Virtual Machine (interpreter).

**12.1. Development Environment Setup**

*   **Programming Language**:
    *   **Recommended**: Rust, C++, Go. These languages offer good performance, memory safety (especially Rust), and control over low-level operations suitable for VM development.
    *   **Alternatives**: Python (with C extensions like CFFI for performance-critical parts), Java, C#. Slower for core emulation but potentially faster for UI and tooling.
*   **Core Libraries/Modules**:
    *   **Memory Management**: Custom allocators might be needed if dealing with extremely large dynamic memory, but standard library features for byte array/vector manipulation are a starting point. For 64-bit numbers, most modern languages provide native `u64`/`i64` support. If not, a BigInt library might be needed.
    *   **File I/O**: Standard libraries for reading story files and writing save game files.
    *   **User Interface (I/O)**:
        *   Console: Libraries like `ncurses` (C/C++), `termion`/`crossterm` (Rust), or standard input/output for basic text interaction.
        *   GUI: Optional, but frameworks like Qt (C++), GTK (C/various), Electron (JS/web tech), or language-specific UI libraries (e.g., Java Swing/FX, Python Tkinter/PyQt) could be used for a richer experience.
    *   **Networking (for LLM Integration)**:
        *   HTTP Client: Libraries like `libcurl` (C/C++), `reqwest` (Rust), `requests` (Python), Apache HttpClient (Java) to make API calls to Hugging Face or other LLM providers.
        *   JSON Parsing: Libraries like `serde_json` (Rust), `nlohmann/json` (C++), `json.hpp` (C++), `Jackson`/`Gson` (Java), `json` (Python) to construct and parse JSON payloads for LLM communication.
    *   **BigInt Arithmetic**: Generally not needed for 64-bit operations as native types suffice.
*   **Development Tools**:
    *   **Compiler/Interpreter**: Specific to the chosen language (e.g., `rustc`, `gcc`/`clang`, `go build`, Python interpreter).
    *   **Build System**: `cargo` (Rust), `cmake`/`make` (C++), `go build` system, `pip`/`poetry` (Python).
    *   **Debugger**: `gdb`, `lldb`, or IDE-integrated debuggers.
    *   **Version Control**: Git.
    *   **Testing Framework**: `Criterion`/`Bencher` (Rust, for benchmarking), Google Test (C++), standard library testing tools (Go, Python).

**12.2. Phased Approach to Building the VM**

1.  **Phase 1: Core Z-Machine CPU and Memory Model (No I/O, No LLM)**
    *   **Memory Implementation**: Implement the 64-bit memory model (Header, Code, Static, Dynamic sections). Ability to load a story file's header, code, and static data into memory. Handle byte/word addressing (64-bit words). Implement Big Endian for all multi-byte values.
    *   **CPU Skeleton**: Implement the main execution loop (fetch-decode-execute). Program Counter (PC), Stack Pointer (SP).
    *   **Basic Opcodes**: Implement a small subset of essential opcodes:
        *   Stack operations: `push`, `pop`.
        *   Control flow: `jump`, `jz`, `call` (without argument handling initially), `ret`.
        *   Arithmetic: `add`, `sub` (on 64-bit values).
        *   Variable access: `store` (global/local), `load` (global/local).
        *   `nop`.
    *   **Story File Loader**: Basic loader for story file structure (reading header, sections).

2.  **Phase 2: Standard Opcode Implementation & Basic I/O**
    *   **Expand Opcodes**: Systematically implement all standard Z-Machine opcodes (0OP, 1OP, 2OP, VAROP) as defined in this specification and any accompanying opcode documents, adapting them for 64-bit operands and addresses.
    *   **Text Processing**: Implement ZSCII decoding, abbreviation handling, and dictionary lookup for traditional parsing.
    *   **Basic Console I/O**: Implement `print`, `print_addr`, `newline`, `sread` (basic line input without LLM for now), `aread` (if implementing timeouts).
    *   **Object Model**: Implement `get_prop`, `put_prop`, `get_parent`, `get_child`, `get_sibling`, `set_attr`, `clear_attr`, `test_attr`, `insert_obj`, `remove_obj`.
    *   **Save/Restore (Quetzal Adaptation)**: Implement `save` and `restore` opcodes using the 64-bit adapted Quetzal format. This is a significant sub-project.

3.  **Phase 3: Advanced Z-Machine Features**
    *   **Advanced I/O**: Implement features like text styles, color, sound (if specified).
    *   **Unicode Support**: Implement `print_unicode`, `check_unicode`.
    *   **Full Z-Machine v1-v8 Compatibility (Conceptual Mapping)**: Ensure conceptual behaviors from earlier Z-machine versions are correctly mapped or extended where appropriate for the 64-bit architecture.

4.  **Phase 4: LLM Integration (Asynchronous)**
    *   **Networking Subsystem**: Integrate an asynchronous HTTP client and JSON parsing libraries. The VM will need to manage a queue of active LLM requests and their states.
    *   **LLM Opcodes**: Implement `start_llm_parse`, `start_llm_generate`, `check_llm_status`, and `get_llm_result`.
        *   `start_llm_*`: These opcodes will create a request handle and store the necessary parameters. They might initiate the network request if the system is designed to send immediately, or queue it.
        *   `check_llm_status`: This opcode will check the state of the request associated with the handle. If it's the first check or the request is pending, it might trigger the actual HTTP communication or check on an ongoing one. It updates the internal state of the request based on LLM response or errors.
        *   `get_llm_result`: This opcode finalizes a successful request, ensuring the Z-code acknowledges the result is ready. The VM would have already placed the data in the result buffer when `check_llm_status` first determined success.
        *   Securely manage API keys (not hardcoded, configurable by the user of the VM).
        *   Construct JSON payloads as per LLM API requirements.
        *   Handle API responses asynchronously, including errors, timeouts, and status codes, and map them to the status codes for `check_llm_status`.
        *   Convert LLM text output to Z-encoded strings for game use before `check_llm_status` reports success.
    *   **Contextual Data Assembly**: Develop routines (either in VM or example Z-code) for assembling the `context_data_addr` payload for `start_llm_*` calls.

5.  **Phase 5: Optimization and Refinement**
    *   **Performance Profiling**: Identify and optimize bottlenecks in the execution loop or specific opcodes.
    *   **Memory Optimization**: Reduce interpreter memory footprint if necessary.
    *   **Error Handling**: Robust error reporting for both VM internal errors and game Z-code errors.
    *   **User Interface Enhancements**: Improve UI/UX if a graphical interface is used.

**12.3. Testing Guidelines**

*   **Unit Testing (Per Component/Opcode)**:
    *   **CPU/Opcodes**: For each opcode, write specific unit tests with known inputs and expected outputs/state changes.
        *   Example: For `add G0, 5 -> L0`, set G0, execute, check L0 and flags.
        *   Test edge cases: zero, negative numbers (if applicable), max values, overflow (if defined behavior).
    *   **Memory Management**: Test allocation, reads, writes, boundary conditions, endianness.
    *   **Story File Loading**: Test loading valid and corrupted story files (header checks, checksums).
    *   **Save/Restore**: Create known game states, save, restore, and verify that the state (memory, stack, PC) is identical. Test compatibility checks.
    *   **LLM Opcodes (Asynchronous)**:
        *   Mock LLM Service: Create a mock HTTP server that can simulate delayed responses, different success/error scenarios for asynchronous calls. Test the VM's ability to manage handles, correctly poll with `check_llm_status`, and retrieve results with `get_llm_result`.
        *   Test parameter passing for `start_llm_*` opcodes.
        *   Thoroughly test all defined status codes for `check_llm_status` and `get_llm_result`.
        *   Test concurrent LLM requests if the VM design supports them.
*   **Integration Testing (Modules Working Together)**:
    *   **Core Logic**: Test sequences of opcodes that perform a common game task.
    *   **I/O and Game Logic**: Test that `sread` correctly captures input, game logic processes it (including potential asynchronous LLM calls for parsing), and `print` displays the correct output.
    *   **LLM and Game Logic (Asynchronous)**:
        *   Use a simple story file that uses the `start_llm_parse`, `check_llm_status`, and `get_llm_result` sequence. Provide controlled input and verify that the parsed output (from a mock or real LLM) is correctly interpreted by the game logic after the necessary polling.
        *   Use a simple story file for asynchronous LLM generation. Verify that the game can wait for and then correctly display the generated text.
*   **System/Acceptance Testing (Using Test Story Files)**:
    *   **Standard Test Suites**: Adapt existing Z-Machine test suites like "praxix.z5" or "cमेट" (if possible, by recompiling their source for ZM2 or creating analogous tests) to verify overall compliance. This will be challenging due to the 64-bit nature and new opcodes.
    *   **Custom Test Stories**: Develop small story files specifically designed to test ZM2 features:
        *   A story that heavily uses all implemented opcodes.
        *   A story that tests LLM parsing for various command structures.
        *   A story that tests LLM generation for different contexts (room descriptions, NPC dialogue).
        *   A story that tests save/load under various conditions.
    *   **Real-World Testing**: If example games are developed, play them extensively.
*   **Benchmarking**:
    *   Measure opcode execution speed.
    *   Measure LLM API call latency and its impact on gameplay.
    *   Measure save/load times.
*   **Debugging Aids**:
    *   Implement verbose logging options in the VM.
    *   Develop a simple Z-Machine debugger (memory view, stack view, PC tracing, breakpoints).

## 13. Key Citations

-   Z-Machine Standards Document Overview
-   Large Language Models and Conversational User Interfaces for Interactive Fiction
-   LampGPT: LLM-Enhanced IF Experiences
-   AI Plays Interactive Fiction GitHub Repository
-   Why can't the parser just be an LLM?
-   Creating a Text Adventure Game with ChatGPT
-   How to build a Large Language Model (LLM) to Generate Creative Writing
-   Best realistic story telling LLM?
-   Overview of Z-machine architecture
