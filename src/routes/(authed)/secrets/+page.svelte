<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import Secret from "../../../components/Secret.svelte";
    import SecretForm from "../../../components/SecretForm.svelte";
    import { secrets, setPasswords } from "../../../state/secrets.svelte";
    import { setEmergencyContacts } from "../../../state/emergency-access.svelte";

    interface SecureItemListProps {
        data: SecureItemList;
    }
    interface EmergencyContactUpsert {
      id_emergency_contact: string;
      email: string;
      description: string;
    }

    interface SecureItemList {
        passwords: SecretUpsert[];
        emergencyContacts: EmergencyContactUpsert[];
    }

    interface SecretUpsert {
        id: string;
        title: string;
        userName?: string;
        site?: string;
        password: string;
        notes?: string;
    };

    let {data} : SecureItemListProps = $props();

    setPasswords(data.passwords);
    setEmergencyContacts(data.emergencyContacts);

    let newSecretModalOpen = $state(false);

    async function create_secret(new_secret: SecretUpsert) {
      const secret_creation_response = await invoke("add_secret", { ...new_secret });
      newSecretModalOpen = false;
    }

    async function searchSecrets(term: string) {
      console.log("Searching for:", term);
      if (term.length >= 3) {
        setPasswords(await invoke("filter_by_search_term", { term }));
      } else {
        setPasswords(await invoke("passwords"));
      }
    }

</script>

<div class="navbar bg-base-100 items-center">
  <div class="flex-1 mx-2 px-2">
    <h2 class="text-2xl font-bold">My secrets</h2>
  </div>
  <div class="flex-none gap-2 m-3">
    <div class="form-control">
      <input type="text" autocorrect="off" autocapitalize="off" autocomplete="off" oninput={(e) => searchSecrets((e.target as HTMLInputElement)?.value)} placeholder="Search" class="input input-bordered w-24 md:w-auto" />
    </div>
   </div>
  </div>

<div class="flex flex-col h-full p-4">
  <div class="space-y-4">
    {#each secrets.passwords as secret (secret.id)}
    <Secret {...secret} />
    {/each}
  </div>
  <div class="mt-auto flex justify-end m-4 fixed bottom-4 right-4 z-10">
    <button aria-label="New secret" class="btn btn-square btn-primary btn-lg" onclick={() => newSecretModalOpen = true}>
      <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" fill="currentColor" class="bi bi-plus-lg" viewBox="0 0 16 16">
          <path fill-rule="evenodd" d="M8 2a.5.5 0 0 1 .5.5v5h5a.5.5 0 0 1 0 1h-5v5a.5.5 0 0 1-1 0v-5h-5a.5.5 0 0 1 0-1h5v-5A.5.5 0 0 1 8 2"/>
        </svg>
    </button>
  </div>
</div>

<SecretForm 
    isOpened={newSecretModalOpen} 
    onClose={() => newSecretModalOpen = false} 
    onSave={create_secret} />