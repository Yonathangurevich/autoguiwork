// Exposing an automation as a worker-server endpoint + building the sample curl.

import { invoke } from "@tauri-apps/api/core";
import type { InputSpec } from "../models";

// Is the shared worker server running? Resolves to the port, or null.
export function serverStatus(): Promise<number | null> {
  return invoke<number | null>("server_status");
}

// Expose an automation as an API endpoint ("set to production"): sets its Server
// trigger + declared inputs, starts the server, returns the URL to call.
export function exposeAsServer(
  name: string,
  inputs: InputSpec[]
): Promise<string> {
  return invoke<string>("expose_as_server", { name, inputs });
}

// Turn an exposed automation back to Manual (not served).
export function unexposeServer(name: string): Promise<void> {
  return invoke("unexpose_server", { name });
}

// Build a ready-to-run curl for this endpoint. The JSON body has one key per
// declared input, with a <name> placeholder the user replaces.
//
// Windows-friendly: a SINGLE line (cmd.exe has no \-line-continuation), -d body
// in double quotes with inner quotes escaped as \" (bash single-quote style
// fails in cmd/PowerShell). Pastes cleanly into cmd.
export function buildCurl(url: string, inputs: InputSpec[]): string {
  const named = inputs.filter((i) => i.name.trim() !== "");
  const body =
    named.length === 0
      ? "{}"
      : "{" +
        named.map((i) => `\\"${i.name}\\": \\"<${i.name}>\\"`).join(", ") +
        "}";
  return `curl -X POST ${url} -H "Content-Type: application/json" -d "${body}"`;
}
