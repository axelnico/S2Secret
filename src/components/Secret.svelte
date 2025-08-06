<script lang="ts">

    import { invoke } from "@tauri-apps/api/core";
    import SecretDeleteForm from "./SecretDeleteForm.svelte";
    import SecretForm from "./SecretForm.svelte";
    import SecretProactiveProtection from "./SecretProactiveProtection.svelte";

    let secret = $props();

    let deleteModalOpen = $state(false);

    let editSecretModalOpen = $state(false);

    let proactiveProtectionModalOpen = $state(false);

    let passwordVisible = $state(false);

    function togglePasswordVisiblity() {
      passwordVisible = !passwordVisible;
    }

    let password = $state("");

    async function revealPassword() {
      password = await invoke<string>("reveal_password", { secretId: secret.id });
      passwordVisible = true;
    }

    async function copyPasswordToClipboard() {
      await invoke("copy_password", { secretId: secret.id });
    }

    interface SecretUpsert {
        id: string;
        title: string;
        userName?: string;
        site?: string;
        password: string;
        notes?: string;
    };

    async function update_secret(secret_updated: SecretUpsert) {
      const secret_creation_response = await invoke("update_secret", { ...secret_updated });
      editSecretModalOpen = false;
    }
</script>


<div class="collapse collapse-arrow bg-base-200">
      <input type="radio" name="my-accordion-2" checked="checked" />
