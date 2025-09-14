<script lang="ts">
    import { preventDefault } from 'svelte/legacy';
  
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
  
    let emergencyFilePath = $state("");
    let emergencyContactPassword = $state("");

    let transientSecret = $state<TransientSecret | null>(null);

    interface TransientSecret {
        title: string;
        userName?: string;
        site?: string;
        password: string;
        notes?: string;
    };

    async function selectEmergencyFile() {
        const selectedFile = await invoke("select_emergency_file") as string;
        emergencyFilePath = selectedFile;
    }
  
    async function recoverSecret() {
        const result = await invoke("recover_secret", {
            emergencyFile: emergencyFilePath,
            password: emergencyContactPassword
        }) as TransientSecret;

        if (result) {
            transientSecret = result;
        } else {
            // Handle recovery failure
        }

        //goto("/", {replaceState: true })
    }
    function cancel() {
      goto("/", { replaceState: true });
    }
  </script>
  
  <div class="container mx-auto">
    <div class="flex place-content-center">
      <div class="w-full max-w-sm p-6 bg-base-100 rounded-lg shadow-lg">
        <img src="/s2secret-full-logo.svg" alt="S2Secret Logo"/>
        <h2 class="text-2xl font-bold text-center mb-6">Emergency Access</h2>
        {#if transientSecret}
        <fieldset class="fieldset bg-base-200 border-base-300 rounded-box w-xs border p-4">
          <legend class="fieldset-legend">Recovered secret</legend>

          <h1>Title</h1>
          <p>{transientSecret.title}</p>
          <h1>Password</h1>
          <p>{transientSecret.password}</p>

          {#if transientSecret.userName}
          <h1>User Name</h1>
          <p>{transientSecret.userName}</p>
          {/if}

          {#if transientSecret.site}
          <h1>Site</h1>
          <p>{transientSecret.site}</p>
          {/if}
          {#if transientSecret.notes}
          <h1>Notes</h1>
          <p>{transientSecret.notes}</p>
          {/if}
        </fieldset>
        {:else}
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
            <input 
              type="password" 
              id="password" 
              name="password" 
              required
              placeholder="************" 
              class="input input-secondary w-full"
              bind:value={emergencyContactPassword}
            />
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
        {/if}
        <button 
              onclick={cancel}
              class="btn btn-error w-full mt-10"
            >
              Cancel
            </button>
      </div>
    </div>
  </div>
  
  
  