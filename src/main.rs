mod profile;
mod youtube;

use crate::profile::{DeviceModel, ProfilesWithImages};
use color_eyre::eyre::{bail, Result, WrapErr};
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::fmt().init();

    let mut args = Args::from_args();
    if let Some(prefix) = args.prefix.strip_prefix('_') {
        warn!(%prefix, "Ignoring leading underscore in prefix");
        args.prefix = prefix.to_owned();
    }

    // Find output path based on platform
    let root_path = if let Some(ref path) = args.out {
        path.clone()
    } else if let Some(home) = dirs::home_dir() {
        if cfg!(target_os = "macos") {
            home.join("Library")
                .join("Application Support")
                .join("com.elgato.StreamDeck")
                .join("ProfilesV2")
                .to_path_buf()
        } else if !cfg!(target_os = "windows") {
            home.join("%AppData%")
                .join("Roaming")
                .join("StreamDeck")
                .join("ProfilesV2")
                .to_path_buf()
        } else {
            bail!("No output path specified")
        }
    } else {
        bail!("Could not find home directory")
    };

    // Parse HTML file to get list of emotes
    let html = if args.file.to_str() == Some("-") {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(&args.file)
            .with_context(|| format!("Failed to read file {:?}", &args.file))?
    };

    let emotes = youtube::parse_emotes(&html)?;

    // Generate profiles
    let profiles = ProfilesWithImages::new(
        args.profile_uuid
            .unwrap_or_else(|| profile::uuid_v5(&args.name, 0)),
        args.model,
        args.device_uuid,
        args.name,
        emotes,
        &args.prefix,
        args.include_labels,
    )
    .await?;

    // Write profiles to filesystem
    let mut current_path = root_path;
    let mut is_root = true;
    for (uuid, manifest) in profiles.manifests {
        if is_root {
            is_root = false;
        } else {
            // Nested profiles have an additional `Profiles` directory
            current_path.push("Profiles");
        }

        current_path.push(format!("{}.sdProfile", uuid.to_string().to_uppercase()));
        info!(path = ?current_path, "Creating profile directory");

        fs::create_dir_all(&current_path)
            .with_context(|| format!("Failed to create path {:?}", &current_path))?;

        let manifest_path = current_path.join("manifest.json");
        let mut json = serde_json::to_value(&manifest)?;

        if args.merge {
            if let Err(e) = merge_manifests(&mut json, &manifest_path) {
                warn!(error = %e, path = ?manifest_path, "Failed to merge existing manifest file");
            }
        }

        fs::write(&manifest_path, serde_json::to_vec(&json)?)
            .with_context(|| format!("Failed to write file {:?}", &manifest_path))?;

        for (position, action) in manifest.actions.iter() {
            let img_path = current_path
                .join(format!("{},{}", position.x, position.y))
                .join("CustomImages");

            fs::create_dir_all(&img_path)
                .with_context(|| format!("Failed to create path {:?}", &img_path))?;

            let img_file_path = img_path.join("state0.png");
            if let Some(bytes) = &action.image {
                fs::write(&img_file_path, bytes)
                    .with_context(|| format!("Failed to write image {:?}", &img_file_path))?;
            }
        }
    }

    Ok(())
}

fn merge_manifests(new_manifest: &mut Value, existing_path: &PathBuf) -> Result<()> {
    let string = fs::read_to_string(existing_path).context("Manifest file is missing")?;
    let old_manifest = serde_json::from_str::<Value>(&string).context("Invalid JSON")?;

    let old_actions = if let Some(actions) = old_manifest
        .pointer("/Actions")
        .and_then(|json| json.as_object())
    {
        actions
    } else {
        bail!("Existing manifest file has invalid `Actions` field");
    };

    let new_actions = if let Some(actions) = new_manifest
        .pointer_mut("/Actions")
        .and_then(|json| json.as_object_mut())
    {
        actions
    } else {
        bail!("New manifest file has invalid `Actions` field");
    };

    for (pos, action) in old_actions.into_iter() {
        if !new_actions.contains_key(pos) {
            new_actions.insert(pos.to_owned(), action.clone());
        }
    }

    Ok(())
}

#[derive(StructOpt)]
pub struct Args {
    /// Path to an HTML file containing the "join" (memberships) page for a channel.
    /// E.g., Download the following page in a browser while logged in:
    /// https://www.youtube.com/channel/UCP4nMSTdwU1KqYWu3UH5DHQ/join
    ///
    /// Use - to read from stdin.
    #[structopt(parse(from_os_str))]
    pub file: PathBuf,

    /// The emote prefix (also known as "family name"). For example, if the channel has an emote
    /// `:_pomuSmall9cm:`, the emote prefix would be `pomu`.
    #[structopt(default_value = "", long)]
    pub prefix: String,

    /// Name of the Stream Deck profile. Note that if the `profile-uuid` argument is unspecified, this name will
    /// be used to determine the name of the output profile directory.
    #[structopt(long)]
    pub name: String,

    /// Device UUID for the Stream Deck
    #[structopt(default_value = "", long)]
    pub device_uuid: String,

    /// Override the UUID for the profile
    #[structopt(long)]
    pub profile_uuid: Option<Uuid>,

    /// Whether to include the name of the emote on each key
    #[structopt(long)]
    pub include_labels: bool,

    /// Merge the output `manifest.json` files instead of overwriting them entirely.
    /// Useful if you have your own custom actions for keys that don't overlap with the ones
    /// generated by this program.
    #[structopt(long)]
    pub merge: bool,

    /// Output path to save the profile to. If unspecified, profiles will be saved to the default
    /// Stream Deck profile location (depending on platform).
    #[structopt(long)]
    pub out: Option<PathBuf>,

    /// The Stream Deck model to generate the profile for
    #[structopt(long, possible_values = &["standard", "xl", "mini"])]
    pub model: DeviceModel,
}
