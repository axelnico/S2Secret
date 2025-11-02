<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    interface EmergencyContact {
        id_emergency_contact: string;
        description: string;
        email: string;
        password: string;
    };

    interface Props {
        emergency_contact?: EmergencyContact;
        isOpened: boolean;
        onClose: () => void;
        onSave: (emergency_contact: EmergencyContact) => Promise<void>;
    }

    let isEmergencyContactPasswordVisible = $state(false);

    let { emergency_contact, isOpened, onClose, onSave } : Props = $props();

    let emergency_contact_modification = $state<EmergencyContact>({
        id_emergency_contact: emergency_contact?.id_emergency_contact || "",
        description: emergency_contact?.description || "",
        email: emergency_contact?.email || "",
        password: emergency_contact?.password || ""
    });

    let password = $derived.by(async () => {
        return emergency_contact_modification ? await invoke<string>("reveal_password", { secretId: emergency_contact_modification.id_emergency_contact }) : "";  
    })

</script>

<dialog id="emergency-contact-data-modal-{emergency_contact_modification.id_emergency_contact}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Emergency Contact Data</h3>
      
      <!-- Form -->
      <form class="space-y-4">
        
        <div class="form-control">
          <label class="label" for="description">
            <span class="label-text">Description</span>
          </label>
          <input 
            type="text" 
            id="description" 
            name="description"
            autocapitalize="off"
            autocomplete="off"
            required
            placeholder="My Emergency Contact" 
            class="input input-secondary w-full"
            bind:value={emergency_contact_modification.description}
          />
        </div>

        <div class="form-control">
            <label class="label" for="email">
              <span class="label-text">Email</span>
            </label>
            <input 
              type="email" 
              id="email" 
              name="email" 
              placeholder="contact@example.com"
              autocapitalize="off"
              autocomplete="off"
              class="input input-secondary w-full"
              bind:value={emergency_contact_modification.email}
            />
          </div>
        
        <!-- Password Input -->
          <div class="form-control">
            <label class="label" for="password">
              <span class="label-text">Password</span>
            </label>
            <div class="join">
            <input 
              type={isEmergencyContactPasswordVisible ? 'text' : 'password'}
              id="password" 
              name="password"
              autocorrect="off"
              autocapitalize="off"
              autocomplete="off"
              required
              maxlength="128"
              placeholder="************" 
              class="input input-secondary input-bordered join-item w-full font-mono"
              bind:value={emergency_contact_modification.password}
            />
              <button
          class={`btn border-secondary border-solid border-info btn-square join-item ${isEmergencyContactPasswordVisible ? 'text-success' : 'text-error'}`}
          onclick={() => (isEmergencyContactPasswordVisible = !isEmergencyContactPasswordVisible)}
          title={isEmergencyContactPasswordVisible ? 'Hide password' : 'Show password'}
        >
          {#if isEmergencyContactPasswordVisible}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
          {/if}
        </button>
            </div>
          </div>
        
      </form>
    <div class="modal-action">
      <form method="dialog">
        <button class="btn" onclick={() => onClose()}>Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={() => onSave(emergency_contact_modification)}>Save</button>
    </div>
  </div>
</dialog>