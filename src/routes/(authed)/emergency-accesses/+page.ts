import { invoke } from "@tauri-apps/api/core";

export async function load() {
    return { emergencyAccesses: await invoke("get_emergency_accesses_for_all_secrets") };
}