import { invoke } from "@tauri-apps/api/core";

export async function load() {
    await invoke("load_emergency_contacts");
    return { emergencyContacts: await invoke("emergency_contacts") };
}