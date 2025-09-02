interface EmergencyContactUpsert {
    id_emergency_contact: string;
    email: string;
    description: string;
}

export let emergencyAccess = $state({contacts: [] as EmergencyContactUpsert[]});

export const setEmergencyContacts = (newContacts: EmergencyContactUpsert[]) => {
    emergencyAccess.contacts = newContacts;
};