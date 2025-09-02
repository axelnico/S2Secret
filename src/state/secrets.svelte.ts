interface SecretUpsert {
    id: string;
    title: string;
    userName?: string;
    site?: string;
    password: string;
    notes?: string;
};

export let secrets = $state({passwords: [] as SecretUpsert[]});

export const setPasswords = (newPasswords: SecretUpsert[]) => {
    secrets.passwords = newPasswords;
};