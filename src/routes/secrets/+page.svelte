<script lang="ts">
    import { goto } from "$app/navigation";

    function showModal() {
      const modal = document.getElementById("my_modal_5") as HTMLDialogElement;
      modal.showModal();
    }
    function logout() {
      goto("/", { replaceState: true });
    }
</script>

<button class="btn btn-error" on:click={logout}><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-lock" viewBox="0 0 16 16">
    <path d="M8 1a2 2 0 0 1 2 2v4H6V3a2 2 0 0 1 2-2m3 6V3a3 3 0 0 0-6 0v4a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2M5 8h6a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1"/>
  </svg>Lock</button>
<button class="btn btn-square btn-outline btn-primary" on:click={showModal}>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-plus-lg" viewBox="0 0 16 16">
        <path fill-rule="evenodd" d="M8 2a.5.5 0 0 1 .5.5v5h5a.5.5 0 0 1 0 1h-5v5a.5.5 0 0 1-1 0v-5h-5a.5.5 0 0 1 0-1h5v-5A.5.5 0 0 1 8 2"/>
      </svg>
  </button>
<dialog id="my_modal_5" class="modal modal-bottom sm:modal-middle">
  <div class="modal-box">
    <h3 class="text-lg font-bold">Secret Data</h3>
    <p class="py-4">Press ESC key or click the button below to close</p>
      
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
            required
            placeholder="My Secret" 
            class="input input-secondary w-full"
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
              class="input input-secondary w-full"
            />
          </div>

          <div class="form-control">
            <label class="label" for="site">
              <span class="label-text">Site</span>
            </label>
            <input 
              type="url" 
              id="site" 
              name="site" 
              placeholder="https://example.com" 
              class="input input-secondary w-full"
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
            required
            placeholder="********" 
            class="input input-secondary w-full"
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
            class="textarea textarea-bordered textarea-secondary textarea-sm w-full"></textarea>
        </div>
        
      </form>
    <div class="modal-action">
      <form method="dialog">
        <!-- if there is a button in form, it will close the modal -->
        <button class="btn">Cancel</button>
      </form>
      <button class="btn btn-primary" on:click={logout}>Add secret</button>
    </div>
  </div>
</dialog>