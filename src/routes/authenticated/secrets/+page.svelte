<script lang="ts">

    let secrets = $state([{ title: "Home Router", username: "asus-test", password: "random_password1" }, 
    { title: "Facebook", username: "facebook_user", password: "random_password2" },
     { title: "Bank", username: "bank_user", password: "random_pasword3" }]);

    function showModal() {
      const modal = document.getElementById("my_modal_5") as HTMLDialogElement;
      modal.showModal();
    }

    function showPassword() {
      const password = document.getElementById("password") as HTMLInputElement;
      password.type = password.type === "password" ? "text" : "password";
    }
</script>

<div class="navbar bg-base-100">
  <div class="flex-1">
    <h2 class="text-2xl font-bold text-center mb-6">My secrets</h2>
  </div>
  <div class="flex-none gap-2">
    <div class="form-control">
      <input type="text" placeholder="Search" class="input input-bordered w-24 md:w-auto" />
    </div>
   </div>
  </div>


  {#each secrets as secret}
  <div class="collapse collapse-arrow bg-base-200">
    <input type="radio" name="my-accordion-2" checked="checked" />
    <div class="collapse-title text-xl font-medium">{secret.title}</div>
    <div class="collapse-content">
      <p>Username: {secret.username}</p>
      <p>Password: ************
        <svg role="button" tabindex="0" onclick={showPassword} fill="currentcolor" width="26" height="26" viewBox="0 0 32 32" version="1.1" xmlns="http://www.w3.org/2000/svg">
          <title>locked1</title>
          <path d="M16 31c-5.247 0-9.5-4.254-9.5-9.5 0-3.41 1.802-6.391 4.5-8.067v-5.933c0-3.038 2.463-5.5 5.5-5.5s5.5 2.462 5.5 5.5v6.637c2.135 1.742 3.5 4.392 3.5 7.363 0 5.246-4.253 9.5-9.5 9.5zM20 7.5c0-1.933-1.566-3.5-3.5-3.5-1.933 0-3.5 1.567-3.5 3.5v4.991c0.944-0.314 1.95-0.491 3-0.491 1.432 0 2.783 0.325 4 0.892v-5.392zM16 13.5c-4.418 0-8 3.582-8 8s3.582 8 8 8 8-3.582 8-8-3.582-8-8-8zM16 29c-4.143 0-7.5-3.357-7.5-7.5s3.357-7.5 7.5-7.5c4.143 0 7.5 3.357 7.5 7.5s-3.357 7.5-7.5 7.5zM17.5 19.5c0-0.828-0.672-1.5-1.5-1.5s-1.5 0.672-1.5 1.5c0 0.711 0.504 1.277 1.167 1.434l-1.167 4.566h3.062l-1.314-4.551c0.705-0.121 1.252-0.709 1.252-1.449z"></path>
          </svg>
      </p>
    </div>
  </div>
  {/each}
  <button class="btn btn-square btn-outline btn-primary" onclick={showModal}>
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
      <button class="btn btn-primary" onclick={() => {}}>Add secret</button>
    </div>
  </div>
</dialog>