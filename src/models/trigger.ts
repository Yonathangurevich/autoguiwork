// How an automation is started — mirrors the Rust engine's Trigger.
// Manual = run by clicking Test. Server = exposed as a POST endpoint that
// declares the inputs callers must send.

// One declared input an endpoint accepts (the API contract + UI form field).
export interface InputSpec {
  name: string;
  required: boolean;
}

export type Trigger =
  | "Manual"
  | { Server: { endpoint_id: string; inputs: InputSpec[] } };
