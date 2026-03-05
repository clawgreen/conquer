# Error Patterns Log

## Resolved

### Buffer overflow: LOGIN > LEADERLTH
- **Date**: 2026-03-04
- **Symptom**: SIGTRAP (signal 5, exit 133) during headless makeworld
- **Cause**: `strcpy(ntn[0].leader, LOGIN)` where LOGIN="greenclawdbot" (14 chars) but LEADERLTH=9
- **Fix**: Use `strncpy(ntn[0].leader, "god", LEADERLTH)` in headless path
- **Lesson**: Always use strncpy with LEADERLTH for leader fields

### Curses crash in headless createworld
- **Date**: 2026-03-04
- **Symptom**: Curses functions (mvaddstr, etc.) crash when initscr() was never called
- **Cause**: Headless mode skipped initscr() but createworld() still calls curses output functions
- **Fix**: In newinit(), use `newterm(NULL, fnull, fnull_in)` with /dev/null to create a dummy curses context
- **Lesson**: If code paths call curses functions, you need SOME curses context even in headless mode

### macOS: no -lcrypt
- **Date**: 2026-03-04
- **Symptom**: `ld: library 'crypt' not found` when building oracle
- **Cause**: macOS provides crypt() in libc, not a separate library
- **Fix**: Only use `-lcurses` on macOS (no `-lcrypt`)

### getpwnam(LOGIN)->pw_uid check fails
- **Date**: 2026-03-04
- **Symptom**: "Sorry -- you can not create a world" when running as bot user
- **Cause**: UID check requires LOGIN user to match real user
- **Fix**: Skip UID check when CONQUER_HEADLESS=1
