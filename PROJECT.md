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
1. **Parser (Rust + Tree-sitter):** CST generation.
2. **Transformer:** CST -> IR (Intermediate Representation) with detail filtering.
3. **IR (YAML):** Editable bridge between code and diagram.
4. **Layout Engine:** Lane-based grid (Lane 120 as main flow).
5. **Renderer:** ASCII/Unicode (initially) and SVG (planned).

---

## 3. Current Project State
- **Language:** Rust
- **Parser:** tree-sitter-c
- **Core Pipeline:** Functional C -> YAML -> ASCII pipeline.
- **Recent Achievements:**
    - [x] Subroutine/Function call rendering (`PredefinedProcess`).
    - [x] Automatic sub-diagram generation for local functions using BFS discovery.
    - [x] Symmetric 3-space padding for all nodes.
    - [x] Full UTF-8 support for Spanish characters (ñ, á, etc.).
    - [x] Safe string truncation to prevent panics.

---

## 4. Roadmap & To-Do List

### Phase 1-3: Core (Completed)
- [x] CLI structure (`parse`, `render`).
- [x] AST -> IR Transformer (If, While, For, Decls).
- [x] Level of Detail filtering (1-5).
- [x] Lane-based ASCII Renderer.

### Phase 4: Polish (In Progress)
- [ ] **Function calls in conditions:** Extract and render subroutines when called inside `if`/`while` conditions.
- [ ] **Advanced Edge Routing:** Refine branch avoidance for complex nested logic.
- [ ] **i18n:** Formal support for multiple languages (Spanish priority).
- [ ] **Visuals:** Colored terminal output support.
- [ ] **Global Variables:** Special visualization at the start of the flow.

### Phase 5: Expansion
- [ ] **Multi-file support:** Scan headers to resolve and render functions from multiple files.
- [ ] **Deep Recursion visualization:** Improve visual representation of deep recursion (currently flat).
- [ ] **SVG Export:** Native SVG writer or Graphviz backend (needs adaptation for multi-function flowcharts).
- [ ] **Loop Rendering:** Improve visualization of back-edges in `do-while/for`.
- [ ] **Documentation:** Finalize README and help messages.
