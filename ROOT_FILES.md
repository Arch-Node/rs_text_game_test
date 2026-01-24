# Root Directory Files - Quick Reference

## 📄 Core Files

### `Cargo.toml`
**Status:** ✅ Active  
**Purpose:** Rust project configuration with multiple binaries and features  
**Key Features:**
- `default`: No features (single-player only)
- `spacetimedb-http`: Terminal client with HTTP API
- `spacetimedb-sdk-client`: Terminal client with SDK (vendored OpenSSL)
- `signal-client`: Signal Messenger bot with webhook server
- Binaries: `rust_game_test` (main), `terminal_client`, `sdk_client`, `signal_bot`

### `Cargo.lock`
**Status:** ✅ Active (auto-generated)  
**Purpose:** Dependency version lock file  
**Action:** Keep for reproducible builds

## 📚 Documentation

### `README.md`
**Status:** ✅ Active (just updated)  
**Purpose:** Main project documentation  
**Contents:**
- Project overview and features
- SpacetimeDB backend info
- Build instructions for all three clients
- Architecture highlights
- Development status

### `TODO.md`
**Status:** ✅ Active  
**Purpose:** Complete 23-task development roadmap  
**Recent Updates:** Marked Room table and SDK generation complete

### `ROOT_FILES.md`
**Status:** ✅ Active (this file)  
**Purpose:** Quick reference for all root files

### `bot_config.toml.example`
**Status:** ✅ Active (example file)  
**Purpose:** Example TOML configuration for Signal bot (not yet implemented)

> **📝 Note:** Module-specific documentation has been moved into respective directories:
> - SDK docs: [src/spacetimedb_client/SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md)
> - Terminal client docs: [src/terminal_client/TERMINAL_CLIENT_STATUS.md](src/terminal_client/TERMINAL_CLIENT_STATUS.md)
> - Signal client docs: [src/signal_client/](src/signal_client/) (5 files)

## 🧪 Testing

### `tests/test_backend.sh`
**Status:** ✅ Active  
**Purpose:** Automated backend health check  
**Tests:**
- Server status (localhost:3000/health)
- Room count (should be 10)
- connect_session reducer
- Session creation

**Usage:**
```bash
chmod +x tests/test_backend.sh
./tests/test_backend.sh
```

### `tests/test_signal_bot.sh`
**Status:** ✅ Active  
**Purpose:** Automated Signal bot validation  
**Tests:**
- signal-cli-rest-api status
- SpacetimeDB running
- Configuration validation
- Bot build test

**Usage:**
```bash
chmod +x tests/test_signal_bot.sh
./tests/test_signal_bot.sh
```

## 📦 Data Files

### `rooms.json`
**Status:** ⚠️ LEGACY (marked deprecated)  
**Purpose:** Room data for single-player game  
**Current State:**
- Still used by original `cargo run` single-player
- Multiplayer uses SpacetimeDB Room table instead
- Kept for backward compatibility
- Marked with deprecation notice in meta section

**Action:** Keep until single-player is fully retired or migrated

## 🗂️ Directories

### `src/`
**Status:** ✅ Active  
**Contents:**
- `main.rs` - Single-player game
- `bin/` - Multiple binaries (terminal_client, sdk_client, signal_bot)
- `terminal_client/` - HTTP client modules + TERMINAL_CLIENT_STATUS.md
- `spacetimedb_client/` - Generated SDK bindings (19 files) + SDK_CLIENT_GUIDE.md
- `signal_client/` - Signal bot modules + documentation (5 .md files)
- Core modules: models, commands, world, output
- `multiplayer/` - Architecture modules (legacy/design reference)

### `docs/`
**Status:** ✅ Active  
**Contents:**
- `architecture-analysis.md` - Single → multi transition
- `multiplayer-state-design.md` - Tick-based design
- `spacetimedb-integration.md` - Backend architecture
- `spacetimedb-quickstart.md` - Quick reference

### `examples/`
**Status:** ✅ Empty (cleaned up)  
**Previous Contents:**
- `test_flow.rs` - Manual flow test (deleted - didn't compile)

**Action:** Directory kept for future examples

### `target/`
**Status:** ✅ Active (auto-generated)  
**Purpose:** Cargo build artifacts  
**Action:** Gitignored, auto-regenerated

### `tests/`
**Status:** ✅ Active  
**Purpose:** Test scripts and integration tests  
**Contents:**
- `test_backend.sh` - SpacetimeDB backend validation
- `test_signal_bot.sh` - Signal bot validation
- Future: Rust integration tests can go here

## 🔧 Configuration

### `.gitignore`
**Status:** ✅ Active  
**Purpose:** Exclude build artifacts and temp files from git

### `.git/`
**Status:** ✅ Active  
**Purpose:** Git repository data  
**Repo:** Arch-Node/rs_text_game_test on main branch

### `LICENSE`
**Status:** ✅ Active  
**Purpose:** Project license

## 📊 Summary

| File | Status | Action |
|------|--------|--------|
| Cargo.toml | ✅ Active | Keep - updated with SDK and Signal features |
| Cargo.lock | ✅ Active | Keep - auto-generated |
| README.md | ✅ Active | Keep - main documentation |
| TODO.md | ✅ Active | Keep - tracks progress |
| ROOT_FILES.md | ✅ Active | Keep - this file |
| bot_config.toml.example | ✅ Active | Keep - example config |
| test_backend.sh | ➡️ Moved | Now in tests/ directory |
| test_signal_bot.sh | ➡️ Moved | Now in tests/ directory |
| rooms.json | ⚠️ Legacy | Keep for now - marked deprecated |
| src/ | ✅ Active | Keep - main code (with module docs) |
| docs/ | ✅ Active | Keep - architecture docs |
| examples/ | ✅ Active | Keep - ready for future examples |
| tests/ | ✅ Active | Keep - test scripts |
| target/ | ✅ Active | Keep - build artifacts |
| .gitignore | ✅ Active | Keep - git config |
| LICENSE | ✅ Active | Keep - legal |

## 🎯 Recommendations

### Files to Keep ✅
All files currently serve a purpose. The root directory is well-organized.

### Files to Watch ⚠️
- `rooms.json` - Can be removed once single-player is retired

### Removed Files ✅
- `examples/test_flow.rs` - Deleted (didn't compile, not useful)

### Missing Files? 🤔
Consider adding:
- `CONTRIBUTING.md` - If accepting contributions
- `.env.example` - If environment variables needed
- `CHANGELOG.md` - For tracking version history
- `docker-compose.yml` - For easier SpacetimeDB setup

---

**Last Updated:** January 23, 2026  
**SpacetimeDB Module:** text-game on localhost:3000  
**Room Count:** 10 (centered at 100,100,100)
