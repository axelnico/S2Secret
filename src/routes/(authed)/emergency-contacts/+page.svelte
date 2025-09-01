<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";


    import EmergencyContact from "../../../components/EmergencyContact.svelte";
    import EmergencyContactForm from "../../../components/EmergencyContactForm.svelte";

    interface EmergencyContactProps {
        data: EmergencyContactList;
    }

    interface EmergencyContactList {
        emergency_contacts: EmergencyContactUpsert[];
    }

    interface EmergencyContactUpsert {
        id_emergency_contact: string;
        email: string;
        description: string;

    }

    let {data} : EmergencyContactProps = $props();

    let newEmergencyContactModalOpen = $state(false);

    async function create_emergency_contact(new_contact: EmergencyContactUpsert) {
      const contact_creation_response = await invoke("add_emergency_contact", { ...new_contact });
      newEmergencyContactModalOpen = false;
    }

</script>

<div class="navbar bg-base-100 items-center">
  <div class="flex-1 mx-2 px-2">
    <h2 class="text-2xl font-bold">My emergency contacts</h2>
  </div>
  </div>

<div class="flex flex-col h-full p-4">
  <div class="space-y-4">
    {#each data.emergency_contacts as contact (contact.id_emergency_contact)}
    <EmergencyContact {...contact} />
    {/each}
  </div>
  <div class="mt-auto flex justify-end m-4 fixed bottom-4 right-4 z-10">
    <button aria-label="New secret" class="btn btn-square btn-primary btn-lg" onclick={()=>newEmergencyContactModalOpen = true}>
      <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" fill="currentColor" class="bi bi-plus-lg" viewBox="0 0 16 16">
          <path fill-rule="evenodd" d="M8 2a.5.5 0 0 1 .5.5v5h5a.5.5 0 0 1 0 1h-5v5a.5.5 0 0 1-1 0v-5h-5a.5.5 0 0 1 0-1h5v-5A.5.5 0 0 1 8 2"/>
        </svg>
    </button>
  </div>
</div>

<EmergencyContactForm 
    isOpened={newEmergencyContactModalOpen} 
    onClose={() => newEmergencyContactModalOpen = false} 
    onSave={create_emergency_contact} />