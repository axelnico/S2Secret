<script lang="ts">

    interface Secret {
        id: string;
        title: string;
        userName?: string;
        site?: string;
        password: string;
        notes?: string;
    };
    interface Props {
        secret?: Secret;
        isOpened: boolean;
        onClose: () => void;
        onSave: (secret: Secret) => Promise<void>;
    }

    let { secret, isOpened, onClose, onSave } : Props = $props();

    let secret_modification = $state<Secret>({
        id: secret?.id || "",
        title: secret?.title || "",
        userName: secret?.userName || "",
        site: secret?.site || "",
        password: secret?.password || "",
        notes: secret?.notes || ""
    });

</script>

<dialog id="secret-data-modal-{secret_modification.id}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Secret Data</h3>
      
      <!-- Form -->
      <form class="space-y-4">
        
        <div class="form-control">
          <label class="label" for="title">
            <span class="label-text">Title</span>
          </label>
          <input 
            type="text" 
            id="title" 
            name="title"
            autocapitalize="off"
            autocomplete="off"
            required
            placeholder="My Secret" 
            class="input input-secondary w-full"
            bind:value={secret_modification.title}
          />
        </div>

        <div class="form-control">
            <label class="label" for="username">
              <span class="label-text">Username</span>
            </label>
            <input 
              type="text" 
              id="username" 
              name="username" 
              placeholder="@username"
              autocapitalize="off"
              autocomplete="off"
              class="input input-secondary w-full"
              bind:value={secret_modification.userName}
            />
          </div>

          <div class="form-control">
            <label class="label" for="site">
              <span class="label-text">Site</span>
            </label>
            <input 
              type="url" 
              id="site"
              autocapitalize="off"
              autocomplete="off"
              name="site" 
              placeholder="https://example.com" 
              class="input input-secondary w-full"
              bind:value={secret_modification.site}
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
            autocapitalize="off"
            autocomplete="off"
            required
            placeholder="********" 
            class="input input-secondary w-full"
            bind:value={secret_modification.password}
          />
        </div>
        <div class="form-control">
            <label class="label" for="notes">
                <span class="label-text">Notes</span>
              </label>
            <textarea
            id="notes"
            name="notes"
            placeholder="Insert additional notes here (max 1024 characters)"
            bind:value={secret_modification.notes}
            class="textarea textarea-bordered textarea-secondary textarea-sm w-full"></textarea>
        </div>
        
      </form>
    <div class="modal-action">
      <form method="dialog">
        <button class="btn" onclick={() => onClose()}>Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={() => onSave(secret_modification)}>Save</button>
    </div>
  </div>
</dialog>