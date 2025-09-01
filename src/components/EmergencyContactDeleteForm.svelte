<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";

    let {emergencyContactId, emergencyEmail, isOpened, onClose } = $props();

    let deleteConfirmationInputText = $state("");

    let deleteConfirmationInputTextHint = `delete ${emergencyEmail}`;

    async function delete_emergency_contact() {
      const emergency_contact_deletion_response = await invoke("delete_emergency_contact", { emergencyContactId:emergencyContactId });
      onClose();
}
</script>

<dialog id="delete-modal-{emergencyContactId}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Emergency Contact Deletion</h3>
    <p class="py-4">You are going to delete all the information related to {emergencyEmail}.</p>
    <p class="py-4">ALL secrets accesses will be revoked for your contact.</p>
    <p class="py-4"><span class="text-warning">Warning: This action is irreversible!</span> If you are sure to continue please complete the input below with the words "delete" followed by a space and the email of the emergency contact you want to delete.</p>

      <!-- Form -->
      <form class="space-y-4">
        
        <div class="form-control">
          <input 
            type="text" 
            id="title" 
            name="title" 
            required
            placeholder={deleteConfirmationInputTextHint}
            class="input input-secondary w-full"
            bind:value={deleteConfirmationInputText}
          />
        </div>

      </form>
    <div class="modal-action">
      <form method="dialog">
        <button onclick={() => {deleteConfirmationInputText = ""; onClose();}} class="btn">Cancel</button>
      </form>
      <button disabled={deleteConfirmationInputText != deleteConfirmationInputTextHint} class="btn btn-error" onclick={delete_emergency_contact}>Delete emergency contact</button>
    </div>
  </div>
</dialog>