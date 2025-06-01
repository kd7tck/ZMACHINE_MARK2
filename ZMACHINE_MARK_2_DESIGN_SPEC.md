# Z-Machine Mark 2 Design Specification

## 1. Overview

**Statement of Purpose:** The Z-Machine Mark 2 aims to revolutionize text-only interactive fiction by blending the classic Z-machine architecture with modern Large Language Model (LLM) capabilities. Its purpose is to create highly immersive, dynamic, and intuitively interactive experiences for players, moving beyond predefined commands and responses.

Z-Machine Mark 2 is a modernized version of the original Z-machine, designed for text-only interactive fiction games. It operates as a 128-bit virtual machine, allowing for large data handling and future scalability. It integrates with a Large Language Model (LLM) via Hugging Face's API for natural language understanding and generation, aiming to create a more immersive and interactive experience.

**Key Components and Interactions:**

The Z-Machine Mark 2 comprises the following core components:

1.  **128-bit Virtual Machine (VM):** The heart of the system, responsible for executing game logic, managing the game state, and handling memory operations. Its 128-bit architecture supports vast address spaces and data sizes.
2.  **Game Story File:** Contains the game's code, data (objects, text, etc.), and initial state, loaded by the VM.
3.  **Large Language Model (LLM):** An external AI model accessed via API (e.g., Hugging Face). It provides:
    *   **Natural Language Understanding (NLU):** Parses player's typed commands into structured actions that the VM can understand.
    *   **Natural Language Generation (NLG):** Creates rich, dynamic, and contextually relevant descriptive text for the game world and events.
4.  **Player Interface:** The means by which the player interacts with the game, typically a text input/output console.

**Interaction Flow:**

*   The **Player** inputs a command through the **Player Interface**.
*   The **VM** receives this input and, along with relevant game state information, sends it to the **LLM** for NLU.
*   The **LLM** parses the input and returns a structured action to the **VM**.
*   The **VM** executes the action, updating the internal **Game State** (e.g., player location, inventory, world events).
*   For generating output, the **VM** may provide prompts or context to the **LLM** for NLG.
*   The **LLM** generates descriptive text, which the **VM** then combines with its own direct output and presents to the **Player** via the **Player Interface**.
This cycle repeats, creating a continuous interactive loop.

## 2. Memory Model

-   **Word Size**: Each word is 128 bits (16 bytes).
-   **Address Space**: 128-bit addressing, providing 2<sup>128</sup> words of addressable memory.
-   **Memory Layout**:
    -   **Header**: Contains metadata (version, release number, memory map).
    -   **Code Section**: Stores game logic routines. Instructions can span multiple 128-bit words.
    -   **Data Sections**: Includes objects, text abbreviations, and dynamic memory.
-   **Byte-Addressable**: Remains byte-addressable, with 128-bit word granularity. Each 128-bit word consists of 16 bytes. Addresses refer to individual bytes, but operations like reads and writes are typically performed on whole words.

**Visual Representation of Memory Layout:**

```
+---------------------------------------+ 0x00000000000000000000000000000000
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
+---------------------------------------+ ... up to 2^128 - 1
| (Potentially Unused Address Space)    |
+---------------------------------------+
```

**Header Structure:**

The header is a fixed-size block at the beginning of the memory (e.g., the first 1024 bytes). All multi-byte values are stored in Big Endian format. Pointers within the header are absolute byte addresses.

| Offset (Bytes) | Size (Bytes) | Field Name                     | Description                                                                                                |
|----------------|--------------|--------------------------------|------------------------------------------------------------------------------------------------------------|
| 0              | 2            | `version`                      | Z-Machine Mark 2 version (e.g., 0x0200 for v2.0).                                                          |
| 2              | 2            | `release_number`               | Release number of the story file.                                                                          |
| 4              | 16           | `story_id`                     | Unique 128-bit identifier for the story.                                                                   |
| 20             | 16           | `checksum`                     | 128-bit checksum of the story file (excluding header beyond this field) for integrity verification.        |
| 36             | 16           | `code_section_start`           | Byte address of the start of the Code Section.                                                             |
| 52             | 16           | `code_section_length`          | Length of the Code Section in bytes.                                                                       |
| 68             | 16           | `static_data_section_start`    | Byte address of the start of the Static Data Section.                                                      |
| 84             | 16           | `static_data_section_length`   | Length of the Static Data Section in bytes.                                                                |
| 100            | 16           | `dynamic_data_section_start`   | Byte address of the start of the Dynamic Data Section (initial heap pointer).                              |
| 116            | 16           | `dynamic_data_section_length`  | Total available length for the Dynamic Data Section in bytes.                                              |
| 132            | 16           | `globals_table_start`          | Byte address of the global variables table within the Static Data Section.                                 |
| 148            | 16           | `object_table_start`           | Byte address of the object table within the Static Data Section.                                           |
| 164            | 16           | `dictionary_start`             | Byte address of the dictionary within the Static Data Section.                                             |
| 180            | 16           | `abbreviation_table_start`     | Byte address of the text abbreviation table within the Static Data Section.                                |
| 196            | 4            | `flags1`                       | Various flags (e.g., transcripting, fixed-pitch font required). (Details TBD)                              |
| 200            | 4            | `flags2`                       | More flags. (Details TBD)                                                                                  |
| 204            | 16           | `llm_api_endpoint_ptr`         | Pointer to a null-terminated string within Static Data for the LLM API endpoint. 0 if not used.            |
| 220            | 16           | `llm_parameters_ptr`           | Pointer to a structure/string within Static Data for LLM parameters (e.g., model name). 0 if not used.   |
| 236            | 788          | `reserved`                     | Reserved for future expansion. Must be initialized to zero.                                                |
| **Total Size** | **1024**     |                                |                                                                                                            |

