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
| 148            | 8            | `property_defaults_table_start`| Byte address of the property defaults table within the Static Data Section. 0 if not used or if defaults are always 0. |
| 156            | 868          | `reserved`                     | Reserved for future expansion. Must be initialized to zero.                                                |
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
        *   **Property Defaults Table**: An array of 64-bit values, indexed by (Property ID - 1). Used by `get_prop` when a property is not found on an object. Pointed to by `property_defaults_table_start` in the Header. If `property_defaults_table_start` is 0, all default property values are assumed to be 0.
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

This list provides a few examples of standard opcodes adapted for ZM2, reflecting the systematic numbering scheme. For a more comprehensive list and detailed specifications of all opcodes (including behavior changes from Z-Spec 1.1 for 64-bit operations), refer to Section 4.A "Detailed Opcode Specification" and the supplementary `ZM2_Opcodes.md` document (once populated). All operations are on 64-bit values unless specified.

*   **0OP (No Operands):** Example Range `0x0000 - 0x00FF`
    *   `rtrue` (`0x0000`): Return true (1) from the current routine.
    *   `rfalse` (`0x0001`): Return false (0) from the current routine.
    *   `print` (`0x0002`): Prints the Z-encoded string at the current PC, then advances PC.
    *   `newline` (`0x0003`): Prints a newline character.
    *   `save` (`0x0004`): (Branch) Saves the game state. Branches if successful.
    *   `restore` (`0x0005`): (Branch) Restores the game state. Branches if successful.
    *   `quit` (`0x0006`): Terminates the game.
    *   `nop` (`0x0007`): No operation.

*   **1OP (One Operand):** Example Range `0x0100 - 0x01FF`
    *   `jz` (`0x0100`): Jump if value is zero. Operand is the value to test. Branch data follows.
    *   `get_sibling` (`0x0101`): Stores sibling object ID.
    *   `get_child` (`0x0102`): Stores child object ID.
    *   `get_parent` (`0x0103`): Stores parent object ID.
    *   `get_prop_len` (`0x0104`): Stores length of property data.
    *   `inc` (`0x0105`): Increments variable.
    *   `dec` (`0x0106`): Decrements variable.
    *   `print_addr` (`0x0107`): Prints Z-encoded string at the given byte address.
    *   `print_obj` (`0x0108`): Prints short name of the object.
    *   `ret` (`0x0109`): Return value from current routine.
    *   `pop` (`0x010A`): Pops value from stack into variable.

*   **2OP (Two Operands):** Example Range `0x0200 - 0x02FF`
    *   `je` (`0x0200`): Jump if value1 equals value2. Branch data follows.
    *   `jl` (`0x0201`): Jump if value1 < value2. Branch data follows.
    *   `jg` (`0x0202`): Jump if value1 > value2. Branch data follows.
    *   `add` (`0x0203`): a + b.
    *   `sub` (`0x0204`): a - b.
    *   `mul` (`0x0205`): a * b.
    *   `div` (`0x0206`): a / b (integer division).
    *   `mod` (`0x0207`): a % b.
    *   `loadw` (`0x0208`): Reads word from array.
    *   `get_prop` (`0x020A`): Reads property value.

*   **VAROP (Variable Number of Operands):** Example Range `0x0300 - 0x03FF`
    *   `call` (`0x0300`): Calls a routine. `routine_paddr` is a packed address.
    *   `store` (`0x0301`): Stores value in variable (ZM2 version).
    *   `print_unicode` (`0x0302`): Prints a Unicode character.
    *   `check_unicode` (`0x0303`): Checks if current output stream supports the Unicode character.
    *   `sread` (`0x0304`): Reads player input.

*   **EXT (Extended for ZM2):** Example Range `0xEE00 - 0xEFFF`
    *   These opcodes are specific to Z-Machine Mark 2, primarily for LLM integration and other advanced features. See Section 4.A.3.e onwards for examples like `start_llm_parse` (`0xEE00`).

**VM Utility Opcodes:**

Note: `get_context_as_json` is an EXT opcode and is detailed in Section 4.A.3.
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

**Usage Note**: Opcodes that take a `RoutinePADDR` (e.g., `call`, `timeout_routine_paddr` in `aread`, `sound_finished_routine_paddr` in `sound_effect`) expect the packed address to resolve to the beginning of executable code for a routine. Opcodes that take a `StringPADDR` (e.g., `print_paddr`) expect it to resolve to the beginning of a Z-encoded string.

### 4.A.1 Preamble/Introduction to Opcode Specification

