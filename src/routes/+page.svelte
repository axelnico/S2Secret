<script lang="ts">
  import { preventDefault } from 'svelte/legacy';

  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";

  let email = $state("");
  let masterPassword = $state("");
  let greetMsg = "";

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    // greetMsg = await invoke("greet", { email });
    goto("/authenticated/secrets", { replaceState: true });
  }
</script>

<div class="container mx-auto">
  <div class="flex place-content-center">
    <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
      <h2 class="text-2xl font-bold text-center mb-6">Login</h2>
      
      <!-- Form -->
      <form class="space-y-4" onsubmit={preventDefault(greet)}>
        
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
            placeholder="********" 
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
            Login
          </button>
        </div>
      </form>
  
      <!-- Extra Links -->
      <div class="text-center mt-4 text-sm">
        <p>Don't have an account? <a href="#" class="text-primary hover:underline">Sign up</a></p>
      </div>
    </div>
  </div>
</div>


