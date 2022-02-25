use crate::profile::Emote;
use color_eyre::eyre::{bail, ContextCompat, Result, WrapErr};
use serde_json::Value;

pub fn parse_emotes(html: &str) -> Result<Vec<Emote>> {
    const START: &'static str = "ytInitialData = ";

    let start_index = html.find(START).wrap_err("failed to find ytInitialData")? + START.len();
    let (json_str, _) = html[start_index..]
        .split_once(";</script>")
        .wrap_err("failed to find ending semicolon")?;

    let json =
        serde_json::from_str::<Value>(json_str).wrap_err("failed to parse ytInitialData JSON")?;

    let tabs = json
        .pointer("/contents/twoColumnBrowseResultsRenderer/tabs")
        .wrap_err("failed to find tab data in ytInitialData")?
        .as_array()
        .wrap_err("failed to parse tabs as array")?;

    let emotes = tabs
        .iter()
        .flat_map(|value| {
            value
                .pointer("/tabRenderer/content/sectionListRenderer/contents")
                .into_iter()
                .flat_map(|value| value.as_array().into_iter().flatten())
        })
        .flat_map(|value| {
            value
                .pointer("/sponsorshipsExpandablePerksRenderer/expandableItems")
                .into_iter()
                .flat_map(|value| value.as_array().into_iter().flatten())
        })
        .flat_map(|value| {
            value
                .pointer("/sponsorshipsPerkRenderer/images")
                .into_iter()
                .flat_map(|value| value.as_array().into_iter().flatten())
        })
        .map(|value| {
            let name = value
                .pointer("/accessibility/accessibilityData/label")
                .wrap_err("failed to find label")?
                .as_str()
                .wrap_err("failed to parse label as string")?
                .to_owned();

            let full_url = value
                .pointer("/thumbnails/0/url")
                .wrap_err("failed to find url")?
                .as_str()
                .wrap_err("failed to parse url as string")?;

            let url = if let Some((first, _)) = full_url.split_once('=') {
                first.to_owned()
            } else {
                full_url.to_owned()
            };

            Ok(Emote { name, url })
        })
        .collect::<Result<Vec<Emote>>>()?;

    if emotes.is_empty() {
        bail!("failed to find emotes in JSON")
    } else {
        Ok(emotes)
    }
}
