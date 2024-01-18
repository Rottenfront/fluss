# Fluss

Pronounced as [flʊs]

Fluss is experimental editor that tries to implement modular and extensible system development environment. Mainly backend is based on Skia, and it is planned to add piet backend.

## TODO list

### Editor

- [ ] Multicursor editing
- [ ] Scrollbar
- [ ] Copy, paste, Input Method Editor
- [ ] Source file AST
  - mainly based on TreeSitter, it's possible to use Lady Deirdre
- [ ] AST-based navigation
- [ ] Source code highlighting
  - Multithreaded, based on AST
- [ ] LSP
  - Multithreaded
  - Uses separated widgets to display information

### Components

- [ ] Tabbing system
- [ ] Editor space splitter (vertical and horizontal)
- [ ] Abstract tab type (to implement markdown & html preview)
- [ ] Left, bottom, right docks for tools
- [ ] Toolbar with buttons and bottom bar with text widgets, tooltips are required
- [ ] Top-level windows (hovering, completion, toltips)
- [ ] Hard-coded plug-in system
  - On every plug-in update it's required to recompile editor
  - The most easy and simple system
- [ ] Runtime plug-in system
  - Deno, wasm or special scripting language required
  - Declarative GUI framework required

- [ ] Declarative GUI framework
  - Must be standalone project
  - Must be pluggable into editor pane
  - Backend is based on kurbo types
  - kurbo will be used as geometry types provider
  - winit is used as an example of event system for GUI widgets. To find the
    event destination, focus system (for keyboard) and cascade search
    (for mouse) will be used
  - Should be identical to rui/cushy/gpui2. Main widgets list will be based on
    SwiftUI widget's purpose and appearance list

### Modules

- [ ] Source code editor
  - Focus on JetBrains IntelliJ editor's identity
  - Uses only monospace fonts
- [ ] Terminal
  - Based on VTE protocol
  - Uses only monospace fonts
- [ ] File explorer
  - Uses any font
  - Appearance must be completely customizable
  - Planned:
    1. Creation and renaming files and folders
    2. drag'n'drop for files and folders, main example VSCode
    3. Code check and formatting in files and folders

Language support priorities are Rust, C и C++.
On the second place are TypeScript, Python, C#, Java и Kotlin.
After implementing the main features of the editor, it is possible to support
any language that has a server that supports LSP, and an implementation of the
TreeSitter parser.

Don't look at this project as a product until all these points are completed.

There are also plans to support ChatGPT/Copilot or Codeium in the foreseeable
future.
If possible, CRDTs will be implemented for multi-user source code editing.

## Why?

I am doing this project as my goal in Rust programming.
I want to create an editor in which I will enjoy writing code.
This is a pet project started by an ordinary lyceum student with a
specialization in physics and mathematics.
Don't expect anything very big from him.
Perhaps in a couple of months this repository will cease to exist.
