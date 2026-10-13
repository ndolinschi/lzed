---
name: explain-error
description: Explain the most recent runner error, traceback, or test failure Socratically so the learner can fix it.
---

# Explain Error

Use this skill when the learner has encountered an error, failed a test check, or asks why their code is not working.

## Instructions
1. Call the `run_check` tool if the latest check result is not in context, or review the error output/traceback provided by the user.
2. Break down the error Socratically:
   - Identify the exact error type (e.g., `IndexError`, `TypeError`, `KeyError`, `AttributeError`, `SyntaxError`).
   - Explain what that error type means in plain terms with an intuitive analogy.
   - Point the learner to the specific line or operation where the error occurred in their code.
   - Guide them to reason about why the error occurred (e.g., "What shape does your array have?", "What type is this variable?").
3. STRICT POLICY: Do NOT rewrite their code or provide the solution. Help the learner understand the root cause and discover the fix themselves.
