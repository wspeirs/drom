# Specification - Implement advanced configuration and project management

## Overview
Enhance `drom` to support a more complex configuration structure across multiple TOML files. This includes dedicated sections for environment cleanup, pre-project generation tasks, and sophisticated project management with dependencies and groupings.

## Functional Requirements

### 1. Enhanced `drom.toml` Structure
The `drom.toml` file will be expanded to include the following sections:

#### `clean` Section
- **Description:** A list of directory paths that should be removed to reset the environment.
- **Execution:** Directories specified in this section should be deleted in parallel.

#### `generate` Section
- **Description:** Defines tasks for code generation (e.g., Protobuf, Thrift) or other pre-processing steps.
- **Execution:** These tasks are executed based on the dependency requirements of individual projects.

#### `projects` Section
- **Description:** Defines the individual executable components of the system.
- **Features:**
  - **Dependencies:** Projects can depend on tasks in the `generate` section or other projects.
  - **Commands:** Projects reference base commands defined in `commands.toml`.
  - **Arguments:** Projects can provide specific arguments to the base commands (Template Injection).

#### `groups` Section
- **Description:** Explicitly defines logical groupings of projects.
- **Format:** `[[group]]` array containing a group name and a list of member project names.

### 2. `commands.toml` Integration
A separate `commands.toml` file will store reusable command templates.
- **Format:** Key-value pairs where the key is the command alias (e.g., `mvn`) and the value is the base executable path or command string.
- **Usage:** `drom` will resolve command aliases from `projects` in `drom.toml` using the definitions in `commands.toml` and inject the project-specific arguments.

### 3. Execution Logic
- **Parallel Cleanup:** `clean` operations should be performed concurrently.
- **Dependency-Driven Generation:** Projects must not start until their declared `generate` dependencies are successfully completed. Unrelated projects can start immediately.

## Acceptance Criteria
- `drom` successfully parses the new `drom.toml` structure including `clean`, `generate`, `projects`, and `groups`.
- `drom` successfully parses `commands.toml`.
- Verification that `clean` directories are removed (Parallel execution).
- Verification that projects correctly resolve base commands from `commands.toml` and append their own arguments.
- Verification that dependency ordering between `generate` tasks and `projects` is respected.

## Out of Scope
- Complex cycle detection in dependencies (basic detection only).
- Advanced UI for group execution (CLI only).
- Execution of multiple projects in parallel (to be addressed in a future track).
