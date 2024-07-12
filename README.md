# BAD REd

**B**y **A**ustin **D**umm **R**ust **Ed**itor

An experimental and personal-development focused text editor written in Rust with support for Lua scripting tightly
integrated into the editor.

## Goals

- Research and experimentation around efficient text storage and manipulation algorithms
    [x] Naive String Storage
    [x] Gap Buffer
    [ ] Piece Table
    [ ] Ropes
- Run Lua scripts safely (and potentially concurrently) with the core Rust editor code
- Allow for editor self-extension through Lua
- Self documentation

## Non-goals

(Things I am not currently focusing on for this project)

- Language Server Protocol integration
- Production level efficiency
    - The editor itself is not intended for serious development use
- Cross-platform support
    - This was developed with the Kitty terminal running on macOS. Other platforms are not tested

## Profiling

|------------------------|--------------|------------|
|                        | Naive String | Gap Buffer |
|------------------------|--------------|------------|
| Large File Open & Edit | 11m 43s      | 9.9s       |
|------------------------|--------------|------------|
