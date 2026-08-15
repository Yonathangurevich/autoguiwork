// Argument values passed to actions — the mirror of the Rust engine's
// TextArg / PosArg. An arg is either a fixed literal or a run-time variable
// (Var), which the engine resolves from the caller's inputs / earlier steps.
//
// serde enum shapes (how the JSON looks):
//   unit variant   -> "Name"        newtype -> { Name: inner }
//   tuple variant  -> { Name: [..] }  struct -> { Name: { .. } }

// A screen position: a fixed (x, y), or a variable set by a Find-image step.
export type PosArg = { Literal: [number, number] } | { Var: string };

// A string: fixed text, or a variable (e.g. an input arg like "invoice").
export type TextArg = { Literal: string } | { Var: string };
