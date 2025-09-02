<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import SecretForm from "./SecretForm.svelte";
    import { setPasswords } from "../state/secrets.svelte";

    interface SecretUpsert {
        id: string;
        title: string;
        userName?: string;
        site?: string;
        password: string;
        notes?: string;
    };

    interface Props {
        isOpened: boolean;
    }

    let { isOpened } : Props = $props();

    let newSecretModalOpen = $state(isOpened);

    async function create_secret(new_secret: SecretUpsert) {
      const secret_creation_response = await invoke("add_secret", { ...new_secret });
      setPasswords(await invoke("passwords"));
      newSecretModalOpen = false;
    }

</script>

<SecretForm 
    isOpened={newSecretModalOpen} 
    onClose={() => newSecretModalOpen = false} 
    onSave={create_secret} />