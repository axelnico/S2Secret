<script lang="ts">
    import { preventDefault } from 'svelte/legacy';
  
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
  
    let email = $state("");
    let name = $state("");
    let masterPassword = $state("");
  
    async function register() {
      const register_response = await invoke("register_user", { email, name, masterPassword });
      console.log(register_response);
    }
    function cancel() {
      goto("/", { replaceState: true });
    }
  </script>
  
  <div class="container mx-auto">
    <div class="flex place-content-center">
      <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
        <h2 class="text-2xl font-bold text-center mb-6">Registration</h2>
        
        <!-- Form -->
        <form class="space-y-4" onsubmit={register}>
          
          <!-- Email Input -->
          <div class="form-control">
            <label class="label" for="email">
              <span class="label-text">Email</span>
            </label>
            <input 
              type="email" 
              id="email" 
              name="email" 
              required
              placeholder="email@example.com" 
              class="input input-secondary w-full"
              bind:value={email}
            />
          </div>

          <!-- Name Input -->
          <div class="form-control">
            <label class="label" for="name">
              <span class="label-text">Name</span>
            </label>
            <input 
              id="name" 
              name="name" 
              required
              placeholder="Firstname Lastname" 
              class="input input-secondary w-full"
              bind:value={name}
            />
          </div>
          
          <!-- Password Input -->
          <div class="form-control">
            <label class="label" for="password">
              <span class="label-text">Master Password</span>
            </label>
            <input 
              type="password" 
              id="password" 
              name="password" 
              required
              minlength="12"
              placeholder="************" 
              class="input input-secondary w-full"
              bind:value={masterPassword}
            />
          </div>
          
          <!-- Submit Button -->
          <div class="form-control mt-6">
            <button 
              type="submit" 
              class="btn btn-primary w-full"
            >
              Register
            </button>
          </div>
        </form>
        <button 
              onclick={cancel}
              class="btn btn-error w-full mt-10"
            >
              Cancel
            </button>
      </div>
    </div>
  </div>
  
  
  