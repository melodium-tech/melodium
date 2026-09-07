# Runtime

Mélodium uses a runtime engine. Script files are fully parsed and their logic built and checked _before_ any execution starts. This means type errors, missing connections, unsatisfied inputs, and dependency problems are all caught at startup, not mid-run.

When launching a Mélodium program, the following stages happen in order:

1. **Script textual parsing and semantic build** - the source files are parsed and their syntax validated. Element declarations are collected and checked for structural correctness.
1. **Usage and dependencies resolution** - all `use` imports are resolved and external packages are loaded. Missing or incompatible dependencies are reported here.
1. **Logic building** - the full treatment graph is assembled and validated: connection types are checked, cycles are detected, and every input is verified to be satisfied. No code runs until this passes.
1. **Models instantiation** - model instances are created and configured with their parameters. Startup behaviors (such as opening connections) occur here.
1. **Execution and tracks triggering** - the program begins running. Models start producing events and spawning tracks.
