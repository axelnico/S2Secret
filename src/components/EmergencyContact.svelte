<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import EmergencyContactDeleteForm from "./EmergencyContactDeleteForm.svelte";

    let emergencyContact = $props();

    let deleteModalOpen = $state(false);


    let passwordVisible = $state(false);

    function togglePasswordVisiblity() {
      passwordVisible = !passwordVisible;
    }

    let password = $state("");

    async function revealPassword() {
      password = await invoke<string>("reveal_password", { secretId: emergencyContact.id_emergency_contact });
      passwordVisible = true;
    }

    async function copyPasswordToClipboard() {
      await invoke("copy_password", { secretId: emergencyContact.id_emergency_contact });
    }

</script>


<div class="collapse collapse-arrow bg-base-200">
      <input type="radio" name="my-accordion-2" checked="checked" />
<div class="collapse-title p-4 relative z-10 pointer-events-none">
  <div class="flex flex-col md:flex-row items-center justify-between space-y-2 md:space-y-0">
    <div class="flex flex-col items-center md:items-start">
      <span class="text-xl font-bold">{emergencyContact.description}</span>
    </div>

    <div class="flex items-center justify-between m-4">
      <div class="flex w-full">
        <div class="card bg-base-300 rounded-box h-20 flex flex-row grow items-center space-x-2 px-4 flex-1">
          <span class="font-semibold mr-2">Email:</span>
          <button class="pointer-events-auto p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => {}}>
            <span class="font-semibold mr-2">{emergencyContact.email}</span>
            </button>
        </div>
        <div class="divider divider-horizontal divider-primary"></div>
        <div class="card bg-base-300 rounded-box h-20 flex flex-row grow items-center space-x-2 px-4 flex-1">
          <span class="font-semibold mr-2">Password:</span>
      <button class="pointer-events-auto p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={copyPasswordToClipboard}>
        {#if passwordVisible}
        <span class="text-base normal-case">{password}</span>
        {:else}
        <div class="flex items-center">
          {#each { length: 12 }, password}
          <svg width="15" height="15" viewBox="0 0 48 48" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <rect width="48" height="48" fill="white" fill-opacity="0.01"/>
            <path d="M24 33C28.9706 33 33 28.9706 33 24C33 19.0294 28.9706 15 24 15C19.0294 15 15 19.0294 15 24C15 28.9706 19.0294 33 24 33Z" fill="currentColor" stroke="currentColor" stroke-width="4"/>
            </svg>
          {/each}
        </div>
        {/if}
      </button>
        </div>
      </div>
     
      
      

      <button class="pointer-events-auto p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={revealPassword}>
        <svg
          fill="currentColor"
          width="25"
          height="25"
          viewBox="0 0 32 32"
          xmlns="http://www.w3.org/2000/svg"
        >
          <title>{passwordVisible ? "Hide password" : "Show password"}</title>
          <path d="M16 31c-5.247 0-9.5-4.254-9.5-9.5 0-3.41 1.802-6.391 4.5-8.067v-5.933c0-3.038 2.463-5.5 5.5-5.5s5.5 2.462 5.5 5.5v6.637c2.135 1.742 3.5 4.392 3.5 7.363 0 5.246-4.253 9.5-9.5 9.5zM20 7.5c0-1.933-1.566-3.5-3.5-3.5-1.933 0-3.5 1.567-3.5 3.5v4.991c0.944-0.314 1.95-0.491 3-0.491 1.432 0 2.783 0.325 4 0.892v-5.392zM16 13.5c-4.418 0-8 3.582-8 8s3.582 8 8 8 8-3.582 8-8-3.582-8-8-8zM16 29c-4.143 0-7.5-3.357-7.5-7.5s3.357-7.5 7.5-7.5c4.143 0 7.5 3.357 7.5 7.5s-3.357 7.5-7.5 7.5zM17.5 19.5c0-0.828-0.672-1.5-1.5-1.5s-1.5 0.672-1.5 1.5c0 0.711 0.504 1.277 1.167 1.434l-1.167 4.566h3.062l-1.314-4.551c0.705-0.121 1.252-0.709 1.252-1.449z"></path>
        </svg>
      </button>
    </div>
  </div>
</div>

<div class="collapse-content p-4">
  <div class="space-y-2">
    <div class="flex justify-between">
      <div class="flex justify-start">
        
      </div>
      <div class="flex justify-end">
        <button aria-label="delete-secret" class="p-2 bg-transparent border-none text-error hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => deleteModalOpen = true}>
          <svg width="25" height="25" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M10 12V17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M14 12V17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M4 7H20" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M6 10V18C6 19.6569 7.34315 21 9 21H15C16.6569 21 18 19.6569 18 18V10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M9 5C9 3.89543 9.89543 3 11 3H13C14.1046 3 15 3.89543 15 5V7H9V5Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
        </button>
      </div>
    </div>
  </div>
</div>
    </div>

<EmergencyContactDeleteForm
    emergencyEmail={emergencyContact.email}
    emergencyContactId={emergencyContact.id_emergency_contact}
    isOpened={deleteModalOpen}
    onClose={() => { deleteModalOpen = false; }} />
