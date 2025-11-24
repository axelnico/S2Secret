interface EmergencyContactUpsert {
    id_emergency_contact: string;
    email: string;
    description: string;
}

interface EmergencyAccess {
    idEmergencyContact: string;
    idSecret: string;
}
export let emergencyAccess = $state({contacts: [] as EmergencyContactUpsert[]});

export const setEmergencyContacts = (newContacts: EmergencyContactUpsert[]) => {
    emergencyAccess.contacts = newContacts;
};

export let emergencyAccesses = $state({accesses: [] as EmergencyAccess[]});

export const setEmergencyAccesses = (newAccesses: EmergencyAccess[]) => {
    emergencyAccesses.accesses = newAccesses;
}

export const preEmergencyAccessData = $state({secretId:"",
                                              emergencyContactId:"",
                                              temporalSessionId:"",
                                              password:"",
                                              vShare: null as Uint8Array | null,
                                              dataEncryptionKey: null as Uint8Array | null});