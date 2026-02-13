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

## Phase 2: Parallel Cleanup and Command Resolution
- [x] Task: Implement parallel cleanup logic d549910
    - [x] Add logic to iterate through the `clean` section and delete directories in parallel (using `std::fs` and potentially threads or a crate if needed, though `std` is preferred)
- [ ] Task: Implement Command Template Injection
    - [ ] Implement logic to resolve command aliases from `commands.toml`
    - [ ] Implement logic to append project-specific arguments to the base command
- [ ] Task: Write tests for cleanup and resolution
    - [ ] Test that multiple directories are deleted correctly
    - [ ] Test that commands are correctly resolved and arguments are appended
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Parallel Cleanup and Command Resolution' (Protocol in workflow.md)

## Phase 3: Dependency-Driven Execution
- [ ] Task: Implement dependency resolution logic
    - [ ] Implement a simple dependency tracker for `generate` tasks and `projects`
    - [ ] Ensure projects wait for their specific `generate` dependencies
- [ ] Task: Implement group-aware execution (CLI)
    - [ ] Add ability to identify projects belonging to a group
- [ ] Task: Write tests for dependency execution
    - [ ] Test that a project waits for a `generate` task to finish
    - [ ] Test that unrelated projects can run without waiting for unrelated `generate` tasks
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Dependency-Driven Execution' (Protocol in workflow.md)
