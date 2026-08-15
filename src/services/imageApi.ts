// Saving/reading FindImage template images (the paste-a-screenshot flow).

import { invoke } from "@tauri-apps/api/core";

// Save image bytes (from clipboard) into the automation's assets/ as a lossless
// PNG. Returns the relative path to store in the FindImage step's image_path.
export function saveImage(
  appName: string,
  stepIndex: number,
  bytes: Uint8Array
): Promise<string> {
  // Tauri serializes a plain number[] to Vec<u8> on the Rust side.
  return invoke<string>("save_image", {
    appName,
    stepIndex,
    bytes: Array.from(bytes),
  });
}

// Read a saved template image back as a data: URL, so the preview survives
// navigating away and reopening the automation.
export function readImage(
  appName: string,
  relativePath: string
): Promise<string> {
  return invoke<string>("read_image", { appName, relativePath });
}
