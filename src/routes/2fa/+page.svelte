<script lang="ts">
  import { preventDefault } from 'svelte/legacy';

  import { invoke } from "@tauri-apps/api/core";
  import { goto, preloadData } from "$app/navigation";
  import { getContext } from 'svelte';
  import { preLoginData } from "../../state/login.svelte";

  let oneTimeSecretCode = $state("");

  const email = preLoginData.email;
  const temporalSessionId = preLoginData.temporalSessionId;

  console.log("Email:", email);
  console.log("Temporal Session ID:", temporalSessionId);

  async function login() {

      await invoke<boolean>("send_2fa_secret_code", { oneTimeSecretCode, email, temporalSessionId });
      preLoginData.email = "";
      preLoginData.temporalSessionId = "";

      const is_authenticated = await invoke<boolean>("is_authenticated");
      if (is_authenticated) {
         goto("/secrets", { replaceState: true });
      }
    }
</script>

<div class="container mx-auto">
  <div class="flex place-content-center">
    <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
      <img src="/s2secret-full-logo.svg" alt="S2Secret Logo"/>
      <h2 class="text-2xl font-bold text-center mb-6">Complete with secret code sent to your email</h2>
      
      <!-- Form -->
      <form class="space-y-4" onsubmit={login}>
        
        <!-- Email Input -->
        <div class="form-control">
          <label class="label" for="email">
            <span class="label-text">Secret Code</span>
          </label>
          <input 
            id="secret_code" 
            name="secret_code" 
            required
            placeholder="Enter the code sent to your email" 
            class="input input-secondary w-full"
            bind:value={oneTimeSecretCode}
          />
        </div>
        
        <!-- Submit Button -->
        <div class="form-control mt-6">
          <button 
            type="submit" 
            class="btn btn-primary w-full"
          >
            Confirm
          </button>
        </div>
      </form>
  
      
    </div>
  </div>
</div>