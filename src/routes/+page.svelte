<script lang="ts">
  import { preventDefault } from 'svelte/legacy';

  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { preLoginData } from "../state/login.svelte";

  let email = $state("");
  let masterPassword = $state("");
  let databaseFilePath = $state("");
  let isMasterPasswordVisible = $state(false);

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
            <div class="join">
            <input 
              type={isMasterPasswordVisible ? 'text' : 'password'}
              id="password" 
              name="password"
              autocorrect="off"
              autocapitalize="off"
              autocomplete="off"
              required
              placeholder="************" 
              class="input input-secondary input-bordered join-item w-full font-mono"
              bind:value={masterPassword}
            />
              <button
          class={`btn border-secondary border-solid border-info btn-square join-item ${isMasterPasswordVisible ? 'text-success' : 'text-error'}`}
          onclick={() => (isMasterPasswordVisible = !isMasterPasswordVisible)}
          title={isMasterPasswordVisible ? 'Hide password' : 'Show password'}
        >
          {#if isMasterPasswordVisible}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
          {/if}
        </button>
            </div>
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