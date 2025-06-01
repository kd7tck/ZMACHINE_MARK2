# ZMACHINE_MARK2

# Z-Machine Mark 2 Design Specification

## 1. Overview

Z-Machine Mark 2 is a modernized version of the original Z-machine, designed for text-only interactive fiction games. It operates as a 128-bit virtual machine, allowing for large data handling and future scalability. It integrates with a Large Language Model (LLM) via Hugging Face's API for natural language understanding and generation, aiming to create a more immersive and interactive experience.

## 2. Memory Model

-   **Word Size**: Each word is 128 bits (16 bytes).
-   **Address Space**: 128-bit addressing, providing 2<sup>128</sup> words of addressable memory.
-   **Memory Layout**:
    -   **Header**: Contains metadata (version, release number, memory map).
    -   **Code Section**: Stores game logic routines. Instructions can span multiple 128-bit words.
    -   **Data Sections**: Includes objects, text abbreviations, and dynamic memory.
-   **Byte-Addressable**: Remains byte-addressable, with 128-bit word granularity.

## 3. Instruction Set

-   **Command Size**: Each opcode is 128 bits long.
-   **Opcode Categories**:
    -   **Standard Opcodes**: Adapted from the original Z-machine for 128-bit operands (e.g., arithmetic, control flow, object manipulation, input/output).
    -   **Extended Opcodes for LLM Integration**:
        -   `call_llm_parse`: Sends player input and game state to LLM for parsing.
        -   `call_llm_generate`: Requests LLM to generate descriptive text.
-   **Version Support**: Designed for backward compatibility with earlier Z-machine concepts, extended for 128-bit operations.

## 4. LLM Integration

-   **Role of the LLM**:
    -   **Natural Language Understanding (NLU)**: Parses player inputs into structured actions.
    -   **Natural Language Generation (NLG)**: Generates dynamic and contextually appropriate text outputs.
-   **Interface with Hugging Face**:
    -   Uses API calls to access pre-trained or fine-tuned LLMs (e.g., GPT-3.5, Llama 2).
    -   For Parsing: Sends player input and game state summary; LLM returns a structured action.
    -   For Generation: Sends a prompt or template; LLM returns detailed descriptive text.
-   **Training the LLM**:
    -   The LLM is fine-tuned on a dataset of interactive fiction commands and responses.

## 5. Game State Management

-   **State Representation**: Game state (variables, objects, locations, player status) maintained in 128-bit words.
-   **State Updates**: The VM executes game logic based on parsed player actions, updating the state accordingly and ensuring persistence.

## 6. Player Interaction

-   **Input**: Players enter natural language commands, sent to the LLM for parsing.
-   **Output**: The VM generates text output, including direct responses and LLM-generated descriptions.

## 7. Workflow (Game Loop)

1.  **Initialization**: VM loads a story file (code, data, initial state) into memory.
2.  **Player Input**: Player enters a command; VM sends it to LLM with current game state.
3.  **Action Execution**: LLM returns a structured action; VM executes it, updating game state.
4.  **Text Output**: VM generates responses, possibly calling LLM for enhanced descriptions.
5.  **State Update**: VM updates game state based on executed actions.
-   **LLM Interaction Details**:
    -   Parsing: Input (player command, game state summary) -> Output (parsed action).
    -   Generation: Input (prompt/context) -> Output (generated text).
-   **Error Handling**: Fallback to traditional parser or request clarification if LLM fails. Mitigate inconsistent LLM outputs with predefined templates.

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
