Anthropic's guide *Building effective agents* separates two kinds of system:

- **Workflows**, where LLM calls are orchestrated through predefined code paths.
- **Agents**, where the LLM dynamically directs its own process and tool use.

The guide's advice is to start with the simplest solution, often a single well-prompted call, and add complexity only when it clearly helps. Common workflow patterns are **prompt chaining**, **routing**, **parallelization**, **orchestrator-workers** and **evaluator-optimizer**.
