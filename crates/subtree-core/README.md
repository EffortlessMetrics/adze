# adze-subtree-core

Core subtree data model used by Adze GLR runtimes.

This crate owns:

- `SubtreeNode` node metadata.
- `Subtree` with dynamic precedence and ambiguity alternatives.
- `ChildEdge` with optional field association.

The `adze` runtime re-exports this API from `adze::subtree` for compatibility.
