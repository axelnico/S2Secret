import { invoke } from "@tauri-apps/api/core";

export async function load() {
    await invoke("load_secrets_descriptive_data");
    await invoke("load_emergency_contacts");
    await invoke("renew_shares");
    return { passwords: await invoke("passwords"), emergencyContacts: await invoke("emergency_contacts") };
}