For clarity and brevity in this main specification, detailed definitions will be provided for new Z-Machine Mark 2 specific opcodes (primarily in the EXT form, see Section 4.A.3.e onwards) and for standard Z-Machine opcodes whose behavior is significantly altered by the 64-bit architecture beyond simple operand size changes. For the remaining standard opcodes adapted from the Z-Machine Standard 1.1 (typically covering versions 3 to 5/6, as appropriate for a given opcode's origin), their fundamental logic remains the same as described in that standard. Implementers should refer to the Z-Machine Standard Document 1.0 or 1.1 for this original logic, with the critical understanding that for Z-Machine Mark 2, all relevant operands, addresses, memory pointers, and intermediate arithmetic calculations are expanded to 64-bits. Packed Addresses (PADDRs), where used for routine calls or string references, must be handled as described in Section 4.A.0. A comprehensive list of all opcodes, their assigned ZM2 values, operand types, and mappings to Z-Machine Standard 1.1 will ideally be maintained in a separate document: `ZM2_Opcodes.md` (currently a placeholder: a full list is not yet populated).

All multi-byte values within instruction streams, including the opcode itself and any literal operands, are stored in **Big Endian** format, consistent with the overall memory model of the Z-Machine Mark 2.

**Systematic Opcode Numbering Scheme:**

The proposed systematic opcode numbering scheme for Z-Machine Mark 2 is as follows:
*   `0OP` (Zero Operands): Range `0x0000 - 0x00FF`
*   `1OP` (One Operand): Range `0x0100 - 0x01FF`
*   `2OP` (Two Operands): Range `0x0200 - 0x02FF`
*   `VAROP` (Variable Operands): Range `0x0300 - 0x03FF`
*   `EXT` (Extended Opcodes for ZM2): Range `0xEE00 - 0xEFFF`

*Note: The `Opcode Value` fields in the following example definitions (Section 4.A.3) adhere to this proposed systematic scheme. Final assignment of all opcode values will be done in the `ZM2_Opcodes.md` document.*

### 4.A.2 Opcode Definition Template

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
    *   `branch_data?`: Indicates if branch information follows the opcode and its operands. See Section 4.A.2.1 "Standard Branch Data Format" for details on this structure.
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

#### 4.A.2.1 Standard Branch Data Format

When an opcode indicates it uses `branch_data`, this data immediately follows the opcode and its main operands in the instruction stream. This data determines whether a branch is taken and where execution should jump to. The format is as follows:

1.  **Branch Byte 1 (BB1)**: The first byte of the branch data.
    *   **Bit 7 (Sense Bit)**: If this bit is 1, the branch occurs if the opcode's condition is true. If this bit is 0, the branch occurs if the opcode's condition is false. Many opcodes have an implicit condition (e.g., `jz` branches if zero, so its effective condition is "value is zero"; the sense bit should typically be 1 for `jz`).
    *   **Bit 6 (Offset Length Bit)**:
        *   If 0: The branch offset is a signed 8-bit value, contained in the next byte (Branch Byte 2).
        *   If 1: The branch offset is a signed 16-bit value, contained in the next two bytes (Branch Byte 2 and Branch Byte 3), stored in Big Endian order.
    *   **Bits 0-5**: These bits are part of the offset if Bit 6 is 1 (forming the top 6 bits of a 14-bit offset in original Z-Machine, but ZM2 simplifies this). For ZM2, if Bit 6 is 1 (16-bit offset), Bits 0-5 of BB1 are ignored. If Bit 6 is 0 (8-bit offset), Bits 0-5 of BB1 are also ignored. (This simplifies ZM2 by removing the 14-bit offset form, using either 8-bit or 16-bit offsets directly).

2.  **Branch Offset**:
    *   **If Offset Length Bit (BB1, Bit 6) is 0**:
        *   **Branch Byte 2 (BB2)**: This single byte contains a signed 8-bit offset value.
    *   **If Offset Length Bit (BB1, Bit 6) is 1**:
        *   **Branch Byte 2 (BB2)**: The high byte of a signed 16-bit offset.
        *   **Branch Byte 3 (BB3)**: The low byte of a signed 16-bit offset. (BB2 and BB3 form a Big Endian 16-bit value).

3.  **Branch Execution**:
    *   The opcode evaluates its condition (e.g., for `jz`, is the value zero?).
    *   The result of this condition (true or false) is compared with the Sense Bit (BB1, Bit 7).
    *   If they match (e.g., condition is true AND Sense Bit is 1, OR condition is false AND Sense Bit is 0), the branch is taken.
    *   **Offset Calculation**: The signed branch offset (8-bit or 16-bit) is calculated.
        *   An offset value of `0` means "return false (0) from the current routine."
        *   An offset value of `1` means "return true (1) from the current routine."
        *   Any other offset value is added to the address of the instruction *immediately following all parts of the current branching instruction* (i.e., opcode, its operands, and all its branch data bytes) to get the new Program Counter (PC).
    *   If the branch is not taken, execution continues with the instruction immediately following all parts of the current branching instruction.

This standardized format ensures all branching opcodes behave consistently regarding offset calculation and condition sensing.

### 4.A.3 Example Opcode Definitions

#### a. `rtrue` (0OP)

*   **`Mnemonic`**: `rtrue`
*   **`Opcode Value`**: `0x0000`
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
*   **`Opcode Value`**: `0x0100`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: 1 byte indicating the type of `value`.
        *   `0x00`: Large Constant (LC) - 8 bytes follow for `value`.
        *   `0x01`: Small Constant (SC) - 1 byte follows for `value`.
        *   `0x02`: Variable (VAR) - 1 byte follows (specifies stack/local/global for `value`).
    *   `value (LC/SC/VAR)`: The value to be tested.
*   **`Description`**: Jump if value is zero.
*   **`Operation Details`**:
    1.  Reads the `type_byte` following the opcode.
    2.  Based on `type_byte`, reads the `value` operand:
        *   If LC, reads 8 bytes.
        *   If SC, reads 1 byte and zero-extends to 64 bits.
        *   If VAR, reads 1 byte variable specifier, then fetches the 64-bit value from that variable.
    3.  If the fetched `value` is equal to 0, a branch is performed based on the subsequent branch data.
    4.  If the `value` is not 0, and the branch data's 'sense' bit indicates branching on false (which it typically would for `jz`), the branch is NOT taken, and execution continues with the instruction immediately following the branch data. If the sense bit indicates branching on true, a branch IS taken. (Standard `jz` implies branch on condition true, so the sense bit in branch data should align).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `value == 0`. The actual branching logic (offset calculation, condition sense) is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `type_byte` (not 0x00, 0x01, or 0x02).
    *   Invalid variable reference if `type_byte` indicates VAR and `value` specifies an invalid variable.

#### c. `add` (2OP)

*   **`Mnemonic`**: `add`
*   **`Opcode Value`**: `0x0200`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: 1 byte for `operand1`.
    *   `type_byte2 (Operand Type Specifier)`: 1 byte for `operand2`.
        *   Type specifier values: `0x00` (LC), `0x01` (SC), `0x02` (VAR).
    *   `operand1 (LC/SC/VAR)`: The first value.
    *   `operand2 (LC/SC/VAR)`: The second value.
    *   `store_variable_ref (Variable Reference)`: Variable to store the result.
*   **`Description`**: Adds `operand1` and `operand2`, stores the 64-bit result in `store_variable_ref`.
*   **`Operation Details`**:
    1.  Reads `type_byte1`. Based on it, reads `operand1`.
    2.  Reads `type_byte2`. Based on it, reads `operand2`.
    3.  Fetches actual values for `operand1` and `operand2` if they are VAR type. SC types are zero-extended to 64 bits.
    4.  Computes `result = (value1 + value2)` using 64-bit signed arithmetic. Wraps on overflow.
    5.  Stores `result` into the variable specified by `store_variable_ref`.
    6.  PC advances past this instruction and all its components (opcode, type bytes, operands, store_variable_ref).
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte1` or `type_byte2`.
    *   Invalid variable reference for `operand1`, `operand2` (if VAR type), or `store_variable_ref`.

#### d. `call` (VAROP)

*   **`Mnemonic`**: `call`
*   **`Opcode Value`**: `0x0300`
*   **`Form`**: `VAROP`
*   **`Operand Types`**: `routine_paddr (RoutinePADDR)`, `arg0 (Small Constant/Variable)`, ..., `argN (Small Constant/Variable)` (up to 7 args), `store_variable_ref (Variable Reference)`
    *   Operand types for `arg0` through `argN` are determined by type byte(s) following the opcode.
    *   The `routine_paddr` is the first operand read.
*   **`Description`**: Calls the routine at the packed address `routine_paddr`, passing up to 7 arguments. Stores the routine's return value into `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `routine_paddr` operand (which is a `RoutinePADDR` as defined in Section 4.A.0). Resolve it to an absolute byte address (`code_section_start + routine_paddr`) within the Code Section. This is the target routine address.
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
    *   If a routine is called with more arguments than it declares, the extra arguments are silently ignored by the called routine.
    *   If a routine is called with fewer arguments than declared, the unspecified local variables corresponding to the missing arguments are initialized to 0 by the VM when the stack frame is created.

---

*(The following EXT opcode definitions continue the "Example Opcode Definitions" numbering. EXT opcodes are specialized instructions for ZM2, often handling complex operations like LLM interaction or advanced I/O. Their operand structures are explicitly defined for each opcode. If an operand can accept multiple types (e.g., Large Constant or Variable), it will be preceded by its own Operand Type Specifier byte, similar to VAROP operands: `0x00` for LC, `0x01` for SC, `0x02` for VAR, `0x03` for PADDR.)*

#### e. `start_llm_parse` (EXT)

*   **`Mnemonic`**: `start_llm_parse`
*   **`Opcode Value`**: `0xEE00`
*   **`Form`**: `EXT`
*   **`Operand Types`**:
    *   `type_input (Operand Type Specifier)`: For `input_text_addr`.
    *   `input_text_addr (LC/VAR Address)`: Byte address of a null-terminated Z-encoded string for player input.
    *   `type_context (Operand Type Specifier)`: For `context_data_addr`.
    *   `context_data_addr (LC/VAR Address)`: Byte address of structured data (e.g., JSON string) for context. Can be 0.
    *   `type_result_buf (Operand Type Specifier)`: For `result_buffer_addr`.
    *   `result_buffer_addr (LC/VAR Address)`: Byte address of a buffer for the LLM's structured response.
    *   `type_max_len (Operand Type Specifier)`: For `max_result_len`.
    *   `max_result_len (LC/SC/VAR)`: Maximum number of bytes for `result_buffer_addr`.
    *   `store_handle_variable_ref (Variable Reference)`: Variable to store the 64-bit request handle.
*   **`Description`**: Initiates an asynchronous LLM parsing task for the string at `input_text_addr` using context from `context_data_addr`.
*   **`Operation Details`**:
    1.  Read all operands according to their specified types. Fetch values if VAR.
    2.  Validate addresses (`input_text_addr`, `result_buffer_addr` must be valid; `context_data_addr` can be 0).
    3.  Validate `max_result_len` (must be > 0).
    4.  Check `flags1.LLMParseEnable`. If OFF (0), store 0 (or `LLM_ERR_DISABLED`) into `store_handle_variable_ref` and return.
    5.  Generate a new, unique 64-bit non-zero handle for this LLM request.
    6.  The VM internally records the request: handle, addresses, max length, and type (parse).
    7.  The actual API call is managed by the VM, typically triggered/polled via `check_llm_status`.
*   **`Stores Result To`**: The new `handle` (or error code) is stored in `store_handle_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: An LLM parse request is queued.
*   **`Error Conditions`**: Invalid type bytes. Invalid addresses or `max_result_len`. `LLMParseEnable` is false. Failure to generate handle. Invalid `store_handle_variable_ref`.

#### f. `start_llm_generate` (EXT)

*   **`Mnemonic`**: `start_llm_generate`
*   **`Opcode Value`**: `0xEE01`
*   **`Form`**: `EXT`
*   **`Operand Types`**:
    *   `type_prompt (Operand Type Specifier)`: For `prompt_text_addr`.
    *   `prompt_text_addr (LC/VAR Address)`: Byte address of a null-terminated Z-encoded string for the LLM prompt.
    *   `type_context (Operand Type Specifier)`: For `context_data_addr`.
    *   `context_data_addr (LC/VAR Address)`: Byte address of structured context data. Can be 0.
    *   `type_result_buf (Operand Type Specifier)`: For `result_buffer_addr`.
    *   `result_buffer_addr (LC/VAR Address)`: Byte address of a buffer for the LLM's generated Z-encoded text.
    *   `type_max_len (Operand Type Specifier)`: For `max_result_len`.
    *   `max_result_len (LC/SC/VAR)`: Maximum bytes for `result_buffer_addr`.
    *   `type_creativity (Operand Type Specifier)`: For `creativity_level`.
    *   `creativity_level (LC/SC/VAR)`: Value (e.g., 0-100) for LLM temperature.
    *   `store_handle_variable_ref (Variable Reference)`: Variable to store the 64-bit request handle.
*   **`Description`**: Initiates an asynchronous LLM text generation task.
*   **`Operation Details`**:
    1.  Read all operands per types. Fetch VAR values.
    2.  Validate addresses and `max_result_len`.
    3.  Check `flags1.LLMGenerateEnable`. If OFF (0), store 0 (or `LLM_ERR_DISABLED`) into `store_handle_variable_ref` and return.
    4.  Generate a new, unique 64-bit non-zero handle.
    5.  VM records request details: handle, addresses, max length, creativity, type (generate).
    6.  Actual API call managed by VM, polled via `check_llm_status`.
*   **`Stores Result To`**: New `handle` (or error code) in `store_handle_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: An LLM generate request is queued.
*   **`Error Conditions`**: Invalid type bytes. Invalid addresses, `max_result_len`, or `creativity_level`. `LLMGenerateEnable` is false. Failure to generate handle. Invalid `store_handle_variable_ref`.

#### g. `check_llm_status` (EXT)

*   **`Mnemonic`**: `check_llm_status`
*   **`Opcode Value`**: `0xEE02`
*   **`Form`**: `EXT`
*   **`Operand Types`**:
    *   `type_handle (Operand Type Specifier)`: For `handle`.
    *   `handle (LC/SC/VAR)`: The 64-bit handle of the LLM request to check.
    *   `store_status_variable_ref (Variable Reference)`: Variable to store the status code.
*   **`Description`**: Polls the status of a pending LLM request.
*   **`Operation Details`**:
    1.  Read `handle` operand.
    2.  VM checks internal state of the request associated with `handle`. This may involve checking network connection, sending request if not yet sent, or processing an incoming response.
    3.  Status codes stored in `store_status_variable_ref`:
        *   `0`: In Progress.
        *   `1`: Success (result is ready in the designated `result_buffer_addr`).
        *   `2`: Failed (Network error, API error, timeout during HTTP).
        *   `3`: Invalid Handle.
        *   `4`: LLM Processing Error (LLM service returned an error).
        *   `5`: Result Buffer Too Small (detected by VM before/during writing).
*   **`Stores Result To`**: `status_code` in `store_status_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: May trigger network activity. Internal state of LLM request updated. If successful, result written to previously specified buffer.
*   **`Error Conditions`**: Invalid type byte for `handle`. Invalid `handle` value (if not caught by status 3). Invalid `store_status_variable_ref`.

#### h. `get_llm_result` (EXT)

*   **`Mnemonic`**: `get_llm_result`
*   **`Opcode Value`**: `0xEE03`
*   **`Form`**: `EXT`
*   **`Operand Types`**:
    *   `type_handle (Operand Type Specifier)`: For `handle`.
    *   `handle (LC/SC/VAR)`: The 64-bit handle of the completed LLM request.
    *   `store_status_variable_ref (Variable Reference)`: Variable to store a confirmation status.
*   **`Description`**: Confirms retrieval of a successful LLM operation's result. Called after `check_llm_status` returns 1 (Success).
*   **`Operation Details`**:
    1.  Read `handle` operand.
    2.  VM verifies that the request associated with `handle` is indeed in a 'Success' state and the result has been written to its buffer.
    3.  Status codes stored in `store_status_variable_ref`:
        *   `0`: Success (result is confirmed available in its buffer).
        *   `1`: Error Retrieving (e.g., handle was not in 'Success' state, or internal inconsistency).
        *   `2`: Invalid Handle.
    4.  This opcode primarily serves as a formal acknowledgment by the Z-code that it is proceeding with a successfully completed LLM operation. The VM would have already made the data available when `check_llm_status` first reported success.
*   **`Stores Result To`**: `status_code` in `store_status_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None, beyond formally consuming the result from the VM's perspective for that handle.
*   **`Error Conditions`**: Invalid type byte for `handle`. Invalid `handle` value. Invalid `store_status_variable_ref`.

#### i. `get_context_as_json` (EXT)

*   **`Mnemonic`**: `get_context_as_json`
*   **`Opcode Value`**: `0xEE04`
*   **`Form`**: `EXT`
*   **`Operand Types`**:
    *   `type_scope (Operand Type Specifier)`: For `object_scope_flag`.
    *   `object_scope_flag (LC/SC/VAR)`: Bitmask defining scope (player inv, location objects, globals, events).
    *   `type_depth (Operand Type Specifier)`: For `max_depth`.
    *   `max_depth (LC/SC/VAR)`: Max depth for object tree traversal.
    *   `type_output_buf (Operand Type Specifier)`: For `output_buffer_addr`.
    *   `output_buffer_addr (LC/VAR Address)`: Byte address for the JSON string.
    *   `type_max_len (Operand Type Specifier)`: For `max_output_len`.
    *   `max_output_len (LC/SC/VAR)`: Maximum bytes for `output_buffer_addr`.
    *   `store_status_variable_ref (Variable Reference)`: Variable to store the status code.
*   **`Description`**: Collects game state information and formats it as a JSON string in `output_buffer_addr`.
*   **`Operation Details`**:
    1.  Read all operands per types. Fetch VAR values.
    2.  Validate `output_buffer_addr` and `max_output_len`.
    3.  Based on `object_scope_flag` and `max_depth`, gather game state (see Section 4, VM Utility Opcodes for details on flags and JSON structure).
    4.  Construct JSON string. If its length > `max_output_len`, store status 1 (Buffer Too Small) and optionally write truncated JSON.
    5.  Else, write complete JSON to `output_buffer_addr` and store status 0 (Success).
*   **`Stores Result To`**: `status_code` (0=Success, 1=BufferSmall, 2=InvalidScope, 3=JSONError) in `store_status_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Memory at `output_buffer_addr` is overwritten.
*   **`Error Conditions`**: Invalid type bytes. Invalid addresses or `max_output_len`. Invalid `store_status_variable_ref`.

#### j. `aread` (EXT)

*   **`Mnemonic`**: `aread`
*   **`Opcode Value`**: `0xEE05`
*   **`Form`**: `EXT` (behaves like a VAROP in terms of argument structure)
*   **`Operand Types`**:
    *   `type_text_buf (Operand Type Specifier)`: For `text_buffer_addr`.
    *   `text_buffer_addr (LC/VAR Address)`: Buffer to store player's typed text. First byte must contain max chars.
    *   `type_parse_buf (Operand Type Specifier)`: For `parse_buffer_addr`.
    *   `parse_buffer_addr (LC/VAR Address)`: Optional buffer for dictionary parse (0 if not used). First byte max words.
    *   (Optional) `type_timeout (Operand Type Specifier)`: For `timeout_seconds`.
    *   (Optional) `timeout_seconds (LC/SC/VAR)`: Number of seconds for timeout (0 for no timeout).
    *   (Optional) `type_routine (Operand Type Specifier)`: For `timeout_routine_paddr`.
    *   (Optional) `timeout_routine_paddr (RoutinePADDR)`: Packed address of routine to call on timeout.
    *   `store_terminator_variable_ref (Variable Reference)`: Stores the terminating character code.
*   **`Description`**: Reads player input with optional timeout. Stores text, optionally parses, and returns terminator.
*   **`Operation Details`**:
    1.  Read `text_buffer_addr` and `parse_buffer_addr`. Fetch VARs.
    2.  Read optional `timeout_seconds` and `timeout_routine_paddr` if provided (determined by presence of type bytes).
    3.  Display input prompt.
    4.  If `timeout_seconds > 0`, start timer.
    5.  Wait for player input line or timeout.
    6.  If timeout occurs:
        *   Resolve `timeout_routine_paddr` (a `RoutinePADDR` as per Section 4.A.0) to an absolute address. Call the routine at this address (if `timeout_routine_paddr` was provided and is valid).
        *   Store 0 (or a specific timeout terminator code if defined for ZM2) in `store_terminator_variable_ref`.
        *   Text in `text_buffer_addr` might be empty or partial. Its length byte should be updated.
        *   Return.
    7.  If input received:
        *   Store characters read into `text_buffer_addr` (second byte stores count, then ZSCII text, null terminated if space).
        *   If `parse_buffer_addr` is non-zero, perform dictionary tokenization as per `sread`/`tokenise`.
        *   Store the terminating character's ZSCII code (e.g., newline, function key if those are terminators) in `store_terminator_variable_ref`.
*   **`Stores Result To`**: Terminating character code in `store_terminator_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Player I/O. Memory at buffers modified. Potential call to timeout routine.
*   **`Error Conditions`**: Invalid type bytes. Invalid addresses, PADDR. Buffer sizes too small.

#### g. `sread` (VAROP)

*   **`Mnemonic`**: `sread`
*   **`Opcode Value`**: `0x0301`
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

#### h. `save` (0OP)

*   **`Mnemonic`**: `save`
*   **`Opcode Value`**: `0x0001`
*   **`Form`**: `0OP`
*   **`Operand Types`**: Branch data follows the opcode.
*   **`Description`**: Attempts to save the current game state. Branches if the save operation is successful.
*   **`Operation Details`**:
    1.  The VM checks if save/load operations are enabled via the `SaveLoadEnable` flag in `flags1` of the Header. If not enabled, the operation fails.
    2.  The VM attempts to serialize the current game state. This includes:
        *   The entire Dynamic Data Section (global variables, object states, heap, stack).
        *   Key VM registers: Program Counter (PC), Stack Pointer (SP), current routine's frame pointer.
    3.  The serialization format should follow the ZM2 Quetzal-like adaptation (see Section 6: Game State Management).
    4.  The VM prompts the player for a filename or uses a platform-specific mechanism to choose a save file location.
    5.  The serialized data is written to the chosen file.
    6.  If the save operation is successful (file written and verified correctly):
        *   A branch is performed according to the branch data.
    7.  If the save operation fails for any reason (e.g., `SaveLoadEnable` is false, disk error, user cancellation, serialization error):
        *   Execution continues with the instruction immediately following the branch data (i.e., the branch is not taken).
    8.  The PC is updated according to whether the branch was taken or not.
*   **`Stores Result To`**: Does not store a result directly. The success/failure is indicated by whether the branch is taken.
*   **`Branches If`**: The save operation completes successfully. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1). The 'sense' bit in the branch data should be set to branch on true (success).
*   **`Side Effects`**: A save file containing the game state is created or overwritten on the storage medium. Player I/O for filename prompt.
*   **`Error Conditions`**:
    *   `SaveLoadEnable` flag in `flags1` is false.
    *   Disk full or write error.
    *   User cancellation during file selection.
    *   Internal error during state serialization.

#### i. `restore` (0OP)

*   **`Mnemonic`**: `restore`
*   **`Opcode Value`**: `0x0002`
*   **`Form`**: `0OP`
*   **`Operand Types`**: Branch data follows the opcode. (Branch is taken if restore *fails*).
*   **`Description`**: Attempts to restore a previously saved game state. If successful, execution does not return to the caller but resumes from the PC in the saved state. If restoration fails, it branches.
*   **`Operation Details`**:
    1.  The VM checks if save/load operations are enabled via the `SaveLoadEnable` flag in `flags1` of the Header. If not enabled, the operation fails.
    2.  The VM prompts the player for a filename or uses a platform-specific mechanism to select a save file.
    3.  The VM attempts to read and deserialize the game state from the chosen file.
    4.  **Header Verification (`IFhd`)**: The `IFhd` chunk is read. Story ID, release number, and checksum are compared with the currently loaded story. If they don't match, the restore fails.
    5.  **Memory and Stack Restoration**: If headers match, the Dynamic Data Section and call stack are restored from the save file data.
    6.  **Register Restoration**: The PC, SP, and other relevant VM registers are restored.
    7.  If the entire restore operation is successful:
        *   The VM **does not** return to the instruction following `restore`. Instead, it immediately begins execution at the restored Program Counter.
    8.  If the restore operation fails for any reason (e.g., `SaveLoadEnable` is false, file not found, corrupted file, header mismatch, deserialization error):
        *   A branch is performed according to the branch data. (Note: This is the inverse of `save`; `restore` branches on failure).
*   **`Stores Result To`**: Does not store a result. In ZM2, `restore` does not return to the caller if successful.
*   **`Branches If`**: The restore operation *fails*. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1). The 'sense' bit in the branch data should be set to branch on true (failure).
*   **`Side Effects`**: If successful, the entire game state (memory, PC, stack) is replaced by the saved state. Player I/O for filename prompt.
*   **`Error Conditions`**:
    *   `SaveLoadEnable` flag in `flags1` is false.
    *   Save file not found or inaccessible.
    *   Save file is corrupted or not a valid ZM2 save file.
    *   Save file is for a different story file (ID, release, or checksum mismatch).
    *   Internal error during state deserialization.

#### j. `restart` (0OP)

*   **`Mnemonic`**: `restart`
*   **`Opcode Value`**: `0x0003`
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Restarts the game from the beginning, reinitializing memory to its original state from the story file.
*   **`Operation Details`**:
    1.  The VM reloads the game state as if it were starting fresh:
        *   The Dynamic Data Section is re-initialized (e.g., global variables reset from initial values, stack cleared).
        *   The Program Counter (PC) is set to the initial execution address specified in the story file header.
        *   The call stack is cleared.
        *   Any other dynamic aspects of game state (e.g., object locations if they were moved from their initial static definitions) are reset to their initial state as defined in the Static Data section or by initial game setup routines.
    2.  Execution begins from the initial PC. This operation does not return.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: The entire current game state is discarded and replaced with the initial game state. The screen may be cleared (interpreter-dependent).
