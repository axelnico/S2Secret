<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";


    interface Props {
        secret_id?: string;
        protection_level?: string;
        isOpened: boolean;
        onClose: () => void;
        next_update?: string;
    }

    const MEDIUM_PROTECTION_LEVEL = 0;
    const HIGH_PROTECTION_LEVEL = 40;
    const EXTREME_PROTECTION_LEVEL = 80;

    let { secret_id, isOpened, onClose, protection_level, next_update  } : Props = $props();

    let selectedProtectionLevel = $state(getProtectionLevelValue(protection_level));


    function getProtectionLevelText(level: number): string {
        switch (level) {
            case MEDIUM_PROTECTION_LEVEL:
                return "Medium";
            case HIGH_PROTECTION_LEVEL:
                return "High";
            case EXTREME_PROTECTION_LEVEL:
                return "Extreme";
            default:
                return "Unknown";
        }
    }

    function getProtectionLevelValue(level?: string): number {
        switch (level) {
            case "Medium":
                return MEDIUM_PROTECTION_LEVEL;
            case "High":
                return HIGH_PROTECTION_LEVEL;
            case "Extreme":
                return EXTREME_PROTECTION_LEVEL;
            default:
                return HIGH_PROTECTION_LEVEL; // Default to High if unknown
        }
    }

    async function enableProactiveProtection() {

        await invoke("enable_proactive_protection", {
            secretId: secret_id,
            proactiveProtectionSelected: getProtectionLevelText(selectedProtectionLevel)
        });
        await invoke("renew_share", { secretId: secret_id });
        await invoke("load_secret_descriptive_data",{ secretId: secret_id });
        onClose();
    }

    async function removeProactiveProtection() {
        await invoke("disable_proactive_protection", { secretId: secret_id }); 
        onClose();
    }


</script>

<dialog id="secret-data-modal-{secret_id}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Proactive protection</h3>
    {#if protection_level}
        <p class="py-4">This secret is protected by proactive measures. Please review the details below:</p>
        <p class="py-2">Next update: {next_update}</p>
        {:else}
        <p class="py-2">Enable proactive protection to ensure your secret shares are regularly updated and secure.</p>
        <p class="py-2">Your secret shares are going to be updated at the interval you choose. The secret value is UNCHANGED.</p>
        <p class="py-2 text-warning">Warning: if this feature is enabled and the secret is shared with an emergency contact, it may be neccesary to manually share with them the updated information. Use with caution.</p>
      {/if}
    
    <div class="w-full max-w-s">
  <input type="range" min="0" max="80" class="range" step="40" bind:value={selectedProtectionLevel} />
  <div class="flex justify-between px-2.5 mt-2 text-xs">
    <span>|</span>
    <span>|</span>
    <span>|</span>
  </div>
  <div class="flex justify-between px-2.5 mt-2 text-xs">
    <span>Medium</span>
    <span>High</span>
    <span>Extreme</span>
  </div>
  <div class="flex justify-between px-2.5 mt-2 text-xs">
    <span>Every 30 days</span>
    <span>Every 7 days</span>
    <span>Every 1 day</span>
  </div>
</div>
    <div class="modal-action flex">
      {#if protection_level}
        <button class="btn btn-error" onclick={removeProactiveProtection}>Remove protection</button>
      {/if}
      <form method="dialog">
        <button class="btn" onclick={() => onClose()}>Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={enableProactiveProtection}>Save</button>
    </div>
  </div>
</dialog>