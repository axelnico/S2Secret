<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    import { emergencyAccesses, setEmergencyAccesses } from "../../../state/emergency-access.svelte";

    interface EmergencyAccessesProps {
        data: EmergencyAccessesList;
    }

    interface EmergencyAccessesList {
        emergencyAccesses: EmergencyAccess[];
    }

    interface EmergencyAccess {
        idEmergencyContact: string;
        idSecret: string;
    }

    let { data }: EmergencyAccessesProps = $props();

    console.log("Loaded emergency accesses:", data.emergencyAccesses);

    setEmergencyAccesses(data.emergencyAccesses);

    let newEmergencyAccessModalOpen = $state(false);

    async function deleteEmergencyAccess(id_emergency_contact: string, id_secret: string) {
      const access_delete_response = await invoke("remove_access_to_emergency_contact_for_secret", { id_emergency_contact, id_secret });
      setEmergencyAccesses(await invoke("get_emergency_accesses_for_all_secrets"));
      newEmergencyAccessModalOpen = false;
    }

</script>

<div class="navbar bg-base-100 items-center">
  <div class="flex-1 mx-2 px-2">
    <h2 class="text-2xl font-bold">Emergency Accesses</h2>
  </div>
  </div>

<div class="flex flex-col h-full p-4">

    <div class="overflow-x-auto">
  <table class="table">
    <!-- head -->
    <thead>
      <tr>
        <th>Emergency Contact</th>
        <th>Secret</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each emergencyAccesses.accesses as access (access.id_emergency_contact)}
    <tr>
        <th>{access.id_emergency_contact}</th>
        <td>{access.id_secret}</td>
        <td>
          <button class="btn btn-danger" onclick={() => deleteEmergencyAccess(access.id_emergency_contact,access.id_secret)}>Delete</button>
        </td>
    </tr>
    {/each}
    </tbody>
  </table>
</div>
</div>

