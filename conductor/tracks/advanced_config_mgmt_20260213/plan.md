# Implementation Plan - Implement advanced configuration and project management

## Phase 1: Configuration Schema and Parsing [checkpoint: c3484e3]
- [x] Task: Update data models for `drom.toml` 3cb8530
    - [x] Add `Clean`, `Generate`, `Project`, and `Group` structs to `src/main.rs`
    - [x] Update `Config` struct to include the new sections
- [x] Task: Implement `commands.toml` parsing c0089d1
    - [x] Create `commands.toml` sample file
    - [x] Implement parsing logic for the command alias mapping
- [x] Task: Write tests for enhanced configuration parsing f806f51
    - [x] Test parsing of a full `drom.toml` with all new sections
    - [x] Test parsing of `commands.toml`
    - [x] Test error handling for missing sections or malformed TOML
- [x] Task: Conductor - User Manual Verification 'Phase 1: Configuration Schema and Parsing' (Protocol in workflow.md)

## Phase 2: Parallel Cleanup and Command Resolution [checkpoint: d4ac76e]
- [x] Task: Implement parallel cleanup logic d549910
    - [x] Add logic to iterate through the `clean` section and delete directories in parallel (using `std::fs` and potentially threads or a crate if needed, though `std` is preferred)
- [x] Task: Implement Command Template Injection 62629da
    - [x] Implement logic to resolve command aliases from `commands.toml`
    - [x] Implement logic to append project-specific arguments to the base command
- [x] Task: Write tests for cleanup and resolution 2affef1
    - [x] Test that multiple directories are deleted correctly
    - [x] Test that commands are correctly resolved and arguments are appended
- [x] Task: Conductor - User Manual Verification 'Phase 2: Parallel Cleanup and Command Resolution' (Protocol in workflow.md)

## Phase 3: Dependency-Driven Execution
- [x] Task: Implement dependency resolution logic 381c74c
    - [x] Implement a simple dependency tracker for `generate` tasks and `projects`
    - [x] Ensure projects wait for their specific `generate` dependencies
- [x] Task: Implement group-aware execution (CLI) 15020a6
    - [x] Add ability to identify projects belonging to a group
- [x] Task: Write tests for dependency execution 3e7df48
    - [x] Test that a project waits for a `generate` task to finish
    - [x] Test that unrelated projects can run without waiting for unrelated `generate` tasks
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Dependency-Driven Execution' (Protocol in workflow.md)
