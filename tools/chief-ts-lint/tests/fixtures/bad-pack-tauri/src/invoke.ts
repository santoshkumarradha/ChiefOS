import { invoke } from "@tauri-apps/api/core";

export async function callChief() {
  return invoke("chief");
}
