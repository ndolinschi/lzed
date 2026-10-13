Claude is offered as a family of models. Larger models such as **Opus** are the most capable. **Sonnet** balances capability and speed, and **Haiku** is the fastest and cheapest. Pick the smallest model that reliably does the job.

Usage is measured in **tokens**, and input and output tokens are priced separately. Everything the model sees in one request is the **context window**: the system prompt, the conversation so far, tool definitions and tool results.

The Messages API is **stateless**. Claude does not remember earlier requests, so your application has to send the relevant conversation history every time.
