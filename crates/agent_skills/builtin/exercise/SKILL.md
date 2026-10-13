---
name: exercise
description: Show the current exercise instructions, context, and goals for this LearnZed track.
---

# Current Exercise

Use this skill when the user wants to understand the active exercise, read its instructions, or get oriented in their LearnZed track.

## Instructions
1. Call the `get_current_exercise` tool to retrieve the active exercise's metadata, title, instructions, context, and user file path.
2. Present the exercise goals clearly to the learner:
   - Outline the objective and any key concepts or functions to use.
   - Mention the user code file they should edit (e.g. `script.py`).
   - Mention any hints available if they need help later.
3. STRICT POLICY: Never reveal or write out the complete solution code.
4. Encourage the learner to take the first step and ask any clarifying questions about the instructions or concepts.
