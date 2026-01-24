# Root Directory Files - Quick Reference

## 📄 Core Files

### `Cargo.toml`
**Status:** ✅ Active  
**Purpose:** Rust project configuration with multiple binaries and features  
**Key Features:**
- `default`: No features (single-player only)
- `spacetimedb-http`: Terminal client with HTTP API
- `spacetimedb-sdk-client`: Terminal client with SDK (vendored OpenSSL)
- Binaries: `rust_game_test` (main), `terminal_client`, `sdk_client`

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
**Status:** ✅ Active (just updated)  
**Purpose:** Complete 23-task development roadmap  
**Recent Updates:** Marked Room table and SDK generation complete

### `SDK_CLIENT_GUIDE.md`
**Status:** ✅ Active (newly created)  
**Purpose:** SpacetimeDB SDK setup and usage guide  
**Contents:**
- Generated bindings structure
- Build instructions
- How to regenerate bindings
- SDK vs HTTP comparison
- Next steps

### `TERMINAL_CLIENT_STATUS.md`
**Status:** ✅ Active (testing reference)  
**Purpose:** Backend testing status and known issues  
**Contents:**
- Test room map (10 rooms)
- API mismatch documentation
- Three solution approaches
- Manual testing commands

### `ROOT_FILES.md`
**Status:** ✅ Active (this file)  
**Purpose:** Quick reference for all root files

## 🧪 Testing

### `test_backend.sh`
**Status:** ✅ Active (just updated)  
**Purpose:** Automated backend health check  
**Tests:**
- Server status (localhost:3000/health)
- Room count (should be 10)
- connect_session reducer
- Session creation

**Usage:**
```bash
chmod +x test_backend.sh
./test_backend.sh
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
- `bin/` - Multiple binaries (terminal_client, sdk_client)
- `terminal_client/` - HTTP client modules
- `spacetimedb_client/` - Generated SDK bindings (19 files)
- Core modules: models, commands, world, output
- `multiplayer/` - Architecture modules

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
| Cargo.toml | ✅ Active | Keep - updated with SDK features |
| Cargo.lock | ✅ Active | Keep - auto-generated |
| README.md | ✅ Active | Keep - just updated |
| TODO.md | ✅ Active | Keep - tracks progress |
| SDK_CLIENT_GUIDE.md | ✅ Active | Keep - new SDK docs |
| TERMINAL_CLIENT_STATUS.md | ✅ Active | Keep - testing reference |
| ROOT_FILES.md | ✅ Active | Keep - this file |
| test_backend.sh | ✅ Active | Keep - useful testing tool |
| rooms.json | ⚠️ Legacy | Keep for now - marked deprecated |
| src/ | ✅ Active | Keep - main code |
| docs/ | ✅ Active | Keep - architecture docs |
| examples/ | ✅ Active | Keep - ready for future examples |
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