**Code and Data Section Organization and Access:**

*   **Code Section:**
    *   **Organization**: This section contains the executable game logic. It is primarily a sequence of routines (functions). Each routine consists of a series of instructions. Instructions themselves can vary in length, potentially spanning multiple 128-bit words, especially if they include immediate data or multiple operands. The first part of a routine might define the number of local variables.
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

A Z-Machine Mark 2 story file (typically with a `.zm2` or `.z128` extension) is a binary file composed of several contiguous sections. The structure and content of these sections are based on the memory model loaded by the VM. All multi-byte numerical data within the story file is stored in **Big Endian** format.

**File Structure:**

1.  **Header Section (Fixed Size, e.g., 1024 bytes)**:
    *   **Content**: Identical to the **Header Structure** defined in Section 2 (Memory Model). Contains metadata such as version, story ID, checksum, and pointers (absolute byte offsets from the start of the file) to the start and length of subsequent sections (`code_section_start`, `static_data_section_start`, etc.).
    *   **Encoding**: As per the Header Structure table in Section 2.

2.  **Code Section (Variable Size)**:
    *   **Content**: Contains all executable game logic, encoded as Z-Machine Mark 2 instructions (opcodes and operands). Routines are sequences of these instructions.
    *   **Encoding**: Raw binary sequence of 128-bit opcodes and their operands. Packed addresses (`PADDR`) within instructions are relative to specific bases (e.g., start of Code Section or Static Data Section) and are expanded by the VM at runtime. Details of instruction encoding are in Section 4 (Instruction Set).
    *   **Location in File**: Starts at the byte offset specified by `code_section_start` in the Header and continues for `code_section_length` bytes.

3.  **Static Data Section (Variable Size)**:
    *   **Content**: Contains game data that is generally fixed at compile time. This includes:
        *   **Object Table**: Defines all game objects, their initial attributes (as bitfields), parent-sibling-child relationships (object IDs), and pointers to their property tables. Each object entry has a standardized size.
        *   **Property Tables**: Store default property values for objects. Each property consists of an ID, length, and data.
        *   **Global Variables Initial Values Table**: A table holding the initial 128-bit values for all 240 global variables. These are copied to the dynamic memory area for global variables upon game initialization.
        *   **Dictionary**: A list of ZSCII-encoded words recognized by the traditional parser (if used as a fallback). Each word is typically fixed-length, padded with nulls. Associated with data like word ID or pointers to grammar tokens.
        *   **Text Abbreviation Table**: A table of pointers to frequently used Z-encoded strings. Used by `print_abbrev` opcode.
        *   **Z-Encoded Strings**: All game text (room descriptions, messages, object names, etc.) is stored as ZSCII strings, potentially using Z-Machine string compression techniques (e.g., Huffman-like encoding of character pairs, references to abbreviation table).
        *   **LLM API Configuration Strings**: Null-terminated strings for `llm_api_endpoint_ptr` and `llm_parameters_ptr` if these are used.
        *   Other static arrays, tables, or data structures defined by the game authoring system.
    *   **Encoding**: Binary data. Object IDs are 128-bit. Pointers within this section (e.g., from object to its properties) are absolute byte offsets from the start of the Static Data Section or the start of the file.
    *   **Location in File**: Starts at `static_data_section_start` and continues for `static_data_section_length` bytes.

4.  **Dynamic Data Section (Initial State - Optional in File)**:
    *   **Content**: The story file *may* optionally include an initial snapshot of the Dynamic Data Section. However, more commonly, the Dynamic Data section in memory is initialized by the VM based on data from the Static Data section (e.g., copying global variable initial values, setting up initial object states).
    *   If included, it would represent the desired starting state of mutable game elements if it differs from what can be constructed from the Static Data section alone.
    *   **Encoding**: Raw binary image of the initial dynamic memory.
    *   **Location in File**: If present, its start and length would also be defined in the Header, potentially as `initial_dynamic_data_start` and `initial_dynamic_data_length`. Most often, the `dynamic_data_section_start` in the header refers to where this section will *reside in memory*, and its initial content is constructed by the VM, not bulk-loaded from the file. The `dynamic_data_section_length` in the header defines the *size* of this memory region to be allocated by the VM.

**Encoding of Game Data:**

*   **Scenes/Locations**: Represented as **Objects** in the Object Table. A location object might have properties for its description (a pointer to a Z-encoded string or a prompt for `call_llm_generate`), exits (pointers to other location objects or routines to handle movement), and lists of other objects currently present in it.
*   **Objects (Items, Scenery, etc.)**: Defined in the Object Table. Each object has:
    *   A unique 128-bit ID.
    *   Attributes (e.g., `portable`, `wearable`, `lit`, `container`).
    *   Parent, sibling, and child object ID pointers to define the object tree (e.g., what's in a room, what's in a container).
    *   A pointer to its property table. Properties might include description text, weight, value, or game-specific data.
