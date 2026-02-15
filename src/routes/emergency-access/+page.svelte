<script lang="ts">
    import { preventDefault } from 'svelte/legacy';
  
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
    import { preEmergencyAccessData } from '../../state/emergency-access.svelte';
  
    let emergencyFilePath = $state("");
    let emergencyContactPassword = $state("");
    let isEmergencyContactPasswordVisible = $state(false);

    async function selectEmergencyFile() {
        const selectedFile = await invoke("select_emergency_file") as string;
        emergencyFilePath = selectedFile;
    }

  
    async function recoverSecret() {
        const result = await invoke("recover_secret", {
            emergencyFile: emergencyFilePath,
            password: emergencyContactPassword
        });

        if (result) {
          console.log(result);
          preEmergencyAccessData.password = emergencyContactPassword;
          preEmergencyAccessData.vShare = result.v_share;
          preEmergencyAccessData.dataEncryptionKey = result.data_encryption_key;
          preEmergencyAccessData.temporalSessionId = result.temporal_session_id;
          preEmergencyAccessData.secretId = result.secret_id;
          preEmergencyAccessData.emergencyContactId = result.emergency_contact_id;
          goto("/emergency-access-2fa", {replaceState: true })
        } else {
            // Handle recovery failure
        }

    }
    function cancel() {
      emergencyFilePath = "";
      emergencyContactPassword = "";
      goto("/", { replaceState: true });
    }
  </script>
  
  <div class="container mx-auto">
    <div class="flex place-content-center">
      <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
        <img src="/s2secret-full-logo.svg" alt="S2Secret Logo"/>
        <h2 class="text-2xl font-bold text-center mb-6">Emergency Access</h2>
       <form class="space-y-4" onsubmit={recoverSecret}>
          
          <!-- Emergency Data Input -->
        <div class="form-control">
          <label class="label" for="emergency_data_file">
            <span class="label-text">Emergency data file</span>
          </label>
          <input type="button"
                onclick={selectEmergencyFile}
                 id="emergency_data_file"
                 name="emergency_data_file"
                 class="btn input-secondary"
                 required
                 placeholder="Select emergency data file"
                 bind:value={emergencyFilePath}
          />
        </div>
          
          <!-- Password Input -->
          <div class="form-control">
            <label class="label" for="password">
              <span class="label-text">Password</span>
            </label>
            <div class="join">
            <input 
              type={isEmergencyContactPasswordVisible ? 'text' : 'password'}
              id="password" 
              name="password"
              autocorrect="off"
              autocapitalize="off"
              autocomplete="off"
              required
              placeholder="************" 
              class="input input-secondary input-bordered join-item w-full font-mono"
              bind:value={emergencyContactPassword}
            />
              <button
          class={`btn border-secondary border-solid border-info btn-square join-item ${isEmergencyContactPasswordVisible ? 'text-success' : 'text-error'}`}
          onclick={() => (isEmergencyContactPasswordVisible = !isEmergencyContactPasswordVisible)}
          title={isEmergencyContactPasswordVisible ? 'Hide password' : 'Show password'}
        >
          {#if isEmergencyContactPasswordVisible}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
          {/if}
        </button>
            </div>
          </div>
          
          <!-- Submit Button -->
          <div class="form-control mt-6">
            <button 
              type="submit" 
              class="btn btn-primary w-full"
            >
              Recover secret
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
  
  
  