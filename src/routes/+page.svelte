<script lang="ts">
  import { preventDefault } from 'svelte/legacy';

  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { preLoginData } from "../state/login.svelte";

  let email = $state("");
  let masterPassword = $state("");
  let databaseFilePath = $state("");

  async function selectDatabaseFile() {
    const selectedFile = await invoke("select_database_file") as string;
    databaseFilePath = selectedFile;
  }

  async function login() {
      const temporalSessionId = await invoke("login", { email, masterPassword}) as string;

      preLoginData.email = email;
      preLoginData.temporalSessionId = temporalSessionId;
      goto("/2fa", {replaceState: true })
        //if (is_authenticated) {
      //  goto("/secrets", { replaceState: true });
      //}
    }
</script>

<div class="container mx-auto">
  <div class="flex place-content-center">
    <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
      <img src="/s2secret-full-logo.svg" alt="S2Secret Logo"/>
      <h2 class="text-2xl font-bold text-center mb-6">Login</h2>
      
      <!-- Form -->
      <form class="space-y-4" onsubmit={login}>
        
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

        <!-- Database Input -->
        <div class="form-control">
          <label class="label" for="email">
            <span class="label-text">Database file</span>
          </label>
          <input type="button"
                onclick={selectDatabaseFile}
                 id="database_file"
                 name="database_file"
                 class="btn input-secondary"
                 required
                 placeholder="Select database file"
                 bind:value={databaseFilePath}
          />
        </div>

        <!-- <div class="join">
           <input class="input input-secondary join-item" required placeholder="database" />
           <button class="btn input-secondary join-item">Choose</button>
        </div> -->
        
        <!-- Submit Button -->
        <div class="form-control mt-6 pt-4">
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
        <p>Don't have an account? <a href="/sign-up" class="text-primary hover:underline">Sign up</a></p>
        <p class="py-3"><a href="/emergency-access" class="text-error hover:underline">Emergency Access</a></p>
      </div>
    </div>
  </div>
</div>