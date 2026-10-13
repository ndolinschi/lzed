---
name: check
description: Run automated tests/checks on the current exercise and provide helpful, Socratic feedback on the results.
---

# Run Exercise Check

Use this skill when the user wants to check or verify their code, or after typing `/check`.

## Instructions
1. Call the `run_check` tool to execute the automated checker against the learner's code.
2. If `passed: true`:
   - Warmly congratulate the learner!
   - Highlight the XP earned and any skills unlocked.
   - Summarize the key concepts they just mastered.
   - Suggest advancing to the next exercise or exploring further.
3. If `passed: false`:
   - Do NOT give away the solution or write the fixed code for the learner.
   - Review any failing checks and error messages Socratically.
   - Ask guiding questions to help the learner spot the discrepancy between what the test expected and what their code produced.
   - Suggest printing intermediate variables or checking indexing/types.