*   **`Error Conditions`**: None typically, unless there's a severe issue reloading the initial state (which would be a VM bug).

#### k. `ret_popped` (0OP)

*   **`Mnemonic`**: `ret_popped`
*   **`Opcode Value`**: `0x0004`
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Returns from the current routine with the value currently at the top of the game stack. The value is popped from the stack.
*   **`Operation Details`**:
    1.  The value at the top of the game stack is popped. Let this be `returnValue`.
    2.  The current routine's stack frame is validated and prepared for removal.
    3.  The Program Counter (PC) is set to the return address stored in the current stack frame.
    4.  The current stack frame is popped from the call stack, restoring the previous frame pointer.
*   **`Stores Result To`**: The `returnValue` (popped from the stack) is stored into the variable indicated by the `store_variable_ref` of the `call` opcode that invoked the current routine.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Call stack depth decreases by one. One value is popped from the game stack.
*   **`Error Conditions`**:
    *   Stack underflow if the game stack is empty when `ret_popped` is called.
    *   Stack underflow if called when no routine call is active (e.g., from top-level execution).

#### l. `quit` (0OP)

*   **`Mnemonic`**: `quit`
*   **`Opcode Value`**: `0x0005`
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Terminates the game execution immediately.
*   **`Operation Details`**:
    1.  The VM halts all execution.
    2.  Any final messages (e.g., "Game Over") are typically printed by game logic *before* `quit` is called.
    3.  The interpreter may close files, release resources, and exit.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: The game ends. The VM stops.
*   **`Error Conditions`**: None.

#### m. `newline` (0OP)

*   **`Mnemonic`**: `newline`
*   **`Opcode Value`**: `0x0006`
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Prints a newline character (carriage return and line feed, or equivalent) to the current output stream.
*   **`Operation Details`**:
    1.  The VM sends a newline sequence (e.g., ZSCII code 13, or platform-specific newline) to the active output stream(s).
    2.  PC advances past this instruction.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Output cursor moves to the beginning of the next line.
*   **`Error Conditions`**: None, unless an I/O error occurs on the output stream.

#### n. `show_status` (0OP)

*   **`Mnemonic`**: `show_status`
*   **`Opcode Value`**: `0x0007`
*   **`Form`**: `0OP`
*   **`Operand Types`**: None.
*   **`Description`**: Requests the VM to display the game's status line (e.g., current room name, score, turns). This is a request; the VM is not obligated to redisplay it if it's already visible or not supported. In ZM2, this opcode's behavior is largely up to the interpreter's UI implementation.
*   **`Operation Details`**:
    1.  The VM receives the request to display the status line.
    2.  Traditionally (Version 3):
        *   The game's current score would be in global variable G1.
        *   The number of turns elapsed would be in global variable G2.
        *   The object ID of the player's current location is typically in a global variable (e.g., G0). The VM prints the short name of this object.
    3.  For ZM2, the interpreter may:
        *   Redraw a dedicated status line window or area if the UI supports it, using current values of score/turns variables (conventionally G1, G2) and player location.
        *   If the UI is purely sequential text, it might print a formatted status line directly to the main output stream.
        *   Do nothing if the status is always visible or the UI paradigm doesn't include a Z-Machine style status line.
    4.  PC advances past this instruction.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: May cause output to the screen.
*   **`Error Conditions`**: None.

#### o. `verify` (0OP)

*   **`Mnemonic`**: `verify`
*   **`Opcode Value`**: `0x0008`
*   **`Form`**: `0OP`
*   **`Operand Types`**: Branch data follows the opcode.
*   **`Description`**: Performs a checksum verification of the game file. Branches if the verification is successful. (Primarily relevant for detecting file corruption).
*   **`Operation Details`**:
    1.  The VM recalculates a checksum of the entire story file, from `code_section_start` to the end of `static_data_section_start + static_data_section_length` (or as specified for the original Z-Machine checksumming rules, typically covering all game data but not dynamic memory).
    2.  This calculated checksum is compared against the `checksum` value stored in the Header.
    3.  If the checksums match, the verification is successful, and a branch is performed.
    4.  If the checksums do not match, the verification fails, and execution continues with the instruction immediately following the branch data.
*   **`Stores Result To`**: Does not store a result. Success/failure is indicated by the branch.
*   **`Branches If`**: The game file's checksum is correct. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1). The 'sense' bit should be set to branch on true (correct checksum).
*   **`Side Effects`**: None, other than CPU time for checksum calculation.
*   **`Error Conditions`**: File I/O error if parts of the game file need to be re-read and fail.

#### p. `piracy` (0OP)

*   **`Mnemonic`**: `piracy`
*   **`Opcode Value`**: `0x0009`
*   **`Form`**: `0OP`
*   **`Operand Types`**: Branch data follows the opcode.
*   **`Description`**: Informs the VM that the game believes it is a pirated copy. This opcode branches to a new location if the game is *not* pirated (according to some internal VM check, which is usually a placeholder). In ZM2, this is largely a legacy opcode; its behavior might be vestigial or tied to a simple interpreter flag.
*   **`Operation Details`**:
    1.  This opcode is a signal from the game to the interpreter.
    2.  Historically, interpreters would simply branch as if the game were genuine, making it ineffective as a copy protection measure.
    3.  For ZM2, the VM could:
        *   Always branch (treat as genuine).
        *   Check a (hypothetical) interpreter-specific flag or setting related to licensing.
        *   Perform some other platform-specific check.
    4.  If the VM considers the game "genuine" (or by default), a branch is performed.
    5.  If the VM considers the game "pirated", execution continues with the instruction immediately following the branch data.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: The VM determines the game is "genuine" (which is the typical default behavior). The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1). The 'sense' bit should be set to branch on true (genuine).
*   **`Side Effects`**: None, unless the game code branched to by this opcode performs some action.
*   **`Error Conditions`**: None.

---

*(The following 1OP opcode definitions continue the "Example Opcode Definitions" numbering, starting from 'q'. They generally follow a pattern where a `type_byte` after the opcode specifies the operand's type, unless the opcode implicitly defines the operand type (e.g., `inc` always takes a Variable).)*

*   **Operand Type Specifier Byte (`type_byte`) for 1OP Opcodes (unless otherwise specified):**
    *   `0x00`: Large Constant (LC) - 8 bytes follow for the operand.
    *   `0x01`: Small Constant (SC) - 1 byte follows for the operand.
    *   `0x02`: Variable (VAR) - 1 byte follows (specifies stack/local/global for the operand).
    *   `0x03`: Packed Address (PADDR) - 4 bytes follow (used by `print_paddr`).
    *(Note: Specific opcodes might only support a subset of these types.)*

#### q. `get_sibling` (1OP)

*   **`Mnemonic`**: `get_sibling`
*   **`Opcode Value`**: `0x0101`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `object_id`.
    *   `object_id (LC/SC/VAR)`: The 64-bit ID of the object whose sibling is to be retrieved. Must be a valid object ID > 0.
    *   `store_variable_ref (Variable Reference)`: Variable to store the resulting sibling object ID.
    *   `branch_data?`: Branch if the object has a sibling.
*   **`Description`**: Gets the ID of the next object in the sibling chain of `object_id`. Stores the ID in `store_variable_ref`. Branches if a sibling exists.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `object_id` according to the type.
    2.  Validate `object_id`. If 0 or invalid, behavior is undefined (typically results in 0 and no branch, or an error).
    3.  Retrieve the `sibling_obj_id` from the object entry for `object_id` (see Section 3, Static Data Section, Object Table).
    4.  Store the retrieved `sibling_obj_id` (which can be 0 if no sibling) into `store_variable_ref`.
    5.  The condition is `sibling_obj_id != 0`.
    6.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1). If the branch is not taken (i.e., no sibling), execution continues normally.
*   **`Stores Result To`**: `store_variable_ref` (the sibling object ID, or 0 if none).
*   **`Branches If`**: `sibling_obj_id != 0`. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `object_id`.
    *   `object_id` is not a valid object ID.
    *   Invalid `store_variable_ref`.

#### r. `get_child` (1OP)

*   **`Mnemonic`**: `get_child`
*   **`Opcode Value`**: `0x0102`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `object_id`.
    *   `object_id (LC/SC/VAR)`: The 64-bit ID of the object whose first child is to be retrieved. Must be a valid object ID > 0.
    *   `store_variable_ref (Variable Reference)`: Variable to store the resulting child object ID.
    *   `branch_data?`: Branch if the object has a child.
*   **`Description`**: Gets the ID of the first child object of `object_id`. Stores the ID in `store_variable_ref`. Branches if a child exists.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `object_id`.
    2.  Validate `object_id`.
    3.  Retrieve the `child_obj_id` from the object entry for `object_id`.
    4.  Store the retrieved `child_obj_id` (or 0 if no child) into `store_variable_ref`.
    5.  The condition is `child_obj_id != 0`.
    6.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1). If the branch is not taken (i.e., no child), execution continues normally.
*   **`Stores Result To`**: `store_variable_ref` (the child object ID, or 0 if none).
*   **`Branches If`**: `child_obj_id != 0`. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `object_id`.
    *   `object_id` is not a valid object ID.
    *   Invalid `store_variable_ref`.

#### s. `get_parent` (1OP)

*   **`Mnemonic`**: `get_parent`
*   **`Opcode Value`**: `0x0103`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `object_id`.
    *   `object_id (LC/SC/VAR)`: The 64-bit ID of the object whose parent is to be retrieved. Must be a valid object ID > 0.
    *   `store_variable_ref (Variable Reference)`: Variable to store the resulting parent object ID.
*   **`Description`**: Gets the ID of the parent object of `object_id`. Stores the ID in `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `object_id`.
    2.  Validate `object_id`.
    3.  Retrieve the `parent_obj_id` from the object entry for `object_id`.
    4.  Store the retrieved `parent_obj_id` (or 0 if no parent) into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref` (the parent object ID, or 0 if none).
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `object_id`.
    *   `object_id` is not a valid object ID.
    *   Invalid `store_variable_ref`.

#### t. `get_prop_len` (1OP)

*   **`Mnemonic`**: `get_prop_len`
*   **`Opcode Value`**: `0x0104`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `property_address`.
    *   `property_address (LC/SC/VAR)`: The 64-bit byte address of the *start of a property entry's `id_and_length` field(s)* (this is the value typically obtained from `get_prop_addr`).
    *   `store_variable_ref (Variable Reference)`: Variable to store the length of the property data in bytes.
*   **`Description`**: Gets the length (in bytes) of the property data, determined from its `id_and_length` field at `property_address`.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `property_address`. Fetch if VAR.
    2.  If `property_address` is 0, store 0 into `store_variable_ref` and return.
    3.  Read Byte 1 (B1) at `property_address`. Let Bit 7 be `len_spec`.
    4.  If `len_spec` is 0: The property data length is 1 byte. Store 1 into `store_variable_ref`.
    5.  If `len_spec` is 1: Read Byte 2 (B2) at `property_address + 1`. This value `L` (1-255) is the property data length. If `L` is 0 from this byte, it's an invalid property encoding; the VM should halt with an error "Invalid property length 0". Otherwise, store `L` into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref` (length in bytes, or 0).
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `property_address`.
    *   `property_address` is invalid (e.g., not within valid memory regions for properties, or points to data that is not a property entry).
    *   Malformed property data at `property_address` (e.g., length byte indicates an invalid length like 0 when 2 bytes are used for size).
    *   Invalid `store_variable_ref`.

#### u. `inc` (1OP)

*   **`Mnemonic`**: `inc`
*   **`Opcode Value`**: `0x0105`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: The variable to be incremented (stack/local/global). This is provided directly as a 1-byte variable specifier; no preceding `type_byte`.
*   **`Description`**: Increments the value of the specified variable by 1.
*   **`Operation Details`**:
    1.  Read the 1-byte `variable_ref`.
    2.  Fetch the 64-bit value from the specified variable.
    3.  Add 1 to the value. Arithmetic is signed, 64-bit two's complement. Wraps on overflow (e.g., MAX_INT + 1 = MIN_INT).
    4.  Store the result back into the specified variable.
*   **`Stores Result To`**: The incremented value is stored back into `variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: The value of `variable_ref` is changed.
*   **`Error Conditions`**: Invalid `variable_ref`.

#### v. `dec` (1OP)

*   **`Mnemonic`**: `dec`
*   **`Opcode Value`**: `0x0106`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: The variable to be decremented. Provided directly as a 1-byte specifier.
*   **`Description`**: Decrements the value of the specified variable by 1.
*   **`Operation Details`**:
    1.  Read the 1-byte `variable_ref`.
    2.  Fetch the 64-bit value from the specified variable.
    3.  Subtract 1 from the value. Arithmetic is signed, 64-bit two's complement. Wraps on underflow (e.g., MIN_INT - 1 = MAX_INT).
    4.  Store the result back into the specified variable.
*   **`Stores Result To`**: The decremented value is stored back into `variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: The value of `variable_ref` is changed.
*   **`Error Conditions`**: Invalid `variable_ref`.

#### w. `print_addr` (1OP)

*   **`Mnemonic`**: `print_addr`
*   **`Opcode Value`**: `0x0107`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `byte_address`.
    *   `byte_address (LC/SC/VAR)`: The 64-bit byte address of the Z-encoded string to be printed.
*   **`Description`**: Prints the Z-encoded string located at `byte_address` in memory.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `byte_address`.
    2.  Validate `byte_address` (must be within valid memory, typically Static Data or Dynamic Data).
    3.  The VM reads Z-chars from `byte_address`, decodes them according to ZSCII rules (including abbreviation expansion, etc.), and prints them to the current output stream.
    4.  Printing stops when the Z-string termination sequence is encountered.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Text is printed to the output.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `byte_address`.
    *   `byte_address` is invalid or points to non-string data.
    *   I/O error on output stream.

#### x. `remove_obj` (1OP)

*   **`Mnemonic`**: `remove_obj`
*   **`Opcode Value`**: `0x0108`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `object_id`.
    *   `object_id (LC/SC/VAR)`: The 64-bit ID of the object to be removed from its parent. Must be a valid object ID > 0.
