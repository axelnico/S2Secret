<script lang="ts">
    import { preventDefault } from 'svelte/legacy';
  
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
  
    let emergencyFilePath = $state("");
    let emergencyContactPassword = $state("");
    let isPasswordVisible = $state(false);

    let transientSecret = $state<TransientSecret | null>(null);

    interface TransientSecret {
        title: string;
        user_name?: string;
        site?: string;
        password: string;
        notes?: string;
    };

    async function selectEmergencyFile() {
        const selectedFile = await invoke("select_emergency_file") as string;
        emergencyFilePath = selectedFile;
    }


    async function copyToClipboard(text: string) {
        await invoke("copy_to_clipboard", { text });
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
      transientSecret = null;
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
        {#if transientSecret}
        <div class="card bg-base-200 shadow-xl w-xs">
  <div class="card-body gap-4">
    <div role="alert" class="alert alert-warning">
  <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 shrink-0 stroke-current" fill="none" viewBox="0 0 22 22">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
  </svg>
  <span>Warning: Copy all the information below in a safe place if you need to use it later.</span>
</div>
    <h2 class="card-title border-b border-base-300 pb-2">Recovered Secret</h2>
    

    <div>
      <div class="text-xs font-bold uppercase opacity-60">Title</div>
      <p class="text-lg font-semibold">{transientSecret.title}</p>
    </div>

    <div>
      <div class="text-xs font-bold uppercase opacity-60 mb-1">Password</div>
      <div class="join w-full">
        <input
          type={isPasswordVisible ? 'text' : 'password'}
          value={transientSecret.password}
          readonly
          class="input input-bordered join-item w-full font-mono"
        />
        <button
          class={`btn border-solid border-info btn-square join-item ${isPasswordVisible ? 'text-success' : 'text-error'}`}
          onclick={() => (isPasswordVisible = !isPasswordVisible)}
          title={isPasswordVisible ? 'Hide password' : 'Show password'}
        >
          {#if isPasswordVisible}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68"/><path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61"/><line x1="2" x2="22" y1="2" y2="22"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/></svg>
          {/if}
        </button>
        <button
          aria-label="copy-secret"
          class="btn border-solid border-info btn-square join-item text-accent"
          onclick={() => copyToClipboard(transientSecret?.password as string)}
          title="Copy password"
        >
             <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
        </button>
      </div>
    </div>

    {#if transientSecret.user_name}
      <div class="divider my-0"></div>
      <div>
         <div class="text-xs font-bold uppercase opacity-60">User Name</div>
        <div class="flex justify-between items-center">
          <p class="font-mono text-sm break-all">{transientSecret.user_name}</p>
          <button aria-label="copy-user-name" class="btn btn-ghost btn-xs text-accent" onclick={() => {copyToClipboard(transientSecret?.user_name as string)}} >
           <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
          </button>
        </div>
      </div>
    {/if}

    {#if transientSecret.site}
      <div class="divider my-0"></div>
      <div>
        <div class="text-xs font-bold uppercase opacity-60">Site</div>
        <div class="flex justify-between items-center">
            <p class="font-mono text-sm break-all">{transientSecret.site}</p>
           <button aria-label="copy-URL" class="btn btn-ghost btn-xs text-accent" onclick={() =>  {copyToClipboard(transientSecret?.site as string)}} >
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            </button>
        </div>
      </div>
    {/if}
    
    {#if transientSecret.notes}
      <div class="divider my-0"></div>
      <div>
        <div class="text-xs font-bold uppercase opacity-60">Notes</div>
        <p class="text-sm mt-1 bg-base-100 p-2 rounded-lg whitespace-pre-wrap">{transientSecret.notes}</p>
      </div>
    {/if}
  </div>
</div>
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
              {transientSecret ? "Close and delete all data": "Cancel"}
            </button>
      </div>
    </div>
  </div>
  
  
  