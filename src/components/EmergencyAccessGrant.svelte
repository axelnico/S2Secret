<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import { emergencyAccess } from "../state/emergency-access.svelte";

    let {secretId, secretTitle, isOpened, onClose } = $props();

    let selectedContactId = $state(null);
    
    //let emergencyContacts = $state([{id:"5D03127A-EF34-4DBF-8C26-2B73986F8890", email:"test@example.com"}]);

</script>

<dialog id="share-modal-{secretId}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Emergency Access</h3>
    <p class="py-4">You are going to share all the information related to {secretTitle}.</p>
      <!-- Form -->
      <form class="space-y-4">
        
        <div class="form-control">
          <label class="label" for="username">
              <span class="label-text">Contact</span>
            </label>
          <select bind:value={selectedContactId} class="select select-secondary">
            <option disabled selected>Pick an emergency contact</option>
            {#each emergencyAccess.contacts as contact}
              <option value={contact.id_emergency_contact}>{contact.email}</option>
            {/each}
          </select>
        </div>
        <div class="form-control">
          <fieldset class="fieldset bg-base-100 border-base-300 rounded-box w-70 border p-4">
            <legend class="fieldset-legend">Share options</legend>
            <label class="label">
              <input type="checkbox" checked={true} class="toggle toggle-warning" />
              Send access data to contact by email
            </label>
          </fieldset>
        </div>
      </form>
    <div class="modal-action">
      <form method="dialog">
        <button onclick={onClose} class="btn">Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={async () => {await invoke("add_access_to_emergency_contact_for_secret", { idEmergencyContact: selectedContactId, secretId: secretId })}}>Share secret</button>
    </div>
  </div>
</dialog>