`setup.py` gives you `template` (containing `{{TICKET}}`), a `ticket` string and a mock `reply` from Claude.

1. Set `prompt` to `template` with `{{TICKET}}` replaced by `ticket`.
2. Set `summary` to the text inside `<summary>...</summary>` in `reply`, stripped of surrounding whitespace.
