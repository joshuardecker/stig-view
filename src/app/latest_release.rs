use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;

/// The information from the github api response we care about.
#[derive(Debug, Deserialize)]
struct GithubAPIResponse {
    tag_name: String,
}

/// Returns a new version number of this app if it exists.
/// If the connection fails to get latest version for any reason, this returns None.
pub fn is_latest_version() -> Option<String> {
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent: Agent = config.into();

    let body: String = agent
        .get("https://api.github.com/repos/joshuardecker/xylok-view/releases/latest")
        .call()
        .ok()?
        .body_mut()
        .read_to_string()
        .ok()?;

    let api_response: GithubAPIResponse = serde_json::from_str(&body).ok()?;

    // Get the version of the app from the Cargo.toml file.
    let version = env!("CARGO_PKG_VERSION");

    // If the newer version is greater, return the newer version number.
    if api_response.tag_name.trim_start_matches("v") > version {
        Some(api_response.tag_name)
    } else {
        None
    }
}
