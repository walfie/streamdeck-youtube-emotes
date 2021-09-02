use crate::profile::Emote;
use anyhow::{Context, Result};
use serde_json::Value;

pub fn parse_emotes(html: &str) -> Result<Vec<Emote>> {
    const START: &'static str = "ytInitialData = ";

    let start_index = html.find(START).context("failed to find ytInitialData")? + START.len();
    let (json_str, _) = html[start_index..]
        .split_once(';')
        .context("failed to find ending semicolon")?;

    let json = serde_json::from_str::<Value>(json_str).context("failed to parse ytInitialData")?;

    let tabs = json
        .pointer("/contents/twoColumnBrowseResultsRenderer/tabs")
        .context("failed to find tab data in ytInitialData")?
        .as_array()
        .context("failed to parse tabs as array")?;

    let content = tabs
        .iter()
        .find_map(|value| match value.pointer("/tabRenderer/title") {
            Some(Value::String(s)) if s == "Membership" => value.pointer(concat!(
                "/tabRenderer/content/sectionListRenderer",
                "/contents/0/sponsorshipsManagementRenderer/content"
            )),
            _ => None,
        })
        .context("failed to find content")?
        .as_array()
        .context("failed to parse content as array")?
        .iter()
        .find_map(|content| {
            content.pointer(concat!(
                "/sponsorshipsExpandableMessageRenderer",
                "/expandableItems/0",
                "/sponsorshipsPerksRenderer/perks/0",
                "/sponsorshipsPerkRenderer/images"
            ))
        })
        .context("failed to find images")?
        .as_array()
        .context("failed to parse images as array")?
        .iter()
        .map(|value| {
            let name = value
                .pointer("/accessibility/accessibilityData/label")
                .context("failed to find label")?
                .as_str()
                .context("failed to parse label as string")?
                .to_owned();

            let full_url = value
                .pointer("/thumbnails/0/url")
                .context("failed to find url")?
                .as_str()
                .context("failed to parse url as string")?;

            let url = if let Some((first, _)) = full_url.split_once('=') {
                first.to_owned()
            } else {
                full_url.to_owned()
            };

            Ok(Emote { name, url })
        })
        .collect::<Result<Vec<Emote>>>()?;

    Ok(content)
}
