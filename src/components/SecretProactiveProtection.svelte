<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";


    interface Props {
        secret_id?: string;
        isOpened: boolean;
        onClose: () => void;
    }

    const MEDIUM_PROTECTION_LEVEL = 0;
    const HIGH_PROTECTION_LEVEL = 40;
    const EXTREME_PROTECTION_LEVEL = 80;

    let protectionLevel = $state(HIGH_PROTECTION_LEVEL); // Default to High protection level

    let { secret_id, isOpened, onClose  } : Props = $props();

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

    async function enableProactiveProtection() {

        await invoke("enable_proactive_protection", {
            secretId: secret_id,
            proactiveProtectionSelected: getProtectionLevelText(protectionLevel)
        });
        onClose();
    }


</script>

<dialog id="secret-data-modal-{secret_id}" class="modal modal-bottom sm:modal-middle" class:modal-open={isOpened}>
  <div class="modal-box">
    <h3 class="text-lg font-bold">Proactive protection</h3>
    <p class="py-4">This secret is protected by proactive measures. Please review the details below:</p>
    <div class="w-full max-w-s">
  <input type="range" min="0" max="80" class="range" step="40" bind:value={protectionLevel} />
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
    <div class="modal-action">
      <form method="dialog">
        <button class="btn" onclick={() => onClose()}>Cancel</button>
      </form>
      <button class="btn btn-primary" onclick={enableProactiveProtection}>Save</button>
    </div>
  </div>
</dialog>