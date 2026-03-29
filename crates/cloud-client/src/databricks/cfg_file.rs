use std::collections::HashMap;

use super::builder::DatabricksConfigKey;
use crate::Result;

/// Default path to the Databricks config file.
const DEFAULT_CFG_PATH: &str = "~/.databrickscfg";
/// Default profile name within the config file.
const DEFAULT_PROFILE: &str = "DEFAULT";

/// Load key-value pairs from a `.databrickscfg` INI profile.
///
/// - `file`: path to the config file; if `None`, defaults to `~/.databrickscfg`.
///   A leading `~/` is expanded using the `HOME` environment variable.
/// - `profile`: section name; if `None`, defaults to `DEFAULT`.
///
/// Returns an empty map (not an error) if the file does not exist.
pub(crate) fn load_cfg_profile(
    file: Option<&str>,
    profile: Option<&str>,
) -> Result<HashMap<DatabricksConfigKey, String>> {
    let raw_path = file.unwrap_or(DEFAULT_CFG_PATH);
    let path = expand_home(raw_path);

    if !std::path::Path::new(&path).exists() {
        return Ok(HashMap::new());
    }

    let cfg = ini::Ini::load_from_file(&path).map_err(|e| crate::Error::Generic {
        source: format!("Failed to parse .databrickscfg at {path:?}: {e}").into(),
    })?;

    let section_name = profile.unwrap_or(DEFAULT_PROFILE);
    let section = cfg.section(Some(section_name));
    let Some(section) = section else {
        return Ok(HashMap::new());
    };

    let mut map = HashMap::new();
    for (k, v) in section.iter() {
        if let Ok(config_key) = k.parse::<DatabricksConfigKey>() {
            map.insert(config_key, v.to_owned());
        }
    }
    Ok(map)
}

/// Expand a leading `~/` to the user's home directory.
fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_owned()
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::*;

    fn write_cfg(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{content}").unwrap();
        f
    }

    #[test]
    fn test_load_default_profile() {
        let tmp = write_cfg("[DEFAULT]\nhost = https://myhost.databricks.com\ntoken = dapi123\n");
        let result = load_cfg_profile(Some(tmp.path().to_str().unwrap()), None).unwrap();
        assert_eq!(
            result.get(&DatabricksConfigKey::Host).map(String::as_str),
            Some("https://myhost.databricks.com")
        );
        assert_eq!(
            result.get(&DatabricksConfigKey::Token).map(String::as_str),
            Some("dapi123")
        );
    }

    #[test]
    fn test_load_named_profile() {
        let tmp = write_cfg(
            "[DEFAULT]\nhost = https://default.databricks.com\n\
             [dev]\nhost = https://dev.databricks.com\ntoken = dapidev\n",
        );
        let result = load_cfg_profile(Some(tmp.path().to_str().unwrap()), Some("dev")).unwrap();
        assert_eq!(
            result.get(&DatabricksConfigKey::Host).map(String::as_str),
            Some("https://dev.databricks.com")
        );
        assert_eq!(
            result.get(&DatabricksConfigKey::Token).map(String::as_str),
            Some("dapidev")
        );
    }

    #[test]
    fn test_missing_profile_returns_empty() {
        let tmp = write_cfg("[DEFAULT]\nhost = https://default.databricks.com\n");
        let result =
            load_cfg_profile(Some(tmp.path().to_str().unwrap()), Some("nonexistent")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_missing_file_returns_empty() {
        let result = load_cfg_profile(Some("/nonexistent/path/.databrickscfg"), None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_unknown_keys_are_ignored() {
        let tmp = write_cfg("[DEFAULT]\nhost = https://x.com\nunknown_key = ignored\n");
        let result = load_cfg_profile(Some(tmp.path().to_str().unwrap()), None).unwrap();
        // Only 'host' should be in the map; 'unknown_key' silently dropped.
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&DatabricksConfigKey::Host));
    }
}
