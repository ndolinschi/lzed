---
name: generate-track
description: Conduct an interactive AI interview to generate a customized LearnZed learning track.
---

# Generate Learning Track

Use this skill when the user wants to generate a new LearnZed track or learning curriculum.

## Workflow

### Phase 1: Needs Assessment & Interview
Engage the user in a short, friendly interview to gather their learning parameters:
1. **Goals & Subject**: What topic, technology, or library do they want to learn? (e.g. FastAPI, Algorithms, PyTorch, SQL)
2. **Current Level**: What is their background? (Absolute beginner, intermediate programmer learning a new stack, advanced)
3. **Language & Stack**: Default to Python (`python`), or confirm if they want another language.
4. **Scope & Time Budget**: How many exercises or chapters? (Default: 1-2 chapters, 2-4 exercises total for a focused learning unit)

If the user has already provided these details in their message, you do not need to ask redundant questions—simply acknowledge their choices and proceed to curriculum proposal.

### Phase 2: Curriculum Proposal
Present a brief outline of the proposed track:
- Track Title & Slug ID (lowercase hyphenated, e.g. `fastapi-foundations`)
- Chapters and exercises
- Concepts covered in each exercise

Ask the user: *"Does this syllabus look good, or would you like to adjust anything before I build the track?"*

### Phase 3: Track Generation
When the user confirms (or says "generate", "build it", "yes", etc.):
Call the `generate_track` tool with the structured track schema:
- `id`: unique slug ID
- `title`: human readable title
- `description`: clear description of what the track teaches
- `language`: programming language (default "python")
- `chapters`: list of chapters with `dir_name`, `title`, and `exercises`.
- For each exercise:
  - `dir_name`: slug like `01-hello-api`
  - `title`: title of the exercise
  - `context`: markdown explanation of the concept
  - `instructions`: clear step-by-step instructions for the learner
  - `starter_code`: skeleton code for the learner with `# TODO:` comments
  - `solution_code`: working reference code that passes the checks
  - `check_code`: checker assertions using learn runner helpers (e.g. `check_output_contains(...)`, `test(...)`, `check_function_called(...)`)
  - `hints`: list of pedagogical hints
  - `xp`: XP reward (default 100)
  - `hint_cost`: hint cost in XP (default 30)

### Phase 4: Delivery
Once the `generate_track` tool successfully runs and validates:
- Inform the user that the track was compiled, tested against the checker, and registered in their LearnZed Track Picker.
- Provide instructions on how to start the track from the Track Picker.
