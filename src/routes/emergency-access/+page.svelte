<script lang="ts">
    import { preventDefault } from 'svelte/legacy';
  
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
  
    let emergencyFilePath = $state("");
    let emergencyContactPassword = $state("");

    async function selectEmergencyFile() {
        const selectedFile = await invoke("select_emergency_file") as string;
        emergencyFilePath = selectedFile;
    }
  
    async function recoverSecret() {
        const result = await invoke("recover_secret", {
            emergencyFile: emergencyFilePath,
            password: emergencyContactPassword
        }) as string;

        if (result) {
            // Handle successful recovery
        } else {
            // Handle recovery failure
        }

        goto("/", {replaceState: true })
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
        
        <!-- Form -->
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
              minlength="12"
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
        <button 
              onclick={cancel}
              class="btn btn-error w-full mt-10"
            >
              Cancel
            </button>
      </div>
    </div>
  </div>
  
  
  