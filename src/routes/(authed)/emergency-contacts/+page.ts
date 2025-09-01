import { invoke } from "@tauri-apps/api/core";

export async function load() {
    await invoke("load_emergency_contacts");
    return { emergency_contacts: await invoke("emergency_contacts") };
}