*   **Characters (NPCs)**: Also represented as **Objects**. NPC-specific logic is handled by routines in the Code Section. Dialogue might be stored as Z-encoded strings, selected by game logic, or generated via `call_llm_generate` using prompts stored as strings in the Static Data section. NPC state (e.g., mood, knowledge) is stored in its object properties or related global variables.
*   **Game Logic (Puzzles, Rules, Event Handling)**: Encoded as routines (functions) in the **Code Section**. These routines manipulate game state (variables, object properties/locations) and interact with the player via I/O opcodes or LLM opcodes.
    *   Example: A puzzle might involve checking if the player has a specific object (`get_parent`) and is in a specific room (global variable for player location), then changing a property on another object (`put_prop`) to signify the puzzle is solved.
*   **Text**: All printable text is stored as Z-encoded strings (see Z-Machine Standard 1.1, Appendix C for ZSCII and string encoding details, adapted for potential Unicode characters via `print_unicode`). Abbreviations are used to save space.

The story file is essentially a serialized form of the static parts of the Z-Machine's memory, plus the executable code. The VM reads this file to populate its memory and then begins execution. The Dynamic Data Section in memory is where the live game state evolves from the initial state defined in the Static Data.

## 4. Instruction Set

-   **Command Size**: Each opcode is 128 bits long.
-   **Opcode Categories**:
    -   **Standard Opcodes**: Adapted from the original Z-machine for 128-bit operands (e.g., arithmetic, control flow, object manipulation, input/output). These opcodes will generally follow the naming and operational conventions of the Z-Machine Standard 1.1, but expanded to handle 128-bit addresses and data. Operands can be constants (small or large), or variables (local, global, or stack).
    -   **Extended Opcodes for LLM Integration**:
        -   `call_llm_parse`: Sends player input and game state to LLM for parsing.
        -   `call_llm_generate`: Requests LLM to generate descriptive text.
-   **Version Support**: Designed for backward compatibility with earlier Z-machine concepts, extended for 128-bit operations. Opcodes will be version-flagged if behavior changes significantly from Z-Spec 1.1.

**Operand Types:**

Operands for opcodes are fetched according to type specifiers. Common types include:
*   **Large Constant (LC):** A 128-bit constant value embedded in the instruction stream.
*   **Small Constant (SC):** An 8-bit or 16-bit constant value embedded in the instruction stream (implementation detail, expanded to 128-bit internally).
*   **Variable (VAR):** A 128-bit value from a variable. The variable is specified by an 8-bit number:
    *   `0x00`: Top of stack (popped).
    *   `0x01-0x0F`: Local variable (L00-L15).
    *   `0x10-0xFF`: Global variable (G00-G239).
*   **Address (ADDR):** A 128-bit byte address. Can be a Large Constant or a Variable.
*   **Packed Address (PADDR):** A compressed address format for routines or strings within the code/static data sections, which is then expanded to a full 128-bit byte address by the VM.

**Standard Opcodes (Illustrative List - Not Exhaustive):**

This list provides examples. A full list will be maintained in a separate ZM2_Opcodes.md document. All operations are on 128-bit values unless specified.

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
    *   `loadb (array_addr, byte_index) -> (result)`: Reads byte from array (zero-extended to 128-bit).
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
    *   `print_unicode (char_code)`: Prints a Unicode character. (Can be VAROP for multiple chars).
    *   `check_unicode (char_code) -> (result)`: Checks if current output stream supports the char.
    *   `sread (text_buffer_addr, parse_buffer_addr)`: Reads player input. (More complex, may involve LLM call implicitly or via `call_llm_parse`).
    *   `aread (text_buffer_addr, parse_buffer_addr, timeout_routine_paddr, timeout_seconds) -> (result)`: Async read with timeout.

**Extended Opcodes for LLM Integration:**

These opcodes facilitate interaction with an external LLM. They require the VM to have networking capabilities and access to the LLM API endpoint and parameters defined in the header.

1.  **`call_llm_parse (input_text_addr, context_data_addr, result_buffer_addr, max_result_len) -> (status_code)`**
    *   **Functionality**: Sends player input text and relevant game context to the LLM for natural language understanding. The LLM is expected to parse the input into a structured format the game can act upon.
    *   **Operands**:
        *   `input_text_addr (ADDR)`: Byte address of a null-terminated Z-encoded string containing the player's raw input.
        *   `context_data_addr (ADDR)`: Byte address of a structured data block (e.g., JSON string or custom binary format) containing relevant game state context. This might include current location, important NPCs, key inventory items, recent events, or specific questions the game wants the LLM to consider.
        *   `result_buffer_addr (ADDR)`: Byte address of a buffer where the LLM's structured response will be written. The format of this response needs to be predefined (e.g., a JSON string with fields like "action", "target1", "target2", "preposition", or a series of VM-understandable action codes).
        *   `max_result_len (LC/VAR)`: Maximum number of bytes to write into `result_buffer_addr`.
    *   **Stores**:
        *   `status_code (VAR)`: A code indicating the success or failure of the API call.
            *   `0`: Success. `result_buffer_addr` contains the parsed data.
            *   `1`: LLM API call failed (network error, API error, timeout).
            *   `2`: LLM could not parse input meaningfully (e.g., returned empty or error structure).
            *   `3`: Result buffer too small (if detectable before writing).
    *   **API Call Parameters (Conceptual - actual format depends on LLM provider)**:
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
    *   **Expected LLM Response (written to `result_buffer_addr`)**:
        *   Example (JSON):
            ```json
            {
              "parsed_action": {
                "verb": "take",
                "noun": "lantern",
                "indirect_noun": null
              },
              "confidence": 0.95
            }
            ```
        *   Alternatively, a sequence of predefined numeric codes that the game logic can interpret.