*   **`Description`**: Detaches `object_id` from its parent. The object itself is not deleted from memory, but it no longer resides within another object. Its parent becomes 0.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `object_id`.
    2.  Validate `object_id`. If 0 or invalid, operation fails silently or logs an error.
    3.  Let `P` be the parent of `object_id`. If `object_id` has no parent (`parent_obj_id == 0`), do nothing.
    4.  If `P.child_obj_id == object_id`, then `P.child_obj_id` is set to `object_id.sibling_obj_id`.
    5.  Otherwise, find `PrevSibling` of `object_id` (i.e., the object `X` such that `X.sibling_obj_id == object_id`). Set `PrevSibling.sibling_obj_id` to `object_id.sibling_obj_id`.
    6.  Set `object_id.parent_obj_id` to 0.
    7.  Set `object_id.sibling_obj_id` to 0 (conventionally, though not strictly required by original Z-Spec, it's cleaner for a detached object).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Object tree is modified. `object_id` has its `parent_obj_id` (and possibly `sibling_obj_id`) changed. Its former parent and/or previous sibling will also be updated.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `object_id`.
    *   `object_id` is not a valid object ID.

#### y. `print_obj` (1OP)

*   **`Mnemonic`**: `print_obj`
*   **`Opcode Value`**: `0x0109`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `object_id`.
    *   `object_id (LC/SC/VAR)`: The 64-bit ID of the object whose short name is to be printed. Must be a valid object ID > 0.
*   **`Description`**: Prints the short name of the specified object.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `object_id`.
    2.  Validate `object_id`. If 0 or invalid, print nothing or an error/debug message.
    3.  Retrieve the `property_table_ptr` for `object_id`.
    4.  The first entry in an object's property table is a 64-bit address pointing to its Z-encoded short name string. Read this address.
    5.  Print the Z-encoded string at this address (similar to `print_addr`).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Text (object's short name) is printed to the output.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `object_id`.
    *   `object_id` is not a valid object ID.
    *   Object has no short name property or property table pointer is invalid.
    *   I/O error on output stream.

#### z. `ret` (1OP)

*   **`Mnemonic`**: `ret`
*   **`Opcode Value`**: `0x010A`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `value`.
    *   `value (LC/SC/VAR)`: The 64-bit value to be returned from the current routine.
*   **`Description`**: Returns the `value` from the current routine.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `value` operand (fetching from variable if VAR).
    2.  The current routine's stack frame is validated.
    3.  The Program Counter (PC) is set to the return address stored in the current stack frame.
    4.  The current stack frame is popped from the call stack.
    5.  The `value` is stored into the variable indicated by the `store_variable_ref` of the `call` opcode that invoked the current routine.
*   **`Stores Result To`**: The `value` is stored in the caller's specified result variable.
*   **`Branches If`**: Does not branch, but transfers control to the return address.
*   **`Side Effects`**: Call stack depth decreases by one.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `value`.
    *   Invalid variable reference if `value` is VAR.
    *   Stack underflow if called when no routine call is active.

#### aa. `jump` (1OP)

*   **`Mnemonic`**: `jump`
*   **`Opcode Value`**: `0x010B`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `offset`.
    *   `offset (SC)`: A signed 16-bit value, typically provided as a Small Constant (SC), directly embedded as 2 bytes (Big Endian) after the type byte. If LC is somehow provided, only the lowest 16 bits are used and interpreted as signed.
*   **`Description`**: Unconditionally jumps to a new address calculated by adding a signed 16-bit `offset`.
*   **`Operation Details`**:
    1.  Read `type_byte`. If SC, read 2 bytes for `offset_value` (signed, Big Endian). If LC, read 8 bytes and take the lower 2 bytes, interpreting them as a signed 16-bit Big Endian value. Other types for `offset` are invalid.
    2.  The `offset_value` (now a signed 16-bit value) is sign-extended to 64 bits.
    3.  The new PC is calculated as: `Address_Of_Instruction_Following_Jump_Opcode_And_Its_Operands + offset_value`. (Note: Z-Machine branch/jump offsets are often calculated from the address *after* the offset data itself. If this `jump` opcode has the opcode byte (1), one type byte (1), and two bytes for SC offset (2), the 'Address_Of_Instruction_Following_Jump_Opcode_And_Its_Operands' would be PC_of_opcode + 1 + 1 + 2 = PC_of_opcode + 4).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Always (unconditional jump).
*   **`Side Effects`**: PC is modified.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `offset` (must be SC or LC).
    *   Resulting PC is outside valid code section.

#### ab. `print_paddr` (1OP)

*   **`Mnemonic`**: `print_paddr`
*   **`Opcode Value`**: `0x010C`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Must indicate PADDR type for ZM2.
    *   `packed_address (StringPADDR)`: A 32-bit packed address (StringPADDR type) of a Z-encoded string within the Static Data Section.
*   **`Description`**: Prints the Z-encoded string located at the `packed_address`.
*   **`Operation Details`**:
    1.  Read `type_byte`. It should indicate PADDR (e.g. 0x03).
    2.  Read the 4-byte `packed_address` value.
    3.  Expand `packed_address` (which is a `StringPADDR` as defined in Section 4.A.0) to a full 64-bit absolute byte address: `AbsoluteAddress = static_data_section_start + packed_address`.
    4.  Validate `AbsoluteAddress`.
    5.  Print the Z-encoded string at `AbsoluteAddress` (similar to `print_addr`).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Text is printed.
*   **`Error Conditions`**:
    *   `type_byte` does not specify PADDR.
    *   Resulting `AbsoluteAddress` is invalid or points to non-string data.
    *   I/O error.

#### ac. `load` (1OP)

*   **`Mnemonic`**: `load`
*   **`Opcode Value`**: `0x010D`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: The variable whose content is to be loaded. This is provided directly as a 1-byte specifier.
    *   `store_variable_ref (Variable Reference)`: Variable to store the loaded value.
*   **`Description`**: Reads the value from `variable_ref` and stores it into `store_variable_ref`. (This is distinct from Z-Spec `load` which is `load variable -> (result)` where result is an implicit store. ZM2 makes store explicit like other ops).
*   **`Operation Details`**:
    1.  Read the 1-byte `variable_ref` (source).
    2.  Fetch the 64-bit value from this source variable.
    3.  Read the `store_variable_ref` (destination).
    4.  Store the fetched value into the destination variable.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None, beyond storing the value.
*   **`Error Conditions`**:
    *   Invalid `variable_ref` (source).
    *   Invalid `store_variable_ref` (destination).

#### ad. `not` (1OP)

*   **`Mnemonic`**: `not`
*   **`Opcode Value`**: `0x010E`
*   **`Form`**: `1OP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: Specifies type of `value`.
    *   `value (LC/SC/VAR)`: The value to be bitwise NOTed.
    *   `store_variable_ref (Variable Reference)`: Variable to store the result.
*   **`Description`**: Performs a bitwise NOT operation on the `value` and stores the result.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `value` operand.
    2.  Perform bitwise NOT: `result = ~value`. This is a 64-bit operation.
    3.  Store `result` into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid `type_byte` for `value`.
    *   Invalid variable reference if `value` is VAR.
    *   Invalid `store_variable_ref`.

---

*(The following 2OP opcode definitions continue the "Example Opcode Definitions" numbering. They generally follow the pattern where two `type_byte`s after the opcode specify the operands' types, as detailed in the `add` opcode example (section 3.c). An explicit `store_variable_ref` or `branch_data` will follow the operands if the opcode stores a result or branches.)*

*   **Operand Type Specifier Bytes (`type_byte1`, `type_byte2`) for 2OP Opcodes:**
    *   Each operand is preceded by a type byte.
    *   `0x00`: Large Constant (LC) - 8 bytes follow for the operand.
    *   `0x01`: Small Constant (SC) - 1 byte follows for the operand.
    *   `0x02`: Variable (VAR) - 1 byte follows (specifies stack/local/global for the operand).
    *(Note: Specific opcodes might only support a subset of these types for each operand.)*

#### ae. `je` (2OP)

*   **`Mnemonic`**: `je`
*   **`Opcode Value`**: `0x0201`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: The first value for comparison.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: The second value for comparison.
    *   `branch_data?`: Branch information.
*   **`Description`**: Jump if `value1` is equal to `value2`. Can compare multiple values if `value2` is followed by more type/value pairs for `value3`, `value4`, etc., jumping if `value1` equals *any* of them. (This extended behavior is from ZSpec 5.2.2.1). For ZM2, we'll first define the basic two-operand `je`. A VAROP `je_varargs` could be defined for the multi-comparison version if needed, or this `je` could be specified to handle more. For now, strictly two operands.
*   **`Operation Details`**:
    1.  Read `type_byte1`, then `value1`. Fetch if VAR.
    2.  Read `type_byte2`, then `value2`. Fetch if VAR.
    3.  The condition is `value1 == value2`.
    4.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `value1 == value2`. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `type_byte1` or `type_byte2`.
    *   Invalid variable reference if operands are VAR.

#### af. `jl` (2OP)

*   **`Mnemonic`**: `jl`
*   **`Opcode Value`**: `0x0202`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: The first value.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: The second value.
    *   `branch_data?`: Branch information.
*   **`Description`**: Jump if `value1` is less than `value2` (signed 64-bit comparison).
*   **`Operation Details`**:
    1.  Read `type_byte1`, then `value1`. Fetch if VAR.
    2.  Read `type_byte2`, then `value2`. Fetch if VAR.
    3.  The condition is `value1 < value2` (signed 64-bit comparison).
    4.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `value1 < value2` (signed). The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `type_byte1` or `type_byte2`.
    *   Invalid variable reference if operands are VAR.

#### ag. `jg` (2OP)

*   **`Mnemonic`**: `jg`
*   **`Opcode Value`**: `0x0203`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: The first value.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: The second value.
    *   `branch_data?`: Branch information.
*   **`Description`**: Jump if `value1` is greater than `value2` (signed 64-bit comparison).
*   **`Operation Details`**:
    1.  Read `type_byte1`, then `value1`. Fetch if VAR.
    2.  Read `type_byte2`, then `value2`. Fetch if VAR.
    3.  The condition is `value1 > value2` (signed 64-bit comparison).
    4.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `value1 > value2` (signed). The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `type_byte1` or `type_byte2`.
    *   Invalid variable reference if operands are VAR.

#### ah. `dec_chk` (2OP)

*   **`Mnemonic`**: `dec_chk`
*   **`Opcode Value`**: `0x0204`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: The variable to be decremented (1-byte specifier, type is implicitly VAR).
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_compare`.
    *   `value_to_compare (LC/SC/VAR)`: The value to compare against after decrementing.
    *   `branch_data?`: Branch information.
*   **`Description`**: Decrements the variable `variable_ref` by 1 (signed). Then, branches if the new value of `variable_ref` is less than `value_to_compare`.
*   **`Operation Details`**:
    1.  Read `variable_ref` (1 byte). Fetch its 64-bit value.
    2.  Decrement the value by 1 (signed 64-bit arithmetic, wraps on underflow).
    3.  Store the new value back into `variable_ref`.
    4.  Read `type_byte_val`, then `value_to_compare`. Fetch if VAR.
    5.  The condition is `(new value of variable_ref) < value_to_compare` (signed comparison).
    6.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: The decremented value is stored back into `variable_ref`.
*   **`Branches If`**: `(decremented_variable_ref_value) < value_to_compare` (signed). The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: `variable_ref` is modified. PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `variable_ref`.
    *   Invalid `type_byte_val`.
    *   Invalid variable reference for `value_to_compare` if VAR.

#### ai. `inc_chk` (2OP)

*   **`Mnemonic`**: `inc_chk`
*   **`Opcode Value`**: `0x0205`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: The variable to be incremented (1-byte specifier, type is implicitly VAR).
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_compare`.
    *   `value_to_compare (LC/SC/VAR)`: The value to compare against after incrementing.
    *   `branch_data?`: Branch information.
*   **`Description`**: Increments the variable `variable_ref` by 1 (signed). Then, branches if the new value of `variable_ref` is greater than `value_to_compare`.
*   **`Operation Details`**:
    1.  Read `variable_ref` (1 byte). Fetch its 64-bit value.
    2.  Increment the value by 1 (signed 64-bit arithmetic, wraps on overflow).
    3.  Store the new value back into `variable_ref`.
    4.  Read `type_byte_val`, then `value_to_compare`. Fetch if VAR.
    5.  The condition is `(new value of variable_ref) > value_to_compare` (signed comparison).
    6.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: The incremented value is stored back into `variable_ref`.
*   **`Branches If`**: `(incremented_variable_ref_value) > value_to_compare` (signed). The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: `variable_ref` is modified. PC may be modified.
*   **`Error Conditions`**:
    *   Invalid `variable_ref`.
    *   Invalid `type_byte_val`.
    *   Invalid variable reference for `value_to_compare` if VAR.

#### aj. `jin` (2OP)

*   **`Mnemonic`**: `jin`
*   **`Opcode Value`**: `0x0206`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj1 (Operand Type Specifier)`: For `object_id1`.
    *   `object_id1 (LC/SC/VAR)`: The first object ID (child).
    *   `type_byte_obj2 (Operand Type Specifier)`: For `object_id2`.
    *   `object_id2 (LC/SC/VAR)`: The second object ID (parent).
    *   `branch_data?`: Branch information.
*   **`Description`**: Jump if `object_id1` is a direct child of `object_id2`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj1`, then `object_id1`. Fetch if VAR. Must be a valid object ID > 0.
    2.  Read `type_byte_obj2`, then `object_id2`. Fetch if VAR. Must be a valid object ID > 0.
    3.  Retrieve the `parent_obj_id` of `object_id1`.
    4.  The condition is `parent_obj_id == object_id2`.
    5.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: `object_id1` is a child of `object_id2`. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id1` or `object_id2` are not valid object IDs.

#### ak. `test` (2OP)

*   **`Mnemonic`**: `test`
*   **`Opcode Value`**: `0x0207`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_bmp (Operand Type Specifier)`: For `bitmap`.
    *   `bitmap (LC/SC/VAR)`: A 64-bit bitmap.
    *   `type_byte_flags (Operand Type Specifier)`: For `flags`.
    *   `flags (LC/SC/VAR)`: A 64-bit value representing flags to test.
    *   `branch_data?`: Branch information.
*   **`Description`**: Performs a bitwise AND of `bitmap` and `flags`. Branches if the result is non-zero.
*   **`Operation Details`**:
    1.  Read `type_byte_bmp`, then `bitmap`. Fetch/extend.
    2.  Read `type_byte_flags`, then `flags`. Fetch/extend.
    3.  Calculate `result = bitmap & flags`.
    4.  The condition is `result != 0`.
    5.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result. (Original Z-Spec `test` stores the result, ZM2 `test` is a branch op only, like `test_attr`).
*   **`Branches If`**: `(bitmap & flags) != 0`. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   Invalid variable references.

#### al. `or` (2OP)

*   **`Mnemonic`**: `or`
*   **`Opcode Value`**: `0x0208`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: The first value.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: The second value.
    *   `store_variable_ref (Variable Reference)`: Variable to store the result.
*   **`Description`**: Performs a bitwise OR of `value1` and `value2`. Stores the result in `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `type_byte1`, then `value1`. Fetch/extend.
    2.  Read `type_byte2`, then `value2`. Fetch/extend.
    3.  Calculate `result = value1 | value2` (64-bit bitwise OR).
    4.  Store `result` into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   Invalid variable references.

#### am. `and` (2OP)

*   **`Mnemonic`**: `and`
*   **`Opcode Value`**: `0x0209`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: The first value.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: The second value.
    *   `store_variable_ref (Variable Reference)`: Variable to store the result.
*   **`Description`**: Performs a bitwise AND of `value1` and `value2`. Stores the result in `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `type_byte1`, then `value1`. Fetch/extend.
    2.  Read `type_byte2`, then `value2`. Fetch/extend.
    3.  Calculate `result = value1 & value2` (64-bit bitwise AND).
    4.  Store `result` into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   Invalid variable references.

#### an. `test_attr` (2OP)

*   **`Mnemonic`**: `test_attr`
*   **`Opcode Value`**: `0x020A`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_attr (Operand Type Specifier)`: For `attribute_id`.
    *   `attribute_id (LC/SC/VAR)`: The attribute number (0-63).
    *   `branch_data?`: Branch information.
*   **`Description`**: Tests if the object `object_id` has the attribute `attribute_id` set. Branches if true.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be a valid object ID > 0.
    2.  Read `type_byte_attr`, then `attribute_id`. Fetch/extend. Must be 0-63.
    3.  Retrieve the 64-bit `attributes` field for `object_id`.
    4.  The condition is `(attributes >> attribute_id) & 1 == 1`.
    5.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Object has the attribute. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is not a valid object ID.
    *   `attribute_id` is out of range (0-63).

#### ao. `set_attr` (2OP)

*   **`Mnemonic`**: `set_attr`
*   **`Opcode Value`**: `0x020B`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_attr (Operand Type Specifier)`: For `attribute_id`.
    *   `attribute_id (LC/SC/VAR)`: The attribute number (0-63) to set.
*   **`Description`**: Sets attribute `attribute_id` on object `object_id`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid.
    2.  Read `type_byte_attr`, then `attribute_id`. Fetch/extend. Must be 0-63.
    3.  Retrieve the 64-bit `attributes` field for `object_id`.
    4.  Set bit `attribute_id`: `attributes = attributes | (1ULL << attribute_id)`.
    5.  Store the modified `attributes` back to the object table entry for `object_id`. (Note: Assumes object table attributes are writable, which they must be for ZM2).
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Attribute of the object is modified.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is not a valid object ID.
    *   `attribute_id` is out of range (0-63).

#### ap. `clear_attr` (2OP)

*   **`Mnemonic`**: `clear_attr`
*   **`Opcode Value`**: `0x020C`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_attr (Operand Type Specifier)`: For `attribute_id`.
    *   `attribute_id (LC/SC/VAR)`: The attribute number (0-63) to clear.
*   **`Description`**: Clears attribute `attribute_id` on object `object_id`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid.
    2.  Read `type_byte_attr`, then `attribute_id`. Fetch/extend. Must be 0-63.
    3.  Retrieve the 64-bit `attributes` field for `object_id`.
    4.  Clear bit `attribute_id`: `attributes = attributes & ~(1ULL << attribute_id)`.
    5.  Store the modified `attributes` back.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Attribute of the object is modified.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is not a valid object ID.
    *   `attribute_id` is out of range (0-63).

#### aq. `insert_obj` (2OP)

*   **`Mnemonic`**: `insert_obj`
*   **`Opcode Value`**: `0x020D`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID to be moved.
    *   `type_byte_dest (Operand Type Specifier)`: For `destination_id`.
    *   `destination_id (LC/SC/VAR)`: The object ID of the new parent.
*   **`Description`**: Moves `object_id` to become the first child of `destination_id`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid (>0).
    2.  Read `type_byte_dest`, then `destination_id`. Fetch if VAR. Must be valid (>0).
    3.  If `object_id == destination_id`, error (cannot insert object into itself).
    4.  Call `remove_obj` internally for `object_id` to detach it from its current parent.
    5.  Set `object_id.parent_obj_id = destination_id`.
    6.  Set `object_id.sibling_obj_id = destination_id.child_obj_id`.
    7.  Set `destination_id.child_obj_id = object_id`.
*   **`Stores Result To`**: Does not store a result.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Object tree is significantly modified.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` or `destination_id` are not valid object IDs or are 0.
    *   Attempting to insert an object into itself.
    *   `destination_id` is an invalid parent (e.g., not a container, though Z-Machine typically doesn't enforce this at opcode level).

#### ar. `loadw` (2OP)

*   **`Mnemonic`**: `loadw`
*   **`Opcode Value`**: `0x020E`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_arr (Operand Type Specifier)`: For `array_addr`.
    *   `array_addr (LC/SC/VAR)`: Byte address of the start of the array/table.
    *   `type_byte_idx (Operand Type Specifier)`: For `word_index`.
    *   `word_index (LC/SC/VAR)`: Index of the word to load (0-based).
    *   `store_variable_ref (Variable Reference)`: Variable to store the loaded 64-bit word.
*   **`Description`**: Loads a 64-bit word from `array_addr + (word_index * 8)` into `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `type_byte_arr`, then `array_addr`. Fetch if VAR.
    2.  Read `type_byte_idx`, then `word_index`. Fetch/extend.
    3.  Calculate effective address: `EffectiveAddr = array_addr + (word_index * 8)`. (Ensure `word_index` is treated as unsigned for positive offset, or signed if negative indices are meaningful).
    4.  Validate `EffectiveAddr` and `EffectiveAddr + 7` are within valid memory.
    5.  Read the 64-bit word from `EffectiveAddr` (Big Endian).
    6.  Store this word into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   Invalid variable references.
    *   `EffectiveAddr` is out of bounds.

#### as. `loadb` (2OP)

*   **`Mnemonic`**: `loadb`
*   **`Opcode Value`**: `0x020F`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_arr (Operand Type Specifier)`: For `array_addr`.
    *   `array_addr (LC/SC/VAR)`: Byte address of the start of the array/table.
    *   `type_byte_idx (Operand Type Specifier)`: For `byte_index`.
    *   `byte_index (LC/SC/VAR)`: Index of the byte to load (0-based).
    *   `store_variable_ref (Variable Reference)`: Variable to store the loaded byte (zero-extended to 64-bit).
*   **`Description`**: Loads a byte from `array_addr + byte_index` into `store_variable_ref`.
*   **`Operation Details`**:
    1.  Read `type_byte_arr`, then `array_addr`. Fetch if VAR.
    2.  Read `type_byte_idx`, then `byte_index`. Fetch/extend.
    3.  Calculate effective address: `EffectiveAddr = array_addr + byte_index`.
    4.  Validate `EffectiveAddr` is within valid memory.
    5.  Read the byte from `EffectiveAddr`.
    6.  Zero-extend the byte to a 64-bit value.
    7.  Store this value into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   Invalid variable references.
    *   `EffectiveAddr` is out of bounds.

#### at. `get_prop` (2OP)

*   **`Mnemonic`**: `get_prop`
*   **`Opcode Value`**: `0x0210`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_prop (Operand Type Specifier)`: For `property_id`.
    *   `property_id (LC/SC/VAR)`: The property ID (1-63).
    *   `store_variable_ref (Variable Reference)`: Variable to store the property value.
*   **`Description`**: Gets the value of property `property_id` from object `object_id`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid.
    2.  Read `type_byte_prop`, then `property_id`. Fetch/extend. Must be 1-63.
    3.  Locate the property data for `object_id` and `property_id`.
        *   Get `property_table_ptr` for `object_id`.
        *   Iterate through properties until `property_id` is matched or end of list.
    4.  If property not found:
        *   Read `property_defaults_table_start` from the Header.
        *   If `property_defaults_table_start` is 0 or points outside valid static data, the default value is 0.
        *   Otherwise, the default value is read from `property_defaults_table_start + ((property_id - 1) * 8)`. (Property IDs are 1-63).
    5.  If property found:
        *   Determine its actual data length (PL) using a mechanism similar to `get_prop_len` (i.e., by looking at the `id_and_length` field associated with this property, whose address is obtained via a `get_prop_addr`-like mechanism).
        *   Let P_DATA_START be the address of the first byte of the property's actual data (i.e., `address_of_id_and_length_field + 1` or `+ 2` depending on the size of the `id_and_length` field itself).
        *   If PL is 1 byte: Read 1 byte from P_DATA_START, zero-extend to 64-bit.
        *   If PL is 2 bytes: Read 2 bytes (Big Endian) from P_DATA_START, zero-extend to 64-bit.
        *   If PL is 4 bytes: Read 4 bytes (Big Endian) from P_DATA_START, zero-extend to 64-bit.
        *   If PL >= 8 bytes: Read the first 8 bytes (Big Endian) as a 64-bit word from P_DATA_START.
        *   If PL is 3, 5, 6, or 7 bytes: Read PL bytes from P_DATA_START and zero-extend to a 64-bit value, preserving the order of bytes read in the lower part of the 64-bit result (e.g., if PL=3, and bytes read are B0, B1, B2, the 64-bit result would be `0x0000000000[B2][B1][B0]`).
        *   If PL is 0 (due to malformed property where `get_prop_len` logic would identify this, or if `get_prop_addr` returned 0 and was then used incorrectly), this should be treated as 'property not found', and the default value should be returned.
    6.  Store the value into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is invalid.
    *   `property_id` is invalid (0 or >63).
    *   Malformed property data (e.g., inconsistent length encoding).

#### au. `get_prop_addr` (2OP)

*   **`Mnemonic`**: `get_prop_addr`
*   **`Opcode Value`**: `0x0211`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_prop (Operand Type Specifier)`: For `property_id`.
    *   `property_id (LC/SC/VAR)`: The property ID (1-63).
    *   `store_variable_ref (Variable Reference)`: Variable to store the byte address of the property's `id_and_length` field. Returns 0 if property not present.
*   **`Description`**: Gets the byte address of the property's `id_and_length` field for property `property_id` of object `object_id`. Returns 0 if property not present.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid.
    2.  Read `type_byte_prop`, then `property_id`. Fetch/extend. Must be 1-63.
    3.  Locate the property:
        *   Get `property_table_ptr` for `object_id`.
        *   Iterate through properties. If `property_id` found, the address stored is the address of the *first byte of its `id_and_length` field*.
    4.  If property not found, store 0.
    5.  Store address into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref` (address or 0).
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is invalid.
    *   `property_id` is invalid.

#### av. `get_next_prop` (2OP)

*   **`Mnemonic`**: `get_next_prop`
*   **`Opcode Value`**: `0x0212`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_prop (Operand Type Specifier)`: For `property_id`.
    *   `property_id (LC/SC/VAR)`: Previous property ID. If 0, get first property ID.
    *   `store_variable_ref (Variable Reference)`: Variable to store the next property ID (1-63). Stores 0 if no more properties.
*   **`Description`**: Finds the next property ID for `object_id` after `property_id`.
*   **`Operation Details`**:
    1.  Read `type_byte_obj`, then `object_id`. Fetch if VAR. Must be valid.
    2.  Read `type_byte_prop`, then `property_id`. Fetch/extend. (0 to find first, or 1-63).
    3.  Get `property_table_ptr` for `object_id`. Skip object's short name.
    4.  If `property_id == 0`:
        *   Read first property's ID. Store it. If no properties, store 0.
    5.  Else (`property_id` is 1-63):
        *   Iterate through properties until `property_id` is found.
        *   The ID of the *next* property in the list is stored. If current `property_id` is last, store 0.
    6.  Store result (next property ID or 0) into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**:
    *   Invalid type bytes.
    *   `object_id` is invalid.
    *   `property_id` is invalid (not 0 and not 1-63, or not found if not 0).

#### aw. `sub` (2OP)

*   **`Mnemonic`**: `sub`
*   **`Opcode Value`**: `0x0213`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1 (Operand Type Specifier)`: For `value1`.
    *   `value1 (LC/SC/VAR)`: Minuend.
    *   `type_byte2 (Operand Type Specifier)`: For `value2`.
    *   `value2 (LC/SC/VAR)`: Subtrahend.
    *   `store_variable_ref (Variable Reference)`: Stores `value1 - value2`.
*   **`Description`**: Subtracts `value2` from `value1`. Stores result.
*   **`Operation Details`**:
    1.  Read types and values for `value1`, `value2`.
    2.  `result = value1 - value2` (64-bit signed, wraps).
    3.  Store `result`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid types or variable refs.

#### ax. `mul` (2OP)

*   **`Mnemonic`**: `mul`
*   **`Opcode Value`**: `0x0214`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1`: For `value1`.
    *   `value1 (LC/SC/VAR)`.
    *   `type_byte2`: For `value2`.
    *   `value2 (LC/SC/VAR)`.
    *   `store_variable_ref`.
*   **`Description`**: Multiplies `value1` by `value2`. Stores result.
*   **`Operation Details`**:
    1.  Read types/values.
    2.  `result = value1 * value2` (64-bit signed, wraps).
    3.  Store `result`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid types or variable refs.

#### ay. `div` (2OP)

*   **`Mnemonic`**: `div`
*   **`Opcode Value`**: `0x0215`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1`: For `dividend`.
    *   `dividend (LC/SC/VAR)`.
    *   `type_byte2`: For `divisor`.
    *   `divisor (LC/SC/VAR)`.
    *   `store_variable_ref`.
*   **`Description`**: Divides `dividend` by `divisor` (signed integer division). Stores result.
*   **`Operation Details`**:
    1.  Read types/values.
    2.  If `divisor` is 0: Halt VM if `DivByZeroHalt` flag is set in header. Otherwise, result is 0.
    3.  Else: `result = dividend / divisor` (64-bit signed integer division, result truncates towards zero).
    4.  Store `result`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Potential halt on division by zero.
*   **`Error Conditions`**: Invalid types/var refs. Division by zero (if not halting).

#### az. `mod` (2OP)

*   **`Mnemonic`**: `mod`
*   **`Opcode Value`**: `0x0216`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte1`: For `dividend`.
    *   `dividend (LC/SC/VAR)`.
    *   `type_byte2`: For `divisor`.
    *   `divisor (LC/SC/VAR)`.
    *   `store_variable_ref`.
*   **`Description`**: Calculates `dividend` modulo `divisor`. Stores result.
*   **`Operation Details`**:
    1.  Read types/values.
    2.  If `divisor` is 0: Halt VM if `DivByZeroHalt` flag set. Otherwise, result is 0.
    3.  Else: `result = dividend % divisor`. Result has same sign as dividend.
    4.  Store `result`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Potential halt on division by zero.
*   **`Error Conditions`**: Invalid types/var refs. Division by zero (if not halting).

#### ba. `set_colour` (2OP)

*   **`Mnemonic`**: `set_colour`
*   **`Opcode Value`**: `0x0217`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_fg (Operand Type Specifier)`: For `foreground_color`.
    *   `foreground_color (LC/SC/VAR)`: Z-Machine color code for foreground.
    *   `type_byte_bg (Operand Type Specifier)`: For `background_color`.
    *   `background_color (LC/SC/VAR)`: Z-Machine color code for background.
*   **`Description`**: Sets the foreground and background colors for text display. (V5+)
*   **`Operation Details`**:
    1.  Read `type_byte_fg`, then `foreground_color`. Fetch/extend.
    2.  Read `type_byte_bg`, then `background_color`. Fetch/extend.
    3.  The VM interprets these color codes (e.g., 1=current, 2=black, 3=red, etc.) and attempts to set the display colors.
    4.  If the interpreter doesn't support color, this opcode does nothing.
    5.  ZM2 specific: Color codes could be expanded or reinterpreted (e.g. 24-bit RGB values if LC is used, or standard ZSCII color codes if SC is used). For now, assume standard ZSCII color codes (1-9).
*   **`Stores Result To`**: Does not store.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Display colors may change.
*   **`Error Conditions`**: Invalid type bytes or variable refs. Invalid color codes (if VM validates).

#### bb. `throw` (2OP)

*   **`Mnemonic`**: `throw`
*   **`Opcode Value`**: `0x0218`
*   **`Form`**: `2OP`
*   **`Operand Types`**:
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_throw`.
    *   `value_to_throw (LC/SC/VAR)`: The value to be thrown.
    *   `type_byte_catch (Operand Type Specifier)`: For `catch_frame_id`.
    *   `catch_frame_id (LC/SC/VAR)`: The ID of the catch frame to throw to.
*   **`Description`**: Throws a value to a catch frame, unwinding the stack until the catch frame `catch_frame_id` is found. (V5+)
*   **`Operation Details`**:
    1.  Read `type_byte_val`, then `value_to_throw`. Fetch/extend.
    2.  Read `type_byte_catch`, then `catch_frame_id`. Fetch/extend.
    3.  The VM unwinds the call stack, frame by frame.
    4.  For each frame, it checks if it's a "catch frame" (set by `catch` opcode, which would store its ID).
    5.  If a catch frame with a matching ID is found:
        *   The stack is restored to that frame's state.
        *   `value_to_throw` is stored in the result variable specified by that `catch` frame.
        *   Execution continues from the instruction after the `catch` opcode.
    6.  If the bottom of the stack is reached and no matching catch frame is found, the VM halts with an error "Uncaught throw".
*   **`Stores Result To`**: Indirectly, to the result variable of a `catch` opcode.
*   **`Branches If`**: Does not branch in the typical sense, but transfers control.
*   **`Side Effects`**: Stack is unwound. PC changes.
*   **`Error Conditions`**: Invalid type bytes or variable refs. Uncaught throw if `catch_frame_id` not found.

---

*(The following VAROP opcode definitions continue the "Example Opcode Definitions" numbering. VAROPs in ZM2 typically read a sequence of operand type bytes followed by the operands themselves. The number of operands is determined by the specific opcode. Refer to the `call` opcode (section 3.d) for a detailed example of operand parsing for VAROPs. Each operand that is not fixed (like a store_variable_ref) will be preceded by a type byte.)*

*   **Operand Type Specifier Byte (`type_byte`) for VAROP Operands (unless otherwise specified for a particular operand):**
    *   `0x00`: Large Constant (LC) - 8 bytes follow for the operand.
    *   `0x01`: Small Constant (SC) - 1 byte follows for the operand.
    *   `0x02`: Variable (VAR) - 1 byte follows (specifies stack/local/global for the operand).
    *   `0x03`: Packed Address (PADDR) - 4 bytes follow.
    *(Note: Not all opcodes will support all types for all operands.)*

#### bc. `storew` (VAROP)

*   **`Mnemonic`**: `storew`
*   **`Opcode Value`**: `0x0302`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_arr (Operand Type Specifier)`: For `array_addr`.
    *   `array_addr (LC/SC/VAR)`: Byte address of the start of the array/table.
    *   `type_byte_idx (Operand Type Specifier)`: For `word_index`.
    *   `word_index (LC/SC/VAR)`: Index of the word to write to (0-based).
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_store`.
    *   `value_to_store (LC/SC/VAR)`: The 64-bit word to store.
*   **`Description`**: Stores `value_to_store` at address `array_addr + (word_index * 8)`.
*   **`Operation Details`**:
    1.  Read `type_byte_arr`, then `array_addr`. Fetch if VAR.
    2.  Read `type_byte_idx`, then `word_index`. Fetch/extend.
    3.  Read `type_byte_val`, then `value_to_store`. Fetch if VAR.
    4.  Calculate effective address: `EffectiveAddr = array_addr + (word_index * 8)`.
    5.  Validate `EffectiveAddr` and `EffectiveAddr + 7` are within valid writable memory.
    6.  Write the 64-bit `value_to_store` to `EffectiveAddr` (Big Endian).
*   **`Stores Result To`**: Does not store a result in a variable (stores in memory).
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Memory is modified at `EffectiveAddr`.
*   **`Error Conditions`**: Invalid type bytes. Invalid variable references. `EffectiveAddr` out of bounds.

#### bd. `storeb` (VAROP)

*   **`Mnemonic`**: `storeb`
*   **`Opcode Value`**: `0x0303`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_arr (Operand Type Specifier)`: For `array_addr`.
    *   `array_addr (LC/SC/VAR)`: Byte address of the start of the array/table.
    *   `type_byte_idx (Operand Type Specifier)`: For `byte_index`.
    *   `byte_index (LC/SC/VAR)`: Index of the byte to write to (0-based).
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_store`.
    *   `value_to_store (LC/SC/VAR)`: The byte value to store (lowest 8 bits are used).
*   **`Description`**: Stores the lowest 8 bits of `value_to_store` at address `array_addr + byte_index`.
*   **`Operation Details`**:
    1.  Read `type_byte_arr`, then `array_addr`. Fetch if VAR.
    2.  Read `type_byte_idx`, then `byte_index`. Fetch/extend.
    3.  Read `type_byte_val`, then `value_to_store`. Fetch if VAR.
    4.  Calculate effective address: `EffectiveAddr = array_addr + byte_index`.
    5.  Validate `EffectiveAddr` is within valid writable memory.
    6.  Take the lowest 8 bits of `value_to_store`.
    7.  Write this byte to `EffectiveAddr`.
*   **`Stores Result To`**: Does not store a result in a variable.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Memory is modified at `EffectiveAddr`.
*   **`Error Conditions`**: Invalid type bytes. Invalid variable references. `EffectiveAddr` out of bounds.

#### be. `put_prop` (VAROP)

*   **`Mnemonic`**: `put_prop`
*   **`Opcode Value`**: `0x0304`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_obj (Operand Type Specifier)`: For `object_id`.
    *   `object_id (LC/SC/VAR)`: The object ID.
    *   `type_byte_prop_id (Operand Type Specifier)`: For `property_id`.
    *   `property_id (LC/SC/VAR)`: The property ID (1-63).
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_store`.
    *   `value_to_store (LC/SC/VAR)`: The value to store in the property.
*   **`Description`**: Writes `value_to_store` to property `property_id` of object `object_id`.
*   **`Operation Details`**:
    1.  Read operands: `object_id`, `property_id`, `value_to_store` using their type bytes.
    2.  Validate `object_id` (must be a valid object ID > 0) and `property_id` (must be 1-63).
    3.  Locate the property data address (PDA) for `object_id` and `property_id` using a mechanism similar to `get_prop_addr` (i.e., finding the address of the `id_and_length` field).
    4.  If the property does not exist on the object (PDA is 0), the VM MUST halt with a fatal error: "Attempted to write to non-existent property [property_id] on object [object_id]".
    5.  Determine the actual data length (PL) of the property using a mechanism similar to `get_prop_len` based on the `id_and_length` field at PDA. If this indicates an invalid encoding (e.g., a 2-byte size field encoding a length of 0), the VM should halt with an error.
    6.  Let P_DATA_START be `PDA + 1` if the size of the `id_and_length` field is 1 byte (i.e. data length is 1), or `PDA + 2` if the size of the `id_and_length` field is 2 bytes.
    7.  The `value_to_store` (a 64-bit value) is written into the property's data field starting at P_DATA_START as follows:
        *   If PL is 1 byte: The lowest byte of `value_to_store` is written to P_DATA_START.
        *   If PL is 2 bytes: The lowest 2 bytes of `value_to_store` are written (Big Endian) starting at P_DATA_START.
        *   If PL is 4 bytes: The lowest 4 bytes of `value_to_store` are written (Big Endian) starting at P_DATA_START.
        *   If PL >= 8 bytes: The full 64-bit `value_to_store` is written (Big Endian) starting at P_DATA_START. If PL > 8, only the first 8 bytes of the property data are modified by this `put_prop` operation.
        *   If PL is 3, 5, 6, or 7 bytes: The lowest PL bytes of `value_to_store` are written (Big Endian for multi-byte segments) starting at P_DATA_START. For example, if PL is 3, bytes 0, 1, 2 of `value_to_store` (representing the lowest 3 bytes of the 64-bit value when viewed in Big Endian order) are written.
*   **`Stores Result To`**: Does not store a result in a variable.
*   **`Branches If`**: Does not branch.
*   **`Side Effects`**: Property value of an object is modified.
*   **`Error Conditions`**: Invalid type bytes. Invalid `object_id` or `property_id`. Property not found on object. Property has an invalid encoded length (e.g. 0 from a 2-byte size field).

#### bf. `print_char` (VAROP)

*   **`Mnemonic`**: `print_char`
*   **`Opcode Value`**: `0x0305`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: For `char_code`.
    *   `char_code (LC/SC/VAR)`: ZSCII character code (0-255) or Unicode code point to print.
*   **`Description`**: Prints a single character.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `char_code`. Fetch/extend.
    2.  If `StrictZSCIICompatMode` is ON: `char_code` is treated as a ZSCII character code (0-255). If the `char_code` represents a printable ZSCII character (including ZSCII newline 13), it is printed. If it's an undefined or unprintable ZSCII value (other than defined control characters like newline), '?' is printed. Values outside the 0-255 range when this mode is ON should ideally be an error or print '?'.
    3.  If `StrictZSCIICompatMode` is OFF (default Unicode mode): `char_code` is treated as a Unicode code point. The VM attempts to print this character. If the specific Unicode character cannot be rendered by the terminal/interpreter, a substitution character (like '?') may be displayed by the terminal itself.
    4.  Standard ZSCII codes (e.g., 13 for newline) are honored, particularly for their control function if applicable, regardless of mode.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Character printed.
*   **`Error Conditions`**: Invalid type byte/var ref. I/O error.

#### bg. `print_num` (VAROP)

*   **`Mnemonic`**: `print_num`
*   **`Opcode Value`**: `0x0306`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: For `value`.
    *   `value (LC/SC/VAR)`: Signed 64-bit integer to print.
*   **`Description`**: Prints `value` as a signed decimal number.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `value`. Fetch/extend.
    2.  Convert the 64-bit signed integer to its decimal string representation.
    3.  Print the string.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Number printed.
*   **`Error Conditions`**: Invalid type byte/var ref. I/O error.

#### bh. `random` (VAROP)

*   **`Mnemonic`**: `random`
*   **`Opcode Value`**: `0x0307`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_range (Operand Type Specifier)`: For `range`.
    *   `range (LC/SC/VAR)`: A positive 64-bit integer.
    *   `store_variable_ref (Variable Reference)`: Stores the random number.
*   **`Description`**: Generates a random number between 1 and `range` (inclusive).
*   **`Operation Details`**:
    1.  Read `type_byte_range`, then `range`. Fetch/extend.
    2.  If `range` is positive: Generate a pseudo-random integer `R` such that `1 <= R <= range`.
    3.  If `range` is negative or zero:
        *   Negative: Seeds the PRNG with `range` (absolute value). Result stored is 0.
        *   Zero: Reseeds PRNG with a time-dependent value. Result stored is 0.
    4.  Store result into `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: None.
*   **`Side Effects`**: PRNG state may change.
*   **`Error Conditions`**: Invalid type byte/var ref for `range`. Invalid `store_variable_ref`.

#### bi. `push` (VAROP)

*   **`Mnemonic`**: `push`
*   **`Opcode Value`**: `0x0308`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: For `value`.
    *   `value (LC/SC/VAR)`: The 64-bit value to push onto the game stack.
*   **`Description`**: Pushes `value` onto the game stack.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `value`. Fetch/extend.
    2.  Push `value` onto the game stack. Stack pointer is incremented.
*   **`Stores Result To`**: None directly (value is on stack).
*   **`Branches If`**: None.
*   **`Side Effects`**: Game stack modified.
*   **`Error Conditions`**: Invalid type byte/var ref. Stack overflow.

#### bj. `pull` (VAROP)

*   **`Mnemonic`**: `pull`
*   **`Opcode Value`**: `0x0309`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: Variable to store the value popped from the stack. If `variable_ref` is stack (0x00), value is popped and discarded.
*   **`Description`**: Pops the top value from the game stack and stores it into `variable_ref`.
*   **`Operation Details`**:
    1.  Read `variable_ref` (1 byte).
    2.  Pop value from game stack. Stack pointer is decremented.
    3.  If `variable_ref` is not 0x00 (stack), store the popped value into the specified local/global variable.
    4.  If `variable_ref` is 0x00 (stack), the value is discarded. (This matches original Z-Spec `pull 0` behavior).
*   **`Stores Result To`**: `variable_ref` (unless it's stack).
*   **`Branches If`**: None.
*   **`Side Effects`**: Game stack modified.
*   **`Error Conditions`**: Invalid `variable_ref`. Stack underflow.

#### bk. `split_window` (VAROP)

*   **`Mnemonic`**: `split_window`
*   **`Opcode Value`**: `0x030A`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_lines (Operand Type Specifier)`: For `lines`.
    *   `lines (LC/SC/VAR)`: Number of lines for the upper window (window 1).
*   **`Description`**: Splits the screen into two windows. Upper window (window 1) gets `lines` lines. Lower window (window 0) gets the rest. (V3+)
*   **`Operation Details`**:
    1.  Read `type_byte_lines`, then `lines`. Fetch/extend.
    2.  VM attempts to split the screen. Window 1 (upper) has `lines` height. Window 0 (lower) takes remaining.
    3.  Current window becomes 0. Cursor moves to window 0.
    4.  If `lines` is 0, only lower window exists.
    5.  If interpreter cannot split, this may do nothing or set a flag.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Screen layout changes. Active window changes.
*   **`Error Conditions`**: Invalid type byte/var ref. `lines` value invalid (e.g., > screen height).

#### bl. `set_window` (VAROP)

*   **`Mnemonic`**: `set_window`
*   **`Opcode Value`**: `0x030B`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_win (Operand Type Specifier)`: For `window_id`.
    *   `window_id (LC/SC/VAR)`: Window to select (0 for lower, 1 for upper).
*   **`Description`**: Selects the active window for text output. (V3+)
*   **`Operation Details`**:
    1.  Read `type_byte_win`, then `window_id`. Fetch/extend.
    2.  If `window_id` is 0 or 1, set it as active window. Cursor moves to its current position in that window.
    3.  If windowing not supported/split, all output goes to single window.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Active output window may change.
*   **`Error Conditions`**: Invalid type byte/var ref. `window_id` not 0 or 1.

#### bm. `erase_window` (VAROP)

*   **`Mnemonic`**: `erase_window`
*   **`Opcode Value`**: `0x030C`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_win (Operand Type Specifier)`: For `window_id`.
    *   `window_id (LC/SC/VAR)`: Window to erase (-1 for all, 0 for lower, 1 for upper).
*   **`Description`**: Clears the specified window(s) and moves cursor to top-left of that window. (V4+)
*   **`Operation Details`**:
    1.  Read `type_byte_win`, then `window_id`. Fetch/extend.
    2.  If -1: Erase all windows, move cursor to (1,1) of window 0.
    3.  If 0 or 1: Erase specified window, move cursor to (1,1) of that window.
    4.  If windowing not supported, erases the whole screen.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Screen content erased. Cursor moved.
*   **`Error Conditions`**: Invalid type byte/var ref. Invalid `window_id`.

#### bn. `erase_line` (VAROP)

*   **`Mnemonic`**: `erase_line`
*   **`Opcode Value`**: `0x030D`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_val (Operand Type Specifier)`: For `value`.
    *   `value (LC/SC/VAR)`: If 1, erase from cursor to end of line. (Other values undefined by spec, ZM2 treats non-1 as error or no-op).
*   **`Description`**: Erases from cursor to end of line in current window. (V4+)
*   **`Operation Details`**:
    1.  Read `type_byte_val`, then `value`. Fetch/extend.
    2.  If `value == 1`, erase from cursor to end of line. Cursor position does not change.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Part of a line is erased.
*   **`Error Conditions`**: Invalid type byte/var ref.

#### bo. `set_cursor` (VAROP)

*   **`Mnemonic`**: `set_cursor`
*   **`Opcode Value`**: `0x030E`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_row (Operand Type Specifier)`: For `row`.
    *   `row (LC/SC/VAR)`: Screen row (1-based).
    *   `type_byte_col (Operand Type Specifier)`: For `column`.
    *   `column (LC/SC/VAR)`: Screen column (1-based).
    *   (Optional, V5+) `type_byte_win (Operand Type Specifier)`: For `window_id`.
    *   (Optional, V5+) `window_id (LC/SC/VAR)`: Window to set cursor in. Defaults to current window if omitted.
*   **`Description`**: Moves cursor to (`row`, `column`) in current (or specified) window. (V4+)
*   **`Operation Details`**:
    1.  Read `row` and `column` with their type bytes.
    2.  Optionally, check for and read `window_id` (if ZM2 supports this V5+ extension). If not provided, use current window.
    3.  Move cursor. If window 1, `row` cannot be set (always 1).
    4.  Ignored if interpreter doesn't support cursor setting.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Cursor position changed.
*   **`Error Conditions`**: Invalid types/var refs. Position out of bounds.

#### bp. `get_cursor` (VAROP)

*   **`Mnemonic`**: `get_cursor`
*   **`Opcode Value`**: `0x030F`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_arr (Operand Type Specifier)`: For `array_addr`.
    *   `array_addr (LC/SC/VAR)`: Byte address of an array to store cursor position.
*   **`Description`**: Stores current cursor position (row and column) into `array_addr`. (V4+)
*   **`Operation Details`**:
    1.  Read `type_byte_arr`, then `array_addr`. Fetch if VAR.
    2.  Get current cursor `row` and `column` in the active window.
    3.  Store `row` at `array_addr` (as a 64-bit word).
    4.  Store `column` at `array_addr + 8` (as a 64-bit word).
    5.  If interpreter cannot determine cursor pos, it should store plausible values (e.g., 1,1) or specific error indicators if array format allows.
*   **`Stores Result To`**: Row and column stored in memory at `array_addr` and `array_addr+8`.
*   **`Branches If`**: None.
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid type byte/var ref. `array_addr` invalid.

#### bq. `set_text_style` (VAROP)

*   **`Mnemonic`**: `set_text_style`
*   **`Opcode Value`**: `0x0310`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_style (Operand Type Specifier)`: For `style_flags`.
    *   `style_flags (LC/SC/VAR)`: Bitmask for styles (reverse, bold, italic, fixed-pitch).
*   **`Description`**: Sets current text style for output. (V4+)
*   **`Operation Details`**:
    1.  Read `type_byte_style`, then `style_flags`. Fetch/extend.
    2.  Bit flags: 0=roman, 1=reverse, 2=bold, 4=italic, 8=fixed-pitch.
    3.  VM attempts to apply style. If a style is unsupported, it's ignored.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Text style for subsequent output changes.
*   **`Error Conditions`**: Invalid type byte/var ref.

#### br. `buffer_mode` (VAROP)

*   **`Mnemonic`**: `buffer_mode`
*   **`Opcode Value`**: `0x0311`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_mode (Operand Type Specifier)`: For `mode`.
    *   `mode (LC/SC/VAR)`: If 0, disable buffered output. If 1, enable.
*   **`Description`**: Controls whether text output is buffered or printed immediately. (V4+)
*   **`Operation Details`**:
    1.  Read `type_byte_mode`, then `mode`. Fetch/extend.
    2.  If `mode == 1`, text output may be buffered by interpreter until newline, screen full, or read.
    3.  If `mode == 0`, output is unbuffered (flushed immediately).
    4.  Mainly affects performance on slow devices; less critical for ZM2 but defined for compatibility.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Interpreter output buffering behavior might change.
*   **`Error Conditions`**: Invalid type byte/var ref.

#### bs. `output_stream` (VAROP)

*   **`Mnemonic`**: `output_stream`
*   **`Opcode Value`**: `0x0312`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_stream (Operand Type Specifier)`: For `stream_id`.
    *   `stream_id (LC/SC/VAR)`: Stream number (1=screen, 2=transcript, 3=memory table, 4=script output for command file). Negative to close.
    *   (If `stream_id == 3`) `type_byte_table (Operand Type Specifier)`: For `table_addr`.
    *   (If `stream_id == 3`) `table_addr (LC/SC/VAR)`: Byte address of table to print to. First word stores length.
*   **`Description`**: Selects or deselects output streams. (V3+)
*   **`Operation Details`**:
    1.  Read `stream_id`.
    2.  Positive `stream_id`: Selects stream.
        *   1: Screen. Always on.
        *   2: Transcript. If `Transcripting` flag not set in header, this fails silently or errors.
        *   3: Memory table. Read `table_addr`. Output is written to this table. First 64-bit word at `table_addr` must be max capacity; VM updates it with current length.
        *   4: Command file output (if enabled).
    3.  Negative `stream_id`: Deselects stream `abs(stream_id)`. Cannot deselect stream 1.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Output may be redirected.
*   **`Error Conditions`**: Invalid types/var refs. Invalid stream ID. Invalid `table_addr` or capacity for stream 3.

#### bt. `input_stream` (VAROP)

*   **`Mnemonic`**: `input_stream`
*   **`Opcode Value`**: `0x0313`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_stream (Operand Type Specifier)`: For `stream_id`.
    *   `stream_id (LC/SC/VAR)`: Stream number (0=keyboard, 1=command file).
*   **`Description`**: Selects input stream (keyboard or command file). (V3+)
*   **`Operation Details`**:
    1.  Read `stream_id`.
    2.  0: Keyboard input.
    3.  1: Command file input (if available). If not, error or ignore.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Input source changes.
*   **`Error Conditions`**: Invalid type/var ref. Command file not available for stream 1.

#### bu. `sound_effect` (VAROP)

*   **`Mnemonic`**: `sound_effect`
*   **`Opcode Value`**: `0x0314`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_num (Operand Type Specifier)`: For `sound_number`.
    *   `sound_number (LC/SC/VAR)`: Sound effect ID.
    *   `type_byte_effect (Operand Type Specifier)`: For `effect`.
    *   `effect (LC/SC/VAR)`: 1=prepare, 2=play, 3=stop, 4=unload.
    *   `type_byte_vol (Operand Type Specifier)`: For `volume_and_repeats`.
    *   `volume_and_repeats (LC/SC/VAR)`: Volume (high byte), repeats (low byte).
    *   (Optional) `type_byte_routine (Operand Type Specifier)`: For `sound_finished_routine_paddr`.
    *   (Optional) `sound_finished_routine_paddr (RoutinePADDR)`: Routine to call when sound finishes.
*   **`Description`**: Manages sound effects. (V4+) Often optional in interpreters.
*   **`Operation Details`**:
    1.  Read operands.
    2.  VM attempts to perform action. If sound not supported, mostly NOP.
    3.  `sound_number`: 1 for beep, 2 for user-defined sound. Higher numbers are game-specific.
    4.  `effect`: Prepare (load), Play, Stop, Unload (release resources).
    5.  `volume_and_repeats`: Volume 1-8 (or 0-255). Repeats 0-255 (0=infinite, 1=once).
    6.  `sound_finished_routine_paddr`: If provided (and non-zero), resolve this `RoutinePADDR` (as per Section 4.A.0) to an absolute address and the VM will attempt to call the routine at this address when the sound effect finishes playing.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Sound may play or stop.
*   **`Error Conditions`**: Invalid types/var refs/paddr. Sound system errors.

#### bv. `read_char` (VAROP)

*   **`Mnemonic`**: `read_char`
*   **`Opcode Value`**: `0x0315`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_one (Operand Type Specifier)`: For `input_device_ignored`. (Typically 1, historical).
    *   `input_device_ignored (LC/SC/VAR)`: Usually 1.
    *   (Optional) `type_byte_timeout (Operand Type Specifier)`: For `timeout_seconds`.
    *   (Optional) `timeout_seconds (LC/SC/VAR)`: Timeout value.
    *   (Optional) `type_byte_routine (Operand Type Specifier)`: For `timeout_routine_paddr`.
    *   (Optional) `timeout_routine_paddr (PADDR)`: Routine to call on timeout.
    *   `store_variable_ref (Variable Reference)`: Stores the character code read.
*   **`Description`**: Reads a single character from the current input stream. (V4+)
*   **`Operation Details`**:
    1.  Read `input_device_ignored`. (ZM2 ignores this, reads from current input stream).
    2.  If timeout operands provided: Read them. Resolve `timeout_routine_paddr` (a `RoutinePADDR` as per Section 4.A.0) to an absolute address. Start timer. If timeout occurs before char read, call the routine at the resolved address (if `timeout_routine_paddr` was non-zero and resolved successfully) and the result stored is 0.
    3.  Read one character.
    4.  Store ZSCII/Unicode code into `store_variable_ref`. If timeout occurred, 0 is stored.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: None.
*   **`Side Effects`**: Waits for input.
*   **`Error Conditions`**: Invalid types/var refs/paddr. I/O error.

#### bw. `scan_table` (VAROP)

*   **`Mnemonic`**: `scan_table`
*   **`Opcode Value`**: `0x0316`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_val (Operand Type Specifier)`: For `value_to_find`.
    *   `value_to_find (LC/SC/VAR)`: Value to search for.
    *   `type_byte_table (Operand Type Specifier)`: For `table_addr`.
    *   `table_addr (LC/SC/VAR)`: Address of the table.
    *   `type_byte_len (Operand Type Specifier)`: For `num_items`.
    *   `num_items (LC/SC/VAR)`: Number of items to search in the table.
    *   (Optional) `type_byte_form (Operand Type Specifier)`: For `form_byte`.
    *   (Optional) `form_byte (LC/SC/VAR)`: Default 0x82 (words). Bit 7=0 for bytes, 1 for words. Bits 0-6 length of field.
    *   `store_variable_ref (Variable Reference)`: Stores address of found item, or 0.
    *   `branch_data?`: Branch if found.
*   **`Description`**: Searches a table for `value_to_find`. (V4+)
*   **`Operation Details`**:
    1.  Read operands. `form_byte` defaults to 0x82 (scan 64-bit words).
    2.  Iterate from `table_addr` for `num_items`. Each item is `form_byte & 0x7F` bytes long.
    3.  If `(form_byte & 0x80) == 0` (bytes): Compare `value_to_find` (lowest byte if `value_to_find` is wider than field) with each byte field.
    4.  If `(form_byte & 0x80) != 0` (words): Compare `value_to_find` (relevant bytes if field < 8 bytes, else full 64-bit) with each word field.
    5.  If found: Store item's address in `store_variable_ref`. The condition for branching is true.
    6.  If not found: Store 0. The condition for branching is false.
    7.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: Value found in table. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid types/var refs. Invalid table address or form.

#### bx. `print_unicode` (VAROP)

*   **`Mnemonic`**: `print_unicode`
*   **`Opcode Value`**: `0x0317`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: For `unicode_char_code`.
    *   `unicode_char_code (LC/SC/VAR)`: The 32-bit Unicode code point to print.
*   **`Description`**: Prints a Unicode character.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `unicode_char_code`. Fetch/extend.
    2.  The VM attempts to print the character corresponding to `unicode_char_code`.
    3.  If `StrictZSCIICompatMode` is ON, VM attempts to transliterate to ZSCII or print '?'.
    4.  If the character cannot be printed, '?' is typically output.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Character printed.
*   **`Error Conditions`**: Invalid type/var ref. I/O error.

#### by. `check_unicode` (VAROP)

*   **`Mnemonic`**: `check_unicode`
*   **`Opcode Value`**: `0x0318`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte (Operand Type Specifier)`: For `unicode_char_code`.
    *   `unicode_char_code (LC/SC/VAR)`: The 32-bit Unicode code point to check.
    *   `store_variable_ref (Variable Reference)`: Stores result.
*   **`Description`**: Checks if the current output stream(s) can print the given Unicode character.
*   **`Operation Details`**:
    1.  Read `type_byte`, then `unicode_char_code`. Fetch/extend.
    2.  Result:
        *   1: Character can be printed accurately.
        *   0: Cannot be printed (will be substituted, e.g., with '?').
        *   2: Can be approximated/transliterated.
    3.  Store result in `store_variable_ref`.
*   **`Stores Result To`**: `store_variable_ref`.
*   **`Branches If`**: None.
*   **`Side Effects`**: None.
*   **`Error Conditions`**: Invalid type/var ref.

#### bz. `store` (VAROP)

*   **`Mnemonic`**: `store`
*   **`Opcode Value`**: `0x0319`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `variable_ref (Variable Reference)`: Variable to store into (1-byte specifier).
    *   `type_byte_val (Operand Type Specifier)`: For `value`.
    *   `value (LC/SC/VAR)`: The 64-bit value to store.
*   **`Description`**: Stores `value` into `variable_ref`.
*   **`Operation Details`**:
    1.  Read `variable_ref` (1 byte).
    2.  Read `type_byte_val`, then `value`. Fetch/extend.
    3.  Store `value` into the variable specified by `variable_ref`.
*   **`Stores Result To`**: `variable_ref`.
*   **`Branches If`**: None.
*   **`Side Effects`**: Target variable is modified.
*   **`Error Conditions`**: Invalid `variable_ref`. Invalid type byte or var ref for `value`.

#### ca. `tokenise` (VAROP)

*   **`Mnemonic`**: `tokenise`
*   **`Opcode Value`**: `0x031A`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_text (Operand Type Specifier)`: For `text_buffer_addr`.
    *   `text_buffer_addr (LC/SC/VAR)`: Address of ZSCII text to tokenise.
    *   `type_byte_parse (Operand Type Specifier)`: For `parse_buffer_addr`.
    *   `parse_buffer_addr (LC/SC/VAR)`: Address of buffer to store parse data.
    *   (Optional) `type_byte_dict (Operand Type Specifier)`: For `dictionary_addr`.
    *   (Optional) `dictionary_addr (LC/SC/VAR)`: Address of dictionary to use (default from header).
    *   (Optional) `type_byte_flag (Operand Type Specifier)`: For `flag_ignore_unknowns`.
    *   (Optional) `flag_ignore_unknowns (LC/SC/VAR)`: If non-zero, don't fill parse buffer slots for unknown words.
*   **`Description`**: Tokenises text from `text_buffer_addr` into `parse_buffer_addr`, using dictionary.
*   **`Operation Details`**:
    1.  Read operands. Use header dictionary if `dictionary_addr` is 0 or omitted.
    2.  Similar to `sread`'s parsing phase:
        *   `text_buffer_addr` format: Max length (byte), actual length (byte), ZSCII text.
        *   `parse_buffer_addr` format: Max words (byte), then receives num_words (byte), then word entries (64-bit dict_addr, 8-bit len, 32-bit offset).
    3.  Performs tokenisation based on dictionary word separators and entries.
*   **`Stores Result To`**: None directly (parse data in `parse_buffer_addr`).
*   **`Branches If`**: None.
*   **`Side Effects`**: `parse_buffer_addr` is modified.
*   **`Error Conditions`**: Invalid types/var refs. Invalid buffer addresses or formats.

#### cb. `encode_text` (VAROP)

*   **`Mnemonic`**: `encode_text`
*   **`Opcode Value`**: `0x031B`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_zscii (Operand Type Specifier)`: For `zscii_text_addr`.
    *   `zscii_text_addr (LC/SC/VAR)`: Address of null-terminated ZSCII text.
    *   `type_byte_len (Operand Type Specifier)`: For `length`.
    *   `length (LC/SC/VAR)`: Number of ZSCII characters to encode.
    *   `type_byte_from (Operand Type Specifier)`: For `from_offset`.
    *   `from_offset (LC/SC/VAR)`: Starting offset in `zscii_text_addr`.
    *   `type_byte_encoded (Operand Type Specifier)`: For `encoded_text_buffer_addr`.
    *   `encoded_text_buffer_addr (LC/SC/VAR)`: Buffer to store Z-encoded text.
*   **`Description`**: Encodes ZSCII text into Z-Machine string format (6 bytes per 4 chars). (V5+)
*   **`Operation Details`**:
    1.  Read operands.
    2.  Encode `length` chars from `zscii_text_addr + from_offset` into `encoded_text_buffer_addr`.
    3.  Uses standard Z-Machine text encoding algorithm (A0, A1, A2 character sets, shifts).
*   **`Stores Result To`**: None directly (encoded data in `encoded_text_buffer_addr`).
*   **`Branches If`**: None.
*   **`Side Effects`**: `encoded_text_buffer_addr` modified.
*   **`Error Conditions`**: Invalid types/var refs. Buffer overlaps or overflows.

#### cc. `copy_table` (VAROP)

*   **`Mnemonic`**: `copy_table`
*   **`Opcode Value`**: `0x031C`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_src (Operand Type Specifier)`: For `source_table_addr`.
    *   `source_table_addr (LC/SC/VAR)`: Address of source table.
    *   `type_byte_dst (Operand Type Specifier)`: For `destination_table_addr`.
    *   `destination_table_addr (LC/SC/VAR)`: Address of destination table.
    *   `type_byte_len (Operand Type Specifier)`: For `length_bytes`.
    *   `length_bytes (LC/SC/VAR)`: Number of bytes to copy. Positive: copy forwards. Negative: copy backwards. Zero: do nothing.
*   **`Description`**: Copies `abs(length_bytes)` bytes from source to destination. (V5+)
*   **`Operation Details`**:
    1.  Read operands.
    2.  If `length_bytes > 0`: Copy `length_bytes` from `source_table_addr` to `destination_table_addr` (memmove semantics, handles overlap correctly by copying low-to-high).
    3.  If `length_bytes < 0`: Copy `abs(length_bytes)` from `source_table_addr` to `destination_table_addr` (memmove semantics, handles overlap by copying high-to-low). Destination can be 0 to zero out source table if `source_table_addr` is in dynamic memory. (Original spec: if dest is 0, source is zeroed. ZM2: if dest is 0, treat as error or NOP to avoid ambiguity). ZM2: If `destination_table_addr` is 0, this is an error.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Memory at destination (and potentially source if zeroing was allowed) is modified.
*   **`Error Conditions`**: Invalid types/var refs. Memory out of bounds. `destination_table_addr` is 0. Overlap with non-dynamic memory if zeroing.

#### cd. `print_table` (VAROP)

*   **`Mnemonic`**: `print_table`
*   **`Opcode Value`**: `0x031D`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_addr (Operand Type Specifier)`: For `table_addr`.
    *   `table_addr (LC/SC/VAR)`: Address of table of ZSCII characters.
    *   `type_byte_width (Operand Type Specifier)`: For `width`.
    *   `width (LC/SC/VAR)`: Width of each row in characters.
    *   `type_byte_height (Operand Type Specifier)`: For `height`.
    *   `height (LC/SC/VAR)`: Number of rows (default 1).
    *   (Optional) `type_byte_skip (Operand Type Specifier)`: For `skip_chars_per_row`.
    *   (Optional) `skip_chars_per_row (LC/SC/VAR)`: Number of bytes to skip after printing each row (default 0).
*   **`Description`**: Prints a table of ZSCII characters. (V5+)
*   **`Operation Details`**:
    1.  Read operands. `height` defaults to 1. `skip_chars_per_row` defaults to 0.
    2.  For each row from 1 to `height`:
        *   Print `width` ZSCII characters from current `table_addr`.
        *   Advance `table_addr` by `width`.
        *   Advance `table_addr` by `skip_chars_per_row`.
        *   Print a newline.
*   **`Stores Result To`**: None.
*   **`Branches If`**: None.
*   **`Side Effects`**: Text printed.
*   **`Error Conditions`**: Invalid types/var refs. Memory out of bounds.

#### ce. `check_arg_count` (VAROP)

*   **`Mnemonic`**: `check_arg_count`
*   **`Opcode Value`**: `0x031E`
*   **`Form`**: `VAROP`
*   **`Operand Types`**:
    *   `type_byte_arg_num (Operand Type Specifier)`: For `argument_number`.
    *   `argument_number (LC/SC/VAR)`: 1-based argument index.
    *   `branch_data?`: Branch information.
*   **`Description`**: Branches if the current routine was called with at least `argument_number` arguments. (V5+)
*   **`Operation Details`**:
    1.  Read `argument_number`.
    2.  Check the count of arguments passed to the current routine (stored in stack frame).
    3.  The condition is `count_of_args_passed >= argument_number`.
    4.  Branching is performed based on this condition and the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Stores Result To`**: None.
*   **`Branches If`**: Current routine received at least `argument_number` arguments. The actual branching logic is defined by the Standard Branch Data Format (see Section 4.A.2.1).
*   **`Side Effects`**: PC may be modified.
*   **`Error Conditions`**: Invalid type/var ref. `argument_number` is 0 or invalid.

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
        *   Crucially, for both NLU and NLG, the VM completes all necessary processing of the LLM's response (JSON parsing for NLU, text extraction and Z-Machine format conversion for NLG) *before* `check_llm_status` reports a status of `1` (Success). Thus, when `get_llm_result` confirms success, the data in `result_buffer_addr` is immediately usable by the Z-code in the expected format (a JSON string for NLU, a Z-encoded/Unicode-sequence string for NLG).
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
        *   The LLM is expected to return a JSON structure that corresponds to the 'Task: Parse the player's command...' part of the prompt. The VM will parse the LLM's overall JSON response (which often wraps the core content, e.g., within a `generated_text` field of an array element). The VM then extracts or reconstructs the meaningful JSON *object* representing the structured action and stores this object *as a JSON string* into `result_buffer_addr`.
        *   Example of the *content written by the VM to `result_buffer_addr`* after a successful LLM NLU task:
            ```json
            {
                "action": "take",
                "noun1": "small brass key",
                "preposition": "from",
                "noun2": "oak table",
                "original_command": "take the red key from the oak table"
            }
            ```
        *   This structured action format, now a direct JSON string in `result_buffer_addr`, is what game logic will work with. The VM handles unwrapping from the LLM API's specific response structure. Game logic should be able to parse this JSON string from `result_buffer_addr` directly. Adding `original_command` to the NLU output JSON can be useful for context or complex disambiguation in Z-code.
        *   The Z-Machine game logic then uses this structured data (e.g., verb "take", noun1 "small brass key", etc.) by parsing the JSON string from `result_buffer_addr` after `get_llm_result` confirms success.

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
            ```
            The VM expects the LLM's response for NLG tasks to be in this format (or similar, depending on the model service), specifically looking for the main generated textual content (e.g., in a field like `generated_text`). The VM extracts this plain text string (typically UTF-8). This extracted plain text is then processed by the VM (converted to ZSCII or a sequence of Unicode code points based on `StrictZSCIICompatMode`, as detailed in Section 7 Player Interaction / Output Formatting) and this processed, game-ready string is written into `result_buffer_addr`.

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
    *   **Object Data**: The current state of all game objects, including their parent-sibling-child relationships, attribute flags (e.g., `worn`, `lit`), and property values. While object definitions are in static memory, their dynamic state (e.g., current location, contents if a container, specific property values that can change) is part of the game state. For Z-Machine Mark 2, the `attributes` field of each object in the object table (defined in Section 3, Static Data Section, Object Table) **is considered part of the dynamic game state** and must be saved during a `save` operation. This means that opcodes like `set_attr` and `clear_attr` modify a version of the attributes that is part of the saveable state, even if the initial object table resides in the Static Data Section loaded from the story file. The VM must ensure that these modifications are correctly captured and restored.
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
    *   **Object Attributes**: Object attributes are stored as bitfields within each object's entry. For ZM2, the `attributes` field of every object defined in the story file's object table is considered writable and part of the dynamic game state. The `save` operation must include the current state of all object attributes. Opcodes like `set_attr`, `clear_attr`, and `test_attr` directly manipulate this saveable version of the attributes.
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
            *   **Stack Chunk (`Stks`)**: Contains a snapshot of the call stack. Each frame must serialize its 64-bit return Program Counter, the 64-bit previous frame pointer, the number of arguments passed to the routine, the variable reference for storing the routine's result, all 64-bit local variables, and any 64-bit temporary values pushed onto the evaluation stack portion of that frame.
            *   **Program Counter Chunk (`PC__`)**: Explicitly stores the 64-bit Program Counter (PC). Other key 64-bit CPU registers like the Stack Pointer (SP) and current routine's frame pointer would also be saved, either in this chunk or a dedicated CPU state chunk.
            *   **(Optional) LLM State Chunk (`LLMs`)**: If recent LLM interactions need to be saved (e.g., conversation history snippets to maintain context across save/load), a dedicated chunk could store this.
        4.  **Output**: The VM writes these chunks to a file, forming the save game file. The `save` opcode would typically branch if the operation is successful.

    *   **Deserialization (`restore` opcode)**:
        1.  **File Selection**: The player or VM environment provides the save game file.
        2.  **Header Verification (`IFhd`)**: The VM reads the `IFhd` chunk from the save file. It compares the story ID, release number, and checksum with the currently loaded story file. If they don't match, the restore operation fails (and typically branches accordingly or halts with an error).
        3.  **Memory Restoration**:
            *   The VM reads the memory chunk (`CMem` or `UMem`).
            *   If compressed, it's decompressed.
            *   The data is written back into the Z-Machine's Dynamic Data Section, overwriting its current content.
        4.  **Stack Restoration (`Stks`)**: The call stack is cleared, and the saved stack frames (with their 64-bit local variables, 64-bit return addresses, etc.) are pushed back onto it.
        5.  **Register Restoration**: The saved 64-bit Program Counter is loaded into the PC register. Other saved 64-bit VM registers (like SP, frame pointers) are also restored.
        6.  **(Optional) LLM State Restoration (`LLMs`)**: Restore any saved LLM context.
        7.  **Resuming Execution**: Unlike Z-Spec 1.1 `restore` which returns a value, ZM2 `restore` typically does not return to the caller. Instead, after successfully restoring the state, the VM immediately resumes execution from the restored Program Counter, effectively continuing the game from the exact point it was saved. If the restore fails, it branches.

    *   **Considerations for 64-bit**:
        *   **File Size**: Uncompressed dynamic memory can be large, though less so than with 128-bit. Efficient compression for `CMem` is still crucial.
        *   **Atomicity**: Save/restore operations should ideally be atomic to prevent corrupted state if an error occurs midway. This is an interpreter implementation concern.
        *   **Endianness**: Ensure consistent endianness handling when writing/reading multi-word values (like 64-bit addresses or data) in the save file. The header structure already specifies Big Endian for internal memory; this should carry over to save files.
        *   **Address Sizes**: All addresses (PC, SP, frame pointers, pointers within data structures like object trees or property tables if their absolute addresses are somehow part of the dynamic state saved) stored in Quetzal chunks must be 64-bit.

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
        | Z-Code: Parses JSON string|<-----| VM: get_llm_result(handle,      |<-----| Z-Code: Calls get_llm_result         |
        | from result_buffer_addr   |      | result_buffer_addr)             |      | to confirm data.                     |
        | to get structured action, |      | (Confirms data in buffer)       |      |                                      |
        | or handles error          |      | (Returns status_code)           |      |                                      |
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
        | Z-Code: Uses game-ready   |<-----| VM: get_llm_result(handle,      |<-----| Z-Code: Calls get_llm_result         |
        | string from               |      | result_buffer_addr)             |      | to confirm data.                     |
        | result_buffer_addr (e.g., |      | (Confirms data in buffer)       |      |                                      |
        | for printing), or handles |      | (Returns status_code)           |      |                                      |
        | error                     |      |                                 |      |                                      |
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
    *   **BigInt Arithmetic**: Native 64-bit integer types (`uint64_t`, `int64_t` in C++; `u64`, `i64` in Rust; `long` in Java; etc.) are sufficient for all ZM2 operations. BigInt libraries are not required for core ZM2 functionality.
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
    *   **Standard Test Suites**: Adapt existing Z-Machine test suites like "praxix.z5" or "cमेट" (if possible, by recompiling their source for ZM2 or creating analogous tests) to verify overall compliance. This will be challenging due to the 64-bit nature and new opcodes. A more viable long-term approach would be to develop a new, dedicated Z-Machine Mark 2 test suite ('zm2test.zm2' or similar) that specifically targets 64-bit operations, new EXT opcodes, LLM interaction mocks, and the expanded memory model.
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
    *   Develop a simple Z-Machine debugger (memory view, stack view, PC tracing, breakpoints). This debugger should be capable of inspecting 64-bit values and addresses correctly. For LLM opcodes, it should ideally show the status of pending LLM requests and the content of LLM-related buffers.

## 13. References and Further Reading

The following resources and topics provide context or inspiration for the Z-Machine Mark 2 design:

*   **Z-Machine Standards Documents**:
    *   "The Z-Machine Standards Document" by Graham Nelson, Version 1.0 or 1.1. (Primary reference for original Z-Machine architecture).
    *   Quetzal Saved Game Format standard.
*   **LLMs in Interactive Fiction - Research and Concepts**:
    *   Discussions on Large Language Models and Conversational User Interfaces for Interactive Fiction (e.g., academic papers, blog posts exploring this area).
    *   LampGPT: LLM-Enhanced IF Experiences (Specific project or paper, if identifiable).
    *   AI Plays Interactive Fiction GitHub Repository (If this is a known repository, a link would be ideal. E.g., `[AI Plays Interactive Fiction GitHub Repository](link-to-repo)`).
    *   General discussions on "Why can't the parser just be an LLM?".
*   **LLM Implementation and Creative Writing**:
    *   Tutorials or articles on "Creating a Text Adventure Game with ChatGPT" or similar LLMs.
    *   Guides on "How to build a Large Language Model (LLM) to Generate Creative Writing".
    *   Community discussions on "Best realistic story telling LLM?".
*   **Z-Machine Architecture Overviews**:
    *   General overviews and explanations of the Z-machine architecture (e.g., from IFWiki, blogs, or articles).

Developers implementing or extending the Z-Machine Mark 2 are encouraged to consult the original Z-Machine Standards Document for foundational concepts and to stay abreast of current research in LLM integration for interactive experiences.
