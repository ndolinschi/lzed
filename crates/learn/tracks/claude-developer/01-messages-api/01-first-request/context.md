A request to `POST /v1/messages` needs a `model`, `max_tokens` and a list of `messages`. Each message has a `role`, either `"user"` or `"assistant"`, and `content`. Instructions for the whole conversation go in the top-level **`system`** parameter. There is no `"system"` role inside `messages`.

With the Python SDK it looks like this: `client.messages.create(**request)`. In this exercise you only build the request, so no network or API key is needed.
