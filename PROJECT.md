# C2Flowch - Project Management

## 1. Original Vision & Context (DRAFT)
Convert C source code into clean, readable ANSI and ISO 5807 flowcharts with a focus on:
- **Human-in-the-loop:** Intermediate YAML representation for manual adjustments.
- **Detail Levels:** Range from executive summaries (Level 1) to technical details (Level 5).
- **Clarity:** Avoid "spaghetti" lines; prefer clean ASCII/Unicode layouts initially.
- **Standardization:** Adhere to ANSI and ISO 5807 flowchart shapes (Process, Decision, I/O, Terminator).

## 2. Development Plan & Strategy
### Persona Analysis
- **User:** Needs a "magic button" with control over detail.
- **Architect:** Uses `tree-sitter-c` for robust parsing; avoids custom C parsers.
- **UX/UI:** Clean CLI flow: `parse` -> `edit` -> `render`.

### Technical Architecture
1. **Cargo Workspaces:** Separated into `c2flowch-core` (pure logic) and `c2flowch` (CLI/IO). Designed for future WebAssembly (Wasm) support.
2. **Parser (Rust + Tree-sitter):** CST generation inside `core`.
3. **Transformer:** CST -> IR (Intermediate Representation) with detail filtering.
4. **IR (YAML):** Editable bridge between code and diagram.
5. **Layout Engine:** Lane-based grid (Lane 120 as main flow).
6. **Renderer:** ASCII/Unicode, professional SVG, and high-quality rasterized PNG.

---

## 3. Current Project State
- **Language:** Rust
- **Parser:** tree-sitter-c
- **Core Pipeline:** Functional C -> YAML -> ASCII/SVG/PNG pipeline.
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
    - [x] **SVG Typography:** Implemented block-centered left-aligned text for all node shapes to ensure professional readability while maintaining symmetry.
    - [x] **i18n:** Implemented a static translation system and language metadata in the IR, supporting English and Spanish.
    - [x] **SVG Export:** Implemented a professional vector rendering engine with precise coordinate mapping, intelligent text-anchor alignment, multiline title support, and automatic subroutine stacking.

---

## 4. Roadmap & To-Do List

### Phase 1-3: Core (Completed)
- [x] CLI structure (`parse`, `render`).
- [x] AST -> IR Transformer (If, While, For, Decls).
- [x] Level of Detail filtering (1-5).
- [x] Lane-based ASCII Renderer.
- [x] High-precision SVG Renderer.

### Phase 4: Polish (In Progress)
- [x] **CLI UX Improvements:**
    - Add version, author, and extended descriptions to `--help`.
    - Document detail levels (1-5) clearly in the CLI help text.
- [ ] **Global Variables:** Special visualization at the start of the flow.

### Phase 5: Expansion
- [ ] **WebAssembly Integration:** Create `c2flowch-wasm` crate to expose core logic to the browser.
- [ ] **Multi-file support:** Scan headers to resolve and render functions from multiple files.

### Phase 6: Final details
- [ ] **Core API Refining:** Evaluate making internal modules private while maintaining the intentional openness required for upcoming CLI and WebAssembly integrations.
- [ ] **Visuals:** Colored terminal output support.
- [ ] **Documentation:** Finalize README and help messages.
