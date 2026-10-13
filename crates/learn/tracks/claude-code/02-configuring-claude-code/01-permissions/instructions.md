Write `settings.json` (valid JSON) with `permissions` where:
- `allow` contains `"Bash(npm run lint)"` and `"Bash(npm run test:*)"`;
- `deny` contains `"Read(./.env)"` and `"Read(./secrets/**)"`.
