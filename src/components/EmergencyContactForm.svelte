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
          <input 
            type="password" 
            id="password" 
            name="password"
            autocorrect="off"
            autocapitalize="off"
            autocomplete="off"
            required
            maxlength="128"
            placeholder="************" 
            class="input input-secondary w-full"
            bind:value={emergency_contact_modification.password}
          />
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