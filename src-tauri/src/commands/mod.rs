pub mod candidates;
pub mod capture;
pub mod development;
pub mod feedback;
pub mod inspirations;
pub mod interviews;
pub mod performance;
pub mod posts;
pub mod preferences;
pub mod privacy;
pub mod providers;
pub mod quality;
pub mod roles;
pub mod stories;
pub mod target_contexts;
pub mod topics;
pub mod vault;
pub mod voice;

pub mod external {
    use std::process::Command;

    const SUPPORT_URL: &str = "https://buymeacoffee.com/SlothDC";
    const LICENSE_URL: &str =
        "https://github.com/Three-Wheeled-Sloth-Studio/Worklore/blob/main/LICENSE";

    fn validate_external_url(url: &str) -> Result<(), String> {
        match url {
            SUPPORT_URL | LICENSE_URL => Ok(()),
            _ => Err("WorkLore only opens approved external links in the system browser.".to_string()),
        }
    }

    #[cfg(target_os = "windows")]
    fn open_with_system_browser(url: &str) -> std::io::Result<std::process::ExitStatus> {
        Command::new("rundll32.exe")
            .arg("url.dll,FileProtocolHandler")
            .arg(url)
            .status()
    }

    #[cfg(target_os = "macos")]
    fn open_with_system_browser(url: &str) -> std::io::Result<std::process::ExitStatus> {
        Command::new("open").arg(url).status()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn open_with_system_browser(url: &str) -> std::io::Result<std::process::ExitStatus> {
        Command::new("xdg-open").arg(url).status()
    }

    #[tauri::command]
    pub fn open_external_url(url: String) -> Result<(), String> {
        validate_external_url(&url)?;
        let status = open_with_system_browser(&url)
            .map_err(|error| format!("WorkLore could not start the system browser: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("WorkLore could not open that link in the system browser.".to_string())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{validate_external_url, LICENSE_URL, SUPPORT_URL};

        #[test]
        fn approved_external_links_are_allowed() {
            assert!(validate_external_url(SUPPORT_URL).is_ok());
            assert!(validate_external_url(LICENSE_URL).is_ok());
        }

        #[test]
        fn arbitrary_external_links_are_rejected() {
            assert!(validate_external_url("https://example.com").is_err());
            assert!(validate_external_url("http://buymeacoffee.com/SlothDC").is_err());
        }
    }
}
