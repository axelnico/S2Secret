<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";


    interface Secret {
        id: string;
        title: string;
        user_name?: string;
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

    let isPasswordVisible = $state(false);

    let { secret, isOpened, onClose, onSave } : Props = $props();

    let secret_modification = $state<Secret>({
        id: secret?.id || "",
        title: secret?.title || "",
        userName: secret?.user_name || "",
        site: secret?.site || "",
        password: secret?.password || "",
        notes: secret?.notes || ""
    });

    //secret_modification.password = await invoke<string>("reveal_password", { secretId: secret_modification.id })


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
            type={isPasswordVisible ? 'text' : 'password'}
            id="password" 
            name="password"
            autocorrect="off"
            autocapitalize="off"
            autocomplete="off"
            required
            maxlength="128"
            placeholder="************" 
            class="input input-secondary w-full"
            bind:value={secret_modification.password}
          />
          <button
          class={`btn border-solid border-info btn-square join-item ${isPasswordVisible ? 'text-success' : 'text-error'}`}
          onclick={() => (isPasswordVisible = !isPasswordVisible)}
          title={isPasswordVisible ? 'Hide password' : 'Show password'}
        >
          {#if isPasswordVisible}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
          {/if}
        </button>
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