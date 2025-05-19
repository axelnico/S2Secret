<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import Secret from "../../../components/Secret.svelte";

    let new_secret = $state({ title: "", userName: "", password: "", site: "", notes: "" });

    let secrets = $state([{ id: "a", title: "Home Router", username: "asus-test", password: "random_password1", site:"https://example.com", notes: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua." }, 
    { id:"b", title: "Facebook", username: "facebook_user", password: "random_password2" },
     { id:"c", title: "Bank", username: "bank_user", password: "random_pasword3", site:"https://example.com" }]);

    function showModal() {
      const modal = document.getElementById("my_modal_5") as HTMLDialogElement;
      modal.showModal();
    }


    async function create_secret() {
      const secret_creation_response = await invoke("add_secret", new_secret);
    }

</script>

<div class="navbar bg-base-100 items-center">
  <div class="flex-1 mx-2 px-2">
    <h2 class="text-2xl font-bold">My secrets</h2>
  </div>
  <div class="flex-none gap-2 m-3">
    <div class="form-control">
      <input type="text" placeholder="Search" class="input input-bordered w-24 md:w-auto" />
    </div>
   </div>
  </div>

<div class="flex flex-col h-full p-4">
  <div class="space-y-4">
    {#each secrets as secret}
    <Secret {...secret} />
    {/each}
  </div>
  <div class="mt-auto flex justify-end m-4 fixed bottom-4 right-4 z-10">
    <button aria-label="New secret" class="btn btn-square btn-primary btn-lg" onclick={showModal}>
      <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" fill="currentColor" class="bi bi-plus-lg" viewBox="0 0 16 16">
          <path fill-rule="evenodd" d="M8 2a.5.5 0 0 1 .5.5v5h5a.5.5 0 0 1 0 1h-5v5a.5.5 0 0 1-1 0v-5h-5a.5.5 0 0 1 0-1h5v-5A.5.5 0 0 1 8 2"/>
        </svg>
    </button>
  </div>
</div>

<dialog id="my_modal_5" class="modal modal-bottom sm:modal-middle">
  <div class="modal-box">
    <h3 class="text-lg font-bold">Secret Data</h3>
    <p class="py-4">Press ESC key or click the Cancel button to close</p>
      
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
            bind:value={new_secret.title}
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
              bind:value={new_secret.userName}
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
              bind:value={new_secret.site}
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
            bind:value={new_secret.password}
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
            bind:value={new_secret.notes}
            class="textarea textarea-bordered textarea-secondary textarea-sm w-full"></textarea>
        </div>
        
      </form>
    <div class="modal-action">
      <form method="dialog">
        <!-- if there is a button in form, it will close the modal -->
        <button class="btn">Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={create_secret}>Add secret</button>
    </div>
  </div>
</dialog>