2.  **`call_llm_generate (prompt_text_addr, context_data_addr, result_buffer_addr, max_result_len, creativity_level) -> (status_code)`**
    *   **Functionality**: Sends a prompt and game context to the LLM to generate descriptive text, dialogue, or other creative content.
    *   **Operands**:
        *   `prompt_text_addr (ADDR)`: Byte address of a null-terminated Z-encoded string containing the prompt for the LLM (e.g., "Describe the old library").
        *   `context_data_addr (ADDR)`: Byte address of a structured data block for context (similar to `call_llm_parse`), helping the LLM generate relevant text.
        *   `result_buffer_addr (ADDR)`: Byte address of a buffer where the LLM's generated Z-encoded text will be written.
        *   `max_result_len (LC/VAR)`: Maximum number of bytes to write into `result_buffer_addr`.
        *   `creativity_level (SC/VAR)`: A value (e.g., 0-100) suggesting the desired creativity/randomness (temperature) for the LLM.
    *   **Stores**:
        *   `status_code (VAR)`:
            *   `0`: Success. `result_buffer_addr` contains the generated Z-encoded text.
            *   `1`: LLM API call failed.
            *   `2`: LLM generated empty or inappropriate content.
            *   `3`: Result buffer too small.
    *   **API Call Parameters (Conceptual)**:
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
    *   **Expected LLM Response (written to `result_buffer_addr`)**:
        *   A Z-encoded string containing the generated text. The VM will need to handle potential encoding issues if the LLM produces characters outside the standard ZSCII set, possibly by converting or filtering them.

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

## 4. LLM Integration

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
    5.  **Making API Calls (from VM via `call_llm_parse` or `call_llm_generate` opcodes)**:
        *   The VM's HTTP client will construct a POST request.
        *   **Headers**:
            *   `Authorization: Bearer YOUR_API_TOKEN`
            *   `Content-Type: application/json`
        *   **Body**: A JSON payload specific to the task (NLU or NLG) and model. See "Data Structures for LLM Communication" below.
    6.  **Handling Responses**:
        *   The VM will parse the JSON response from the API.
        *   For NLU: Extract the structured action.
        *   For NLG: Extract the generated text and convert it to Z-encoded format for use in the game.
        *   Handle potential errors: Network issues, API errors (rate limits, authentication failure), malformed responses, or LLM-specific errors (e.g., content filtering). The `status_code` returned by the LLM opcodes will reflect these outcomes.

-   **Data Structures for LLM Communication**: The `call_llm_parse` and `call_llm_generate` opcodes define operands for `input_text_addr`, `context_data_addr`, and `result_buffer_addr`. The data at these addresses will typically be formatted as JSON strings, though the design allows for other structured binary formats if optimized.

    *   **Sending Information to LLM (NLU - `call_llm_parse`)**:
        *   `input_text_addr`: Points to a Z-encoded string (e.g., "take the red key from the oak table"). This is converted to a plain string for the JSON payload.
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

    *   **Receiving Information from LLM (NLU - `result_buffer_addr`)**:
        *   The LLM is prompted to return a JSON structure.
            ```json
            // Conceptual content written to result_buffer_addr by VM after LLM NLU call
            // This is the JSON part of the Hugging Face API response
            [ // Some models return a list
              {
                "generated_text": "{\"action\": \"take\", \"noun1\": \"small brass key\", \"preposition\": \"from\", \"noun2\": \"oak table\"}"
              }
            ]
            // The VM must parse this to get the inner JSON string, then parse that JSON.
            ```
        *   The Z-Machine game logic then uses this structured data (e.g., verb "take", noun1 "small brass key", etc.) to execute game actions.

    *   **Sending Information to LLM (NLG - `call_llm_generate`)**:
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

## 5. Game State Management

-   **State Representation**: The complete game state is encapsulated within the Z-Machine's memory, primarily within its Dynamic Data Section. This includes:
    *   **Global Variables**: Current values of all 240 global variables (G00-G239), each being a 128-bit word.
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
    *   **Global Variables (G00-G239)**: Stored as an array of 240 128-bit words at a fixed location, typically at the beginning of the Dynamic Data Section or referenced via a pointer from the header. They are accessed directly by opcodes like `store Gx, value` and `load Gx -> (result)`.
    *   **Local Variables (L00-L15)**: Stored on the game's call stack. When a routine is called, space for its local variables (as defined by the routine) is allocated on the stack. They are accessed relative to the current stack frame pointer. Opcodes like `store Lx, value` and `load Lx -> (result)` handle these. Local variables cease to exist when a routine returns.
    *   **Stack Variables (Temporary)**: The top of the stack can be used as a temporary variable, implicitly by some opcodes or explicitly using `push` and `pop`.
    *   **Object Attributes**: Stored as bitfields within each object's entry in the object table. Opcodes like `set_attr`, `clear_attr`, and `test_attr` manipulate these. The object table itself might be in static memory, but a copy of attributes (or the parts that can change) might be in dynamic memory if objects can be created/destroyed or attributes are highly dynamic. For simplicity, we can assume the primary object table attributes are part of the dynamic state that needs saving.
    *   **Object Properties**: Stored in property lists associated with each object. `get_prop`, `put_prop`, `get_prop_addr`, `get_prop_len` opcodes access these. Property data can be variable length. Changes to properties are direct modifications in the Dynamic Data Section (or a dynamic copy of object data).

