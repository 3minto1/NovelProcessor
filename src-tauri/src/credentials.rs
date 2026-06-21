#[cfg(target_os = "windows")]
pub(crate) fn store_api_key(profile_id: &str, api_key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("novel-processor", profile_id)
        .map_err(|error| format!("无法创建凭证条目：{}", error))?;
    entry
        .set_password(api_key)
        .map_err(|error| format!("无法保存 API Key：{}", error))
}

#[cfg(target_os = "windows")]
pub(crate) fn load_api_key(profile_id: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new("novel-processor", profile_id)
        .map_err(|error| format!("无法创建凭证条目：{}", error))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("无法读取 API Key：{}", error)),
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn delete_api_key(profile_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("novel-processor", profile_id)
        .map_err(|error| format!("无法创建凭证条目：{}", error))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("无法删除 API Key：{}", error)),
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn store_api_key(_profile_id: &str, _api_key: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn load_api_key(_profile_id: &str) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn delete_api_key(_profile_id: &str) -> Result<(), String> {
    Ok(())
}
