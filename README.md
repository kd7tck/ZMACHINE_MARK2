# Z-Machine Mark 2 Design Specification

## 1. Overview

The Z-Machine Mark 2 (ZM2) is a modernized, 64-bit virtual machine designed to push the boundaries of text-only interactive fiction. Its core purpose is to create highly immersive, dynamic, and intuitively interactive experiences by seamlessly blending the traditional Z-machine architecture with the power of Large Language Models (LLMs). This moves beyond rigid, predefined commands and responses, enabling more natural player interaction and richer narrative generation.

Key Components and Interactions:

The ZM2 ecosystem is built upon four core components:

64-bit Virtual Machine (VM): The central processing unit, responsible for executing game logic, managing the game state, and handling all memory operations. Its 64-bit architecture allows for significantly larger address spaces and data handling.

Game Story File: A binary file containing the game's compiled code, static data (objects, text), and initial state, which is loaded and executed by the VM.

Large Language Model (LLM): An external AI service (e.g., accessed via Hugging Face's API) that provides:

Natural Language Understanding (NLU): Translates complex player commands into structured actions the VM can process.

Natural Language Generation (NLG): Creates dynamic, contextually relevant, and varied descriptive text for the game world.

Player Interface: The means by which the player interacts with the game, typically a text-based console for input and output.

Interaction Flow:

The interaction in ZM2 is a continuous loop, often involving asynchronous LLM calls:

The Player inputs a command.

The VM receives this input. Game logic can initiate an asynchronous NLU request to the LLM (via start_llm_parse), providing relevant game state as context.

While the LLM request is pending, the game can display a waiting indicator or continue non-blocking operations. The game periodically polls the request status using check_llm_status.

Once the LLM completes the NLU task, its parsed structured action is retrieved by the VM (via get_llm_result).

The VM executes this action, updating the internal Game State.

For generating output, the VM may similarly initiate asynchronous NLG requests to the LLM (via start_llm_generate), providing prompts and context.

Once the LLM generates descriptive text, the VM retrieves it and combines it with its own direct output, presenting the full narrative to the Player.

This asynchronous model is crucial for maintaining a responsive user experience despite potential LLM latency.

## Repository Contents

This repository contains the design specification and the Rust implementation of the Z-Machine Mark 2 Virtual Machine.

*   **`ZMACHINE_MARK_2_DESIGN_SPEC.md`**: The detailed design document for the Z-Machine Mark 2.
*   **`zm2_vm/`**: A Rust Cargo project that implements the Z-Machine Mark 2 virtual machine as a library. This crate will contain the core logic for the VM, including memory management, opcode execution, and LLM integration, as outlined in the design specification.

Further details about the `zm2_vm` crate can be found within its own documentation once developed.