<div class="collapse-title p-4 relative z-10 pointer-events-none">
  <div class="flex flex-col md:flex-row items-center justify-between space-y-2 md:space-y-0">
    <div class="flex flex-col items-center md:items-start">
      <span class="text-xl font-bold">{secret.title}</span>
      <span class="text-sm text-info">
        Shared with: <span class="text-primary">My contact</span>
      </span>
    </div>

    <div class="flex items-center justify-between m-4">
      <div class="flex w-full">
        <div class="card bg-base-300 rounded-box h-20 flex flex-row grow items-center space-x-2 px-4 flex-1">
          <span class="font-semibold mr-2">Username:</span>
          <button class="pointer-events-auto p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => {}}>
            <span class="font-semibold mr-2">{secret.user_name}</span>
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
    <div>
      <span class="font-semibold text-secondary">Site:</span> {secret.site}
    </div>
    <div>
      <span class="font-semibold text-secondary">Notes:</span>
      <p class="whitespace-pre-wrap">
        {secret.notes}
      </p>
    </div>
    <div class="flex justify-between">
      <div class="flex justify-start">
        <button aria-label="share-secret" class="p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => {}}>
          <svg width="25" height="25" viewBox="0 0 16 16" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M11 6C12.6569 6 14 4.65685 14 3C14 1.34315 12.6569 0 11 0C9.34315 0 8 1.34315 8 3C8 3.22371 8.02449 3.44169 8.07092 3.65143L4.86861 5.65287C4.35599 5.24423 3.70652 5 3 5C1.34315 5 0 6.34315 0 8C0 9.65685 1.34315 11 3 11C3.70652 11 4.35599 10.7558 4.86861 10.3471L8.07092 12.3486C8.02449 12.5583 8 12.7763 8 13C8 14.6569 9.34315 16 11 16C12.6569 16 14 14.6569 14 13C14 11.3431 12.6569 10 11 10C10.2935 10 9.644 10.2442 9.13139 10.6529L5.92908 8.65143C5.97551 8.44169 6 8.22371 6 8C6 7.77629 5.97551 7.55831 5.92908 7.34857L9.13139 5.34713C9.644 5.75577 10.2935 6 11 6Z"/>
            </svg>
        </button>
        <button aria-label="share-share" class="p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => {}}>
          <svg width="25" height="25" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M2 16.9C2 15.5906 2 14.9359 2.29472 14.455C2.45963 14.1859 2.68589 13.9596 2.955 13.7947C3.43594 13.5 4.09063 13.5 5.4 13.5H6.5C8.38562 13.5 9.32843 13.5 9.91421 14.0858C10.5 14.6716 10.5 15.6144 10.5 17.5V18.6C10.5 19.9094 10.5 20.5641 10.2053 21.045C10.0404 21.3141 9.81411 21.5404 9.545 21.7053C9.06406 22 8.40937 22 7.1 22C5.13594 22 4.15391 22 3.4325 21.5579C3.02884 21.3106 2.68945 20.9712 2.44208 20.5675" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <path d="M22 7.1C22 8.40937 22 9.06406 21.7053 9.545C21.5404 9.81411 21.3141 10.0404 21.045 10.2053C20.5641 10.5 19.9094 10.5 18.6 10.5H17.5C15.6144 10.5 14.6716 10.5 14.0858 9.91421C13.5 9.32843 13.5 8.38562 13.5 6.5V5.4C13.5 4.09063 13.5 3.43594 13.7947 2.955C13.9596 2.68589 14.1859 2.45963 14.455 2.29472C14.9359 2 15.5906 2 16.9 2C18.8641 2 19.8461 2 20.5675 2.44208C20.9712 2.68945 21.3106 3.02884 21.5579 3.4325" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <path d="M16.5 6.25C16.5 5.73459 16.5 5.47689 16.6291 5.29493C16.6747 5.23072 16.7307 5.17466 16.7949 5.12911C16.9769 5 17.2346 5 17.75 5C18.2654 5 18.5231 5 18.7051 5.12911C18.7693 5.17466 18.8253 5.23072 18.8709 5.29493C19 5.47689 19 5.73459 19 6.25C19 6.76541 19 7.02311 18.8709 7.20507C18.8253 7.26928 18.7693 7.32534 18.7051 7.37089C18.5231 7.5 18.2654 7.5 17.75 7.5C17.2346 7.5 16.9769 7.5 16.7949 7.37089C16.7307 7.32534 16.6747 7.26928 16.6291 7.20507C16.5 7.02311 16.5 6.76541 16.5 6.25Z" fill="currentColor"/>
            <path d="M12.75 22C12.75 22.4142 13.0858 22.75 13.5 22.75C13.9142 22.75 14.25 22.4142 14.25 22H12.75ZM14.3889 13.8371L14.8055 14.4607L14.8055 14.4607L14.3889 13.8371ZM13.8371 14.3889L13.2135 13.9722L13.2135 13.9722L13.8371 14.3889ZM19 12.75H17V14.25H19V12.75ZM12.75 19V22H14.25V19H12.75ZM17 12.75C16.3134 12.75 15.742 12.7491 15.281 12.796C14.8075 12.8441 14.3682 12.9489 13.9722 13.2135L14.8055 14.4607C14.914 14.3882 15.078 14.3244 15.4328 14.2883C15.8002 14.2509 16.2822 14.25 17 14.25V12.75ZM14.25 17C14.25 16.2822 14.2509 15.8002 14.2883 15.4328C14.3244 15.078 14.3882 14.914 14.4607 14.8055L13.2135 13.9722C12.9489 14.3682 12.8441 14.8075 12.796 15.281C12.7491 15.742 12.75 16.3134 12.75 17H14.25ZM13.9722 13.2135C13.6719 13.4141 13.4141 13.6719 13.2135 13.9722L14.4607 14.8055C14.5519 14.669 14.669 14.5519 14.8055 14.4607L13.9722 13.2135Z" fill="currentColor"/>
            <path d="M22.75 13.5C22.75 13.0858 22.4142 12.75 22 12.75C21.5858 12.75 21.25 13.0858 21.25 13.5H22.75ZM20.7654 21.8478L21.0524 22.5407L21.0524 22.5407L20.7654 21.8478ZM21.8478 20.7654L21.1548 20.4784V20.4784L21.8478 20.7654ZM17 22.75H19V21.25H17V22.75ZM22.75 17V13.5H21.25V17H22.75ZM19 22.75C19.4557 22.75 19.835 22.7504 20.1454 22.7292C20.4625 22.7076 20.762 22.661 21.0524 22.5407L20.4784 21.1548C20.4012 21.1868 20.284 21.2163 20.0433 21.2327C19.7958 21.2496 19.4762 21.25 19 21.25V22.75ZM21.25 19C21.25 19.4762 21.2496 19.7958 21.2327 20.0433C21.2163 20.284 21.1868 20.4012 21.1548 20.4784L22.5407 21.0524C22.661 20.762 22.7076 20.4625 22.7292 20.1454C22.7504 19.835 22.75 19.4557 22.75 19H21.25ZM21.0524 22.5407C21.7262 22.2616 22.2616 21.7262 22.5407 21.0524L21.1548 20.4784C21.028 20.7846 20.7846 21.028 20.4784 21.1549L21.0524 22.5407Z" fill="currentColor"/>
            <path d="M2 7.1C2 5.13594 2 4.15391 2.44208 3.4325C2.68945 3.02884 3.02884 2.68945 3.4325 2.44208C4.15391 2 5.13594 2 7.1 2C8.40937 2 9.06406 2 9.545 2.29472C9.81411 2.45963 10.0404 2.68589 10.2053 2.955C10.5 3.43594 10.5 4.09063 10.5 5.4V6.5C10.5 8.38562 10.5 9.32843 9.91421 9.91421C9.32843 10.5 8.38562 10.5 6.5 10.5H5.4C4.09063 10.5 3.43594 10.5 2.955 10.2053C2.68589 10.0404 2.45963 9.81411 2.29472 9.545C2 9.06406 2 8.40937 2 7.1Z" stroke="currentColor" stroke-width="1.5"/>
            <path d="M5 6.25C5 5.73459 5 5.47689 5.12911 5.29493C5.17466 5.23072 5.23072 5.17466 5.29493 5.12911C5.47689 5 5.73459 5 6.25 5C6.76541 5 7.02311 5 7.20507 5.12911C7.26928 5.17466 7.32534 5.23072 7.37089 5.29493C7.5 5.47689 7.5 5.73459 7.5 6.25C7.5 6.76541 7.5 7.02311 7.37089 7.20507C7.32534 7.26928 7.26928 7.32534 7.20507 7.37089C7.02311 7.5 6.76541 7.5 6.25 7.5C5.73459 7.5 5.47689 7.5 5.29493 7.37089C5.23072 7.32534 5.17466 7.26928 5.12911 7.20507C5 7.02311 5 6.76541 5 6.25Z" fill="currentColor"/>
            <path d="M5 17.75C5 17.2346 5 16.9769 5.12911 16.7949C5.17466 16.7307 5.23072 16.6747 5.29493 16.6291C5.47689 16.5 5.73459 16.5 6.25 16.5C6.76541 16.5 7.02311 16.5 7.20507 16.6291C7.26928 16.6747 7.32534 16.7307 7.37089 16.7949C7.5 16.9769 7.5 17.2346 7.5 17.75C7.5 18.2654 7.5 18.5231 7.37089 18.7051C7.32534 18.7693 7.26928 18.8253 7.20507 18.8709C7.02311 19 6.76541 19 6.25 19C5.73459 19 5.47689 19 5.29493 18.8709C5.23072 18.8253 5.17466 18.7693 5.12911 18.7051C5 18.5231 5 18.2654 5 17.75Z" fill="currentColor"/>
            <path d="M16 17.75C16 17.0478 16 16.6967 16.1685 16.4444C16.2415 16.3352 16.3352 16.2415 16.4444 16.1685C16.6967 16 17.0478 16 17.75 16C18.4522 16 18.8033 16 19.0556 16.1685C19.1648 16.2415 19.2585 16.3352 19.3315 16.4444C19.5 16.6967 19.5 17.0478 19.5 17.75C19.5 18.4522 19.5 18.8033 19.3315 19.0556C19.2585 19.1648 19.1648 19.2585 19.0556 19.3315C18.8033 19.5 18.4522 19.5 17.75 19.5C17.0478 19.5 16.6967 19.5 16.4444 19.3315C16.3352 19.2585 16.2415 19.1648 16.1685 19.0556C16 18.8033 16 18.4522 16 17.75Z" fill="currentColor"/>
            </svg>
        </button>
      </div>
      <div class="flex justify-end">
        <button aria-label="proactive-protection" class="p-2 bg-transparent border-none text-success hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => {proactiveProtectionModalOpen = true}}>
          <svg fill="currentColor" width="25" height="25" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
  
            <g id="Change_password">
            
            <path d="M464.4326,147.54a9.8985,9.8985,0,0,0-17.56,9.1406,214.2638,214.2638,0,0,1-38.7686,251.42c-83.8564,83.8476-220.3154,83.874-304.207-.0088a9.8957,9.8957,0,0,0-16.8926,7.0049v56.9a9.8965,9.8965,0,0,0,19.793,0v-34.55A234.9509,234.9509,0,0,0,464.4326,147.54Z"/>
            
            <path d="M103.8965,103.9022c83.8828-83.874,220.3418-83.8652,304.207-.0088a9.8906,9.8906,0,0,0,16.8926-6.9961v-56.9a9.8965,9.8965,0,0,0-19.793,0v34.55C313.0234-1.3556,176.0547,3.7509,89.9043,89.9012A233.9561,233.9561,0,0,0,47.5674,364.454a9.8985,9.8985,0,0,0,17.56-9.1406A214.2485,214.2485,0,0,1,103.8965,103.9022Z"/>
            
            <path d="M126.4009,254.5555v109.44a27.08,27.08,0,0,0,27,27H358.5991a27.077,27.077,0,0,0,27-27v-109.44a27.0777,27.0777,0,0,0-27-27H153.4009A27.0805,27.0805,0,0,0,126.4009,254.5555ZM328,288.13a21.1465,21.1465,0,1,1-21.1465,21.1464A21.1667,21.1667,0,0,1,328,288.13Zm-72,0a21.1465,21.1465,0,1,1-21.1465,21.1464A21.1667,21.1667,0,0,1,256,288.13Zm-72,0a21.1465,21.1465,0,1,1-21.1465,21.1464A21.1667,21.1667,0,0,1,184,288.13Z"/>
            
            <path d="M343.6533,207.756V171.7538a87.6533,87.6533,0,0,0-175.3066,0V207.756H188.14V171.7538a67.86,67.86,0,0,1,135.7208,0V207.756Z"/>
            
            </g>
            
            </svg>
        </button>
        <button aria-label="edit-secret" class="p-2 bg-transparent border-none text-accent hover:text-primary-dark focus:outline-none focus:ring-2 focus:ring-success mr-2" onclick={() => { editSecretModalOpen = true}}>
          <svg width="25" height="25" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M11 2H9C4 2 2 4 2 9V15C2 20 4 22 9 22H15C20 22 22 20 22 15V13" stroke="#292D32" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M16.04 3.02001L8.16 10.9C7.86 11.2 7.56 11.79 7.5 12.22L7.07 15.23C6.91 16.32 7.68 17.08 8.77 16.93L11.78 16.5C12.2 16.44 12.79 16.14 13.1 15.84L20.98 7.96001C22.34 6.60001 22.98 5.02001 20.98 3.02001C18.98 1.02001 17.4 1.66001 16.04 3.02001Z" stroke="#292D32" stroke-width="1.5" stroke-miterlimit="10" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M14.91 4.1499C15.58 6.5399 17.45 8.4099 19.85 9.0899" stroke="#292D32" stroke-width="1.5" stroke-miterlimit="10" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
        </button>
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

<SecretDeleteForm 
    secretTitle={secret.title} 
    secretId={secret.id} 
    isOpened={deleteModalOpen} 
    onClose={() => { deleteModalOpen = false; }} />

  <SecretForm 
    secret={secret}
    isOpened={editSecretModalOpen} 
    onClose={() => editSecretModalOpen = false} 
    onSave={update_secret} />

  <SecretProactiveProtection
    secret_id={secret.id}
    protection_level={secret.proactive_protection}
    isOpened={proactiveProtectionModalOpen}
    next_update={secret.next_share_update}
    onClose={() => proactiveProtectionModalOpen = false}
  />