-   **State Updates**:
    *   Game logic, executed as a sequence of Z-Machine opcodes, directly modifies these variables, object states, and stack contents.
    *   Player actions, once parsed (potentially by the LLM via `call_llm_parse` and then interpreted by game code), trigger routines that change the game state.
    *   For example, `insert_obj` changes parent-child relationships, `put_prop` changes an object's property, `store` changes a variable.
    *   The VM ensures that these changes are persistent within its current memory session until a `save` or `restore` operation.

-   **Serialization and Deserialization (Saving and Loading Games)**:
    The `save` and `restore` opcodes manage this process. The standard format for saved games is Quetzal, but with 128-bit adaptations.

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
                *   For `UMem`: The dynamic memory area is stored uncompressed. Given the potential size with 128-bit words, compression is highly recommended.
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

    *   **Considerations for 128-bit**:
        *   **File Size**: Uncompressed dynamic memory can be very large. Efficient compression for `CMem` is crucial.
        *   **Atomicity**: Save/restore operations should ideally be atomic to prevent corrupted state if an error occurs midway. This is an interpreter implementation concern.
        *   **Endianness**: Ensure consistent endianness handling when writing/reading multi-word values (like 128-bit addresses or data) in the save file. The header structure already specifies Big Endian for internal memory; this should carry over to save files.

## 6. Player Interaction

-   **Input Process**:
    1.  **Raw Input Acquisition**: The Z-Machine interpreter captures the player's typed command from the input stream (e.g., console). This is typically done using an opcode like `sread` or `aread`. The raw text is stored in a designated buffer in Z-Machine memory (`text_buffer_addr`).
    2.  **Pre-processing (Optional but Recommended)**: Before sending the raw input to the LLM, the VM or game logic can perform several pre-processing steps:
        *   **Case Normalization**: Convert input to lowercase to ensure consistency, unless case sensitivity is specifically desired for certain commands or puzzles.
        *   **Whitespace Trimming**: Remove leading/trailing whitespace and normalize multiple internal spaces to single spaces.
        *   **Typo Correction (Basic)**: Implement a simple typo correction algorithm (e.g., Levenshtein distance against a common command vocabulary or game-specific terms) or use a more sophisticated local library if available to the interpreter. This can reduce LLM workload for obvious typos. *This step is optional and adds complexity; relying on the LLM's robustness to typos is also an option.*
        *   **Command History Expansion**: Allow players to use shortcuts like "again" (g) or references to previous commands/objects (e.g., "it", "him", "her", "them"). The VM can expand these locally before sending a more complete command to the LLM. For example, if the player types "take key" then "unlock door with it", "it" would be expanded to "key".
        *   **Profanity/Harmful Content Filtering (Basic Local Check)**: A local pre-filter for obviously problematic content might be implemented before an API call, though the primary responsibility for robust filtering often lies with the LLM service.
        *   **Concatenating with Turn Context**: The pre-processed player command is then combined with contextual information (see `context_data_addr` in `call_llm_parse` opcode details) to form the full prompt for the LLM. This context is crucial for the LLM to understand ambiguous commands.
    3.  **LLM Submission**: The (optionally pre-processed) input text along with the context data is sent to the LLM via the `call_llm_parse` opcode.
    4.  **LLM Response Handling**: The structured response from the LLM (e.g., JSON) is placed in `result_buffer_addr`. Game logic then interprets this structure to take actions. If the LLM fails to parse (returns an error or an "I don't understand" structure), the game might:
        *   Print a generic "I didn't understand that." message.
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
        *   **Source**: Generated by the LLM via the `call_llm_generate` opcode in response to a prompt from the game logic. This is used for:
            *   Dynamic room descriptions.
            *   Detailed object descriptions.
            *   NPC dialogue.
            *   Narrative flavor text, responses to unusual actions, or creative elaborations.
        *   **Formatting and Processing**:
            *   **Conversion to Z-Encoding**: The LLM typically returns plain text (e.g., UTF-8). The VM (or a routine within it) must convert this text to ZSCII for display in the Z-Machine environment. This involves:
                *   Mapping Unicode characters to their ZSCII equivalents.
                *   Handling characters not present in ZSCII (e.g., by transliteration, substitution with a placeholder like '?', or by using Unicode extension opcodes like `print_unicode` if the interpreter supports them and the output stream is configured for it).
                *   Applying Z-Machine string compression/abbreviations if the text is to be stored long-term in dynamic memory (less common for immediate display).
            *   **Content Moderation/Filtering**: Before display, the VM or game logic should ideally apply another layer of filtering to the LLM's output to catch any inappropriate content that might have bypassed the LLM's own safety filters. This is a complex area and might involve local keyword checks or integration with external moderation APIs if available to the interpreter.
            *   **Stylistic Consistency**: The game author should prompt the LLM to maintain a consistent tone and style with the rest of the game. The `creativity_level` parameter in `call_llm_generate` can help, but careful prompt engineering is key.
            *   **Attribution/Distinction (Optional)**: The interpreter or game could optionally prepend a subtle indicator for LLM-generated text if desired for transparency (e.g., a slightly different text color, a small icon, or a style like italics, if the output stream supports it). However, the goal is often seamless integration.
            *   **Line Wrapping and Paging**: The VM's standard output routines will handle line wrapping according to the current window width. If the LLM text is very long, the game might need to explicitly break it into manageable chunks or use "more..." prompts.
        *   **Display**: The processed, Z-encoded text from the LLM is then printed to the game window using standard Z-Machine output opcodes.

    3.  **Combining Outputs**:
        *   Game turns often involve a mix of VM-direct and LLM-generated text. For example:
            *   Player: `> look at table`
            *   VM (direct): `(First, checking to see if you can see the table...)` (internal thought or debug if verbose mode is on)
            *   LLM (generated description, via `call_llm_generate` prompted by game logic): `"The old wooden table is covered in a fine layer of dust. Scratches and a few dark stains mar its surface, hinting at years of use. A half-open drawer on one side suggests something might be inside."`
            *   VM (direct, if a specific game mechanic is tied to it): `You notice a small, almost invisible inscription near the leg.`
        *   The game author orchestrates this sequence of calls to `print`, `call_llm_generate`, etc., to produce the final output for the turn.

