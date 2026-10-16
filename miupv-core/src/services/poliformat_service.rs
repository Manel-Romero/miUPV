//! PoliformaT (Sakai LMS) service for sites, assignments, and announcements.

use crate::client::{parse_json_list, MiupvClient};
use crate::error::CoreError;
use crate::models::{Announcement, Site, Task};
use reqwest::Method;

/// Fetches PoliformaT sites/workspaces for the authenticated user using the default client.
pub async fn get_sites(token: &str) -> Result<Vec<Site>, CoreError> {
    let client = MiupvClient::default();
    get_sites_with_client(&client, token).await
}

/// Fetches PoliformaT sites/workspaces using a custom client.
pub async fn get_sites_with_client(client: &MiupvClient, token: &str) -> Result<Vec<Site>, CoreError> {
    let base = client.config().poliformat_base_url.trim_end_matches('/');
    let url = format!("{base}/direct/site.json?_limit=100");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches pending and past tasks/assignments using the default client.
pub async fn get_tasks(token: &str) -> Result<Vec<Task>, CoreError> {
    let client = MiupvClient::default();
    get_tasks_with_client(&client, token).await
}

/// Fetches pending and past tasks/assignments using a custom client.
pub async fn get_tasks_with_client(client: &MiupvClient, token: &str) -> Result<Vec<Task>, CoreError> {
    let base = client.config().poliformat_base_url.trim_end_matches('/');
    let url = format!("{base}/direct/assignment/my.json");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches user announcements from PoliformaT using the default client.
pub async fn get_announcements(
    token: &str,
    num: u32,
    dias: u32,
) -> Result<Vec<Announcement>, CoreError> {
    let client = MiupvClient::default();
    get_announcements_with_client(&client, token, num, dias).await
}

/// Fetches user announcements from PoliformaT using a custom client.
pub async fn get_announcements_with_client(
    client: &MiupvClient,
    token: &str,
    num: u32,
    dias: u32,
) -> Result<Vec<Announcement>, CoreError> {
    let base = client.config().poliformat_base_url.trim_end_matches('/');
    let url = format!("{base}/direct/announcement/user.json?n={num}&d={dias}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}
