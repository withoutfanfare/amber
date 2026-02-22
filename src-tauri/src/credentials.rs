use serde::{Deserialize, Serialize};

const SERVICE_PREFIX: &str = "com.amber.profile";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProfileCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_password: Option<String>,
}

fn entry_for(profile_id: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(&format!("{SERVICE_PREFIX}.{profile_id}"), "credentials")
}

/// Store credentials as JSON in macOS Keychain.
pub fn store_credentials(
    profile_id: &str,
    creds: &ProfileCredentials,
) -> Result<(), crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    let json = serde_json::to_string(creds)?;
    entry.set_password(&json)?;
    Ok(())
}

/// Retrieve credentials from macOS Keychain.
pub fn get_credentials(profile_id: &str) -> Result<ProfileCredentials, crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    match entry.get_password() {
        Ok(json) => Ok(serde_json::from_str(&json)?),
        Err(keyring::Error::NoEntry) => Ok(ProfileCredentials::default()),
        Err(e) => Err(e.into()),
    }
}

/// Delete credentials from macOS Keychain.
pub fn delete_credentials(profile_id: &str) -> Result<(), crate::error::DsmError> {
    let entry = entry_for(profile_id)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