## 7. Workflow (Game Loop)

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
    *   **VM Action**: The game may execute initial Z-code routines to print a welcome message, title screen, or the first room description. This might involve `print`, `print_addr`, or even `call_llm_generate` if the initial description is dynamic.
    *   **Interaction (I/O, LLM, Memory)**: Output is sent to the player. LLM may be invoked.

**B. Main Game Loop (Repeats Each Turn):**

1.  **Display Prompt & Read Player Input**:
    *   **VM Action**: The VM displays the input prompt (e.g., "> "). It then executes an input opcode (e.g., `sread` or `aread`).
    *   **Interaction (I/O)**: The interpreter waits for player to type a command and press Enter. The raw command string is read from the input stream.
    *   **Interaction (Memory)**: The raw command is stored in the `text_buffer_addr` in the Dynamic Data Section. Any specified parse buffer (`parse_buffer_addr`) is also prepared or cleared.

2.  **Input Pre-processing & LLM Parse Call (Optional Fallback to Traditional Parse)**:
    *   **VM Action**: Game logic (Z-code) may perform pre-processing on the raw input text (case normalization, typo correction, command history expansion as detailed in Section 6).
    *   **VM Action**: The game logic formulates context data (player location, visible objects, etc.) to assist the LLM.
    *   **Interaction (Memory)**: Pre-processed text and context data are prepared in memory buffers.
    *   **VM Action**: The `call_llm_parse` opcode is executed.
        *   **Interaction (LLM)**: The VM (via its interpreter's HTTP client) sends the input text and context data to the configured LLM API.
        *   **Interaction (LLM)**: The VM waits for a response from the LLM.
        *   **Interaction (Memory)**: The LLM's structured response (e.g., JSON) is written into `result_buffer_addr`. The status of the call is returned in a variable.
    *   **Flowchart: `call_llm_parse` Interaction**
        ```
        +---------------------------+      +-------------------------+      +----------------------------+
        | Z-Code: Prepare input &   |----->| VM: call_llm_parse      |----->| LLM Service (API)          |
        | context data in memory    |      | (Sends HTTP Request)    |      | (Processes Input + Context)|
        +---------------------------+      +-------------------------+      +----------------------------+
                                                 ^      |                             |
                                                 |      | (Structured Data, e.g. JSON)|
                                                 |      V                             |
        +---------------------------+      +-------------------------+      +----------------------------+
        | Z-Code: Process LLM       |<-----| VM: Writes result_buffer|      | LLM Service (API)          |
        | response or handle error  |      | & status code           |<-----| (Returns HTTP Response)    |
        +---------------------------+      +-------------------------+      +----------------------------+
        ```
    *   **VM Action (Error/Fallback Handling for Parsing)**:
        *   If `call_llm_parse` status indicates success: The game logic uses the structured data from `result_buffer_addr`.
        *   If `call_llm_parse` status indicates failure (LLM error, network error, unparseable):
            *   The game logic MAY attempt a traditional Z-Machine dictionary-based parse on the original input (using `read` with dictionary, `tokenise`).
            *   The game logic MAY print an error message (e.g., "I didn't understand that.") or ask for clarification.

3.  **Execute Action & Update Game State**:
    *   **VM Action**: Based on the (successfully) parsed action (either from LLM or traditional parser), the VM executes the corresponding game logic routines (Z-code).
    *   **Interaction (CPU, Memory)**: The PC moves through instructions in the Code Section. Opcodes read from and write to global variables, object properties/attributes, the stack, and other parts of the Dynamic Data Section. This is where the game world changes.
        *   Examples: `set_attr` to mark a key as taken, `put_prop` to change an NPC's state, `insert_obj` to move the player to a new room.
    *   **Interaction (Memory)**: The game state (variables, object states, player location, etc.) is updated in the Dynamic Data Section.

4.  **Generate Text Output**:
    *   **VM Action**: Game logic determines what text to display. This can be a combination of:
        *   Directly printing pre-defined strings (e.g., "You open the door.").
        *   Printing object names or properties.
        *   Calling `call_llm_generate` for dynamic descriptions, NPC dialogue, etc.
    *   **Flowchart: `call_llm_generate` Interaction (Similar to `call_llm_parse`)**
        ```
        +---------------------------+      +-------------------------+      +----------------------------+
        | Z-Code: Prepare prompt &  |----->| VM: call_llm_generate   |----->| LLM Service (API)          |
        | context data in memory    |      | (Sends HTTP Request)    |      | (Processes Prompt+Context) |
        +---------------------------+      +-------------------------+      +----------------------------+
                                                 ^      |                             |
                                                 |      | (Generated Text)            |
                                                 |      V                             |
        +---------------------------+      +-------------------------+      +----------------------------+
        | Z-Code: Process LLM text  |<-----| VM: Writes result_buffer|      | LLM Service (API)          |
        | (Z-encode, filter), print |      | & status code           |<-----| (Returns HTTP Response)    |
        +---------------------------+      +-------------------------+      +----------------------------+
        ```
    *   **Interaction (Memory)**: LLM-generated text is processed (Z-encoding, filtering) and stored temporarily if needed.
    *   **Interaction (I/O)**: All output text (VM-direct and LLM-generated) is sent to the player's screen, formatted according to Z-Machine rules (line wrapping, styles).

5.  **Check for Game End Conditions**:
    *   **VM Action**: Game logic checks if any end-game conditions have been met (e.g., player death, puzzle solved, `quit` command).
    *   **Interaction (Memory)**: Reads variables or game state flags.
    *   If game ends: Print final message, score, and halt or offer restart. Loop terminates.
    *   If game continues: Increment turn counter (if used), run any end-of-turn daemons/rules.

6.  **Loop**: Return to Step B.1.

-   **Error Handling within the Loop**:
    -   **LLM Errors**: As described in Step B.2 and B.4, failures in LLM communication or content generation are handled by game logic, potentially falling back to simpler responses or traditional parsing. The `status_code` from `call_llm_parse` and `call_llm_generate` is crucial.
    -   **VM Errors**: Invalid opcodes, memory access violations, stack overflows, etc., will typically halt the VM with an error message (see Section 3 on Instruction Set error handling). These are usually indicative of bugs in the story file Z-code or the interpreter itself.
    -   **Input Errors**: The game should be robust to nonsensical player input, relying on the LLM's parsing capabilities or fallback mechanisms.
-   **Asynchronous Operations (Consideration for `aread`)**:
    -   If `aread` (asynchronous read with timeout) is used, the loop structure might be more complex, involving event checking for input arrival or timeout events. The fundamental sequence of parse-execute-output for a command still applies once input is received.

## 8. Advantages

-   **Enhanced Interactivity**: More natural and flexible player inputs.
-   **Dynamic Text Generation**: Varied and engaging descriptions.
-   **Future-Proofing**: 128-bit architecture for scalability.
-   **Portability**: Runs on any platform with a compatible interpreter, plus AI features.

## 9. Challenges and Considerations

-   **LLM Reliability**: Potential for inconsistent outputs; requires safeguards.
-   **Performance**: API calls may introduce latency.
-   **Training and Fine-Tuning**: LLM needs domain-specific training.
-   **Complexity**: 128-bit architecture might be overkill for text-based games.

## 10. Implementation Notes

-   **Choosing an LLM**: Use a pre-trained model from Hugging Face, fine-tuned on interactive fiction data.
-   **API Integration**: Leverage Hugging Face's Inference API.
-   **Game Development Tools**: Extend tools like Inform for 128-bit operations and LLM integration.
-   **Testing**: Test with existing games for compatibility and performance.

## 10. Building the VM

This section outlines considerations for developing a Z-Machine Mark 2 Virtual Machine (interpreter).

**10.1. Development Environment Setup**

*   **Programming Language**:
    *   **Recommended**: Rust, C++, Go. These languages offer good performance, memory safety (especially Rust), and control over low-level operations suitable for VM development.
    *   **Alternatives**: Python (with C extensions like CFFI for performance-critical parts), Java, C#. Slower for core emulation but potentially faster for UI and tooling.
*   **Core Libraries/Modules**:
    *   **Memory Management**: Custom allocators might be needed if dealing with extremely large dynamic memory, but standard library features for byte array/vector manipulation are a starting point. For 128-bit numbers, a BigInt library will be essential if the chosen language doesn't natively support `u128`/`i128`.
    *   **File I/O**: Standard libraries for reading story files and writing save game files.
    *   **User Interface (I/O)**:
        *   Console: Libraries like `ncurses` (C/C++), `termion`/`crossterm` (Rust), or standard input/output for basic text interaction.
        *   GUI: Optional, but frameworks like Qt (C++), GTK (C/various), Electron (JS/web tech), or language-specific UI libraries (e.g., Java Swing/FX, Python Tkinter/PyQt) could be used for a richer experience.
    *   **Networking (for LLM Integration)**:
        *   HTTP Client: Libraries like `libcurl` (C/C++), `reqwest` (Rust), `requests` (Python), Apache HttpClient (Java) to make API calls to Hugging Face or other LLM providers.
        *   JSON Parsing: Libraries like `serde_json` (Rust), `nlohmann/json` (C++), `json.hpp` (C++), `Jackson`/`Gson` (Java), `json` (Python) to construct and parse JSON payloads for LLM communication.
    *   **BigInt Arithmetic**: For handling 128-bit numbers if not natively supported (e.g. `num-bigint` for Rust if target doesn't fully support u128, or equivalent for other languages).
*   **Development Tools**:
    *   **Compiler/Interpreter**: Specific to the chosen language (e.g., `rustc`, `gcc`/`clang`, `go build`, Python interpreter).
    *   **Build System**: `cargo` (Rust), `cmake`/`make` (C++), `go build` system, `pip`/`poetry` (Python).
    *   **Debugger**: `gdb`, `lldb`, or IDE-integrated debuggers.
    *   **Version Control**: Git.
    *   **Testing Framework**: `Criterion`/`Bencher` (Rust, for benchmarking), Google Test (C++), standard library testing tools (Go, Python).

**10.2. Phased Approach to Building the VM**

1.  **Phase 1: Core Z-Machine CPU and Memory Model (No I/O, No LLM)**
    *   **Memory Implementation**: Implement the 128-bit memory model (Header, Code, Static, Dynamic sections). Ability to load a story file's header, code, and static data into memory. Handle byte/word addressing (128-bit words). Implement Big Endian for all multi-byte values.
    *   **CPU Skeleton**: Implement the main execution loop (fetch-decode-execute). Program Counter (PC), Stack Pointer (SP).
    *   **Basic Opcodes**: Implement a small subset of essential opcodes:
        *   Stack operations: `push`, `pop`.
        *   Control flow: `jump`, `jz`, `call` (without argument handling initially), `ret`.
        *   Arithmetic: `add`, `sub` (on 128-bit values).
        *   Variable access: `store` (global/local), `load` (global/local).
        *   `nop`.
    *   **Story File Loader**: Basic loader for story file structure (reading header, sections).

2.  **Phase 2: Standard Opcode Implementation & Basic I/O**
    *   **Expand Opcodes**: Systematically implement all standard Z-Machine opcodes (0OP, 1OP, 2OP, VAROP) as defined in this specification and any accompanying opcode documents, adapting them for 128-bit operands and addresses.
    *   **Text Processing**: Implement ZSCII decoding, abbreviation handling, and dictionary lookup for traditional parsing.
    *   **Basic Console I/O**: Implement `print`, `print_addr`, `newline`, `sread` (basic line input without LLM for now), `aread` (if implementing timeouts).
    *   **Object Model**: Implement `get_prop`, `put_prop`, `get_parent`, `get_child`, `get_sibling`, `set_attr`, `clear_attr`, `test_attr`, `insert_obj`, `remove_obj`.
    *   **Save/Restore (Quetzal Adaptation)**: Implement `save` and `restore` opcodes using the 128-bit adapted Quetzal format. This is a significant sub-project.

3.  **Phase 3: Advanced Z-Machine Features**
    *   **Advanced I/O**: Implement features like text styles, color, sound (if specified).
    *   **Unicode Support**: Implement `print_unicode`, `check_unicode`.
    *   **Full Z-Machine v1-v8 Compatibility (Conceptual Mapping)**: Ensure conceptual behaviors from earlier Z-machine versions are correctly mapped or extended where appropriate for the 128-bit architecture.

4.  **Phase 4: LLM Integration**
    *   **Networking Subsystem**: Integrate HTTP client and JSON parsing libraries.
    *   **LLM Opcodes**: Implement `call_llm_parse` and `call_llm_generate`.
        *   Securely manage API keys (not hardcoded, configurable by the user of the VM).
        *   Construct JSON payloads as per LLM API requirements.
        *   Handle API responses, including errors, timeouts, and status codes.
        *   Convert LLM text output to Z-encoded strings for game use.
    *   **Contextual Data Assembly**: Develop routines (either in VM or example Z-code) for assembling the `context_data_addr` payload for LLM calls.

5.  **Phase 5: Optimization and Refinement**
    *   **Performance Profiling**: Identify and optimize bottlenecks in the execution loop or specific opcodes.
    *   **Memory Optimization**: Reduce interpreter memory footprint if necessary.
    *   **Error Handling**: Robust error reporting for both VM internal errors and game Z-code errors.
    *   **User Interface Enhancements**: Improve UI/UX if a graphical interface is used.

**10.3. Testing Guidelines**

*   **Unit Testing (Per Component/Opcode)**:
    *   **CPU/Opcodes**: For each opcode, write specific unit tests with known inputs and expected outputs/state changes.
        *   Example: For `add G0, 5 -> L0`, set G0, execute, check L0 and flags.
        *   Test edge cases: zero, negative numbers (if applicable), max values, overflow (if defined behavior).
    *   **Memory Management**: Test allocation, reads, writes, boundary conditions, endianness.
    *   **Story File Loading**: Test loading valid and corrupted story files (header checks, checksums).
    *   **Save/Restore**: Create known game states, save, restore, and verify that the state (memory, stack, PC) is identical. Test compatibility checks.
    *   **LLM Opcodes**:
        *   Mock LLM Service: Create a mock HTTP server that simulates LLM API responses (success, various errors, different content types). Test the VM's ability to correctly call the API and handle responses from the mock.
        *   Test parameter passing (input text, context, result buffer handling, creativity levels).
        *   Test status code interpretation.
*   **Integration Testing (Modules Working Together)**:
    *   **Core Logic**: Test sequences of opcodes that perform a common game task (e.g., moving an object, player movement, simple conversation).
    *   **I/O and Game Logic**: Test that `sread` correctly captures input, game logic processes it, and `print` displays the correct output.
    *   **LLM and Game Logic**:
        *   Use a simple story file that makes `call_llm_parse` calls. Provide controlled input and verify that the parsed output (from a mock or real LLM) is correctly interpreted by the game logic.
        *   Use a simple story file that makes `call_llm_generate` calls with specific prompts. Verify that the returned text is correctly processed and displayed.
*   **System/Acceptance Testing (Using Test Story Files)**:
    *   **Standard Test Suites**: Adapt existing Z-Machine test suites like "praxix.z5" or "cमेट" (if possible, by recompiling their source for ZM2 or creating analogous tests) to verify overall compliance. This will be challenging due to the 128-bit nature and new opcodes.
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

## 11. Key Citations

-   Z-Machine Standards Document Overview
-   Large Language Models and Conversational User Interfaces for Interactive Fiction
-   LampGPT: LLM-Enhanced IF Experiences
-   AI Plays Interactive Fiction GitHub Repository
-   Why can't the parser just be an LLM?
-   Creating a Text Adventure Game with ChatGPT
-   How to build a Large Language Model (LLM) to Generate Creative Writing
-   Best realistic story telling LLM?
-   Overview of Z-machine architecture
