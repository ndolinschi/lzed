---
name: generate-skill
description: Conduct an interactive AI interview to generate a targeted LearnZed skill track awarding a badge.
---

# Generate Skill Track

Use this skill when the user wants to practice a specific skill and earn a profile badge upon completion.

## Workflow

### Phase 1: Skill Assessment
Ask the user what specific capability or skill badge they wish to earn:
- Target skill (e.g. `fastapi-routes`, `pandas-dataframes`, `regex-matching`, `binary-trees`)
- Target difficulty and prerequisite knowledge
- Language (defaults to Python)

### Phase 2: Skill Proposal
Propose a concise, single-chapter skill track (typically 2-4 hands-on exercises) awarding the specified skill badge:
- Badge name (e.g. `fastapi-routes`)
- Track Title (e.g. `FastAPI Routes & Endpoints`)
- Exercises targeting the core concepts of this skill

Confirm with the learner: *"Ready to create this skill track?"*

### Phase 3: Track & Skill Generation
Call the `generate_track` tool with:
- `id`: slug ID for the track
- `title`: title of the track
- `description`: overview of the skill covered
- `skill`: the exact name of the skill badge awarded upon 100% completion (e.g. `fastapi-routes`)
- `language`: language (default "python")
- `chapters`: single chapter containing the targeted exercises
- Exercises: complete with `context`, `instructions`, `starter_code`, `solution_code`, `check_code`, and `hints`.

### Phase 4: Delivery
Notify the user that the skill track has been verified and registered. Explain that completing all exercises in this track will award the badge to their LearnZed profile.
