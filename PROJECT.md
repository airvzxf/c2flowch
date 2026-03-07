# C2Flowch - Project Management

## 1. Original Vision & Context (DRAFT)
Convert C source code into clean, readable ANSI flowcharts with a focus on:
- **Human-in-the-loop:** Intermediate YAML representation for manual adjustments.
- **Detail Levels:** Range from executive summaries (Level 1) to technical details (Level 5).
- **Clarity:** Avoid "spaghetti" lines; prefer clean ASCII/Unicode layouts initially.
- **Standardization:** Adhere to ANSI flowchart shapes (Process, Decision, I/O, Terminator).

## 2. Development Plan & Strategy
### Persona Analysis
- **User:** Needs a "magic button" with control over detail.
- **Architect:** Uses `tree-sitter-c` for robust parsing; avoids custom C parsers.
- **UX/UI:** Clean CLI flow: `parse` -> `edit` -> `render`.

### Technical Architecture
1. **Cargo Workspaces:** Separated into `c2flowch-core` (pure logic) and `c2flowch` (CLI/IO).
2. **Parser (Rust + Tree-sitter):** CST generation inside `core`.
3. **Transformer:** CST -> IR (Intermediate Representation) with detail filtering.
4. **IR (YAML):** Editable bridge between code and diagram.
5. **Layout Engine:** Lane-based grid (Lane 120 as main flow).
6. **Renderer:** ASCII/Unicode (initially) and SVG (planned).

---

## 3. Current Project State
- **Language:** Rust
- **Parser:** tree-sitter-c
- **Core Pipeline:** Functional C -> YAML -> ASCII pipeline.
- **Recent Achievements:**
    - [x] **Architecture:** Refactored project into Cargo Workspaces (`core`, `cli`) to decouple IO from pure logic, paving the way for WebAssembly (Wasm) support.
    - [x] Subroutine/Function call rendering (`PredefinedProcess`).
    - [x] Automatic sub-diagram generation for local functions using BFS discovery.
    - [x] Symmetric 3-space padding for all nodes.
    - [x] Full UTF-8 support for Spanish characters (ñ, á, etc.).
    - [x] Safe string truncation to prevent panics.
    - [x] **Performance:** Optimized graph traversal using adjacency lists (parents/children maps).
    - [x] **Efficiency:** Targeted junction fixing using coordinate tracking (HashSet) instead of full canvas sweeps.
    - [x] **Bug Fix:** Resolved junction character issues (e.g., missing `┬`) at node/line boundaries.
    - [x] **Advanced Edge Routing:** Implemented dynamic cascading offsets for nested branch avoidance (switch/break logic).
    - [x] **Determinism:** Fixed non-deterministic subroutine ordering by stabilizing parser discovery (alphabetical fallback).
    - [x] **i18n:** Implemented a static translation system and language metadata in the IR, supporting English and Spanish.

---

## 4. Roadmap & To-Do List

### Phase 1-3: Core (Completed)
- [x] CLI structure (`parse`, `render`).
- [x] AST -> IR Transformer (If, While, For, Decls).
- [x] Level of Detail filtering (1-5).
- [x] Lane-based ASCII Renderer.

### Phase 4: Polish (In Progress)
- [ ] **CLI UX Improvements:**
    - Add version, author, and extended descriptions to `--help`.
    - Document detail levels (1-5) clearly in the CLI help text.
    - Implement human-friendly error messages using `anyhow` context (e.g., file not found).
- [ ] **Core API Refining:** Evaluate making internal modules private while maintaining the intentional openness required for upcoming CLI and WebAssembly integrations.
- [ ] **Function calls in conditions:** Extract and render subroutines when called inside `if`/`while` conditions.
- [x] **Advanced Edge Routing:** Refine branch avoidance for complex nested logic.
- [x] **i18n:** Formal support for multiple languages (English and Spanish implemented).
- [ ] **Visuals:** Colored terminal output support.
- [ ] **Global Variables:** Special visualization at the start of the flow.

### Phase 5: Expansion
- [ ] **Multi-file support:** Scan headers to resolve and render functions from multiple files.
- [ ] **Deep Recursion visualization:** Improve visual representation of deep recursion (currently flat).
- [ ] **SVG Export:** Native SVG writer or Graphviz backend (needs adaptation for multi-function flowcharts).
- [ ] **Loop Rendering:** Improve visualization of back-edges in `do-while/for`.
- [ ] **Documentation:** Finalize README and help messages.
