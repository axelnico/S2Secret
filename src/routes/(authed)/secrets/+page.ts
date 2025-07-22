import { invoke } from "@tauri-apps/api/core";

export async function load() {
    await invoke("load_secrets_descriptive_data");
    return { passwords: await invoke("passwords") };
}