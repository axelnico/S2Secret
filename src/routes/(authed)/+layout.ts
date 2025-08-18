import { redirect } from '@sveltejs/kit';
import { invoke } from "@tauri-apps/api/core";

export async function load() {
    const is_authenticated = await invoke<boolean>("is_authenticated");
    if (!is_authenticated) {
        return redirect(303, '/');
    } else {
        await invoke("logged_user_data");
    }
}