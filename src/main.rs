mod profile;
mod youtube;

use crate::profile::{DeviceModel, ProfilesWithImages};
use color_eyre::eyre::{Result, WrapErr};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::fmt().init();

    let mut args = Args::from_args();
    if let Some(prefix) = args.prefix.strip_prefix('_') {
        warn!(%prefix, "Ignoring leading underscore in prefix");
        args.prefix = prefix.to_owned();
    }

    let html = if args.file.to_str() == Some("-") {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(&args.file)
            .with_context(|| format!("Failed to read file {:?}", &args.file))?
    };
    let emotes = youtube::parse_emotes(&html)?;

    let profiles = ProfilesWithImages::new(
        args.model,
        args.device_uuid,
        args.name,
        emotes,
        &args.prefix,
        args.include_labels,
    )
    .await?;

    let mut current_path = args.out.to_path_buf();
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
        fs::write(&manifest_path, serde_json::to_vec(&manifest)?)
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

#[derive(StructOpt)]
struct Args {
    /// Path to an HTML file containing the "join" (memberships) page for a channel.
    /// E.g., Download the following page in a browser while logged in:
    /// https://www.youtube.com/channel/UCP4nMSTdwU1KqYWu3UH5DHQ/join
    ///
    /// Use - to read from stdin.
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// The emote prefix (also known as "family name"). For example, if the channel has an emote
    /// `:_pomuSmall9cm:`, the emote prefix would be `pomu`.
    #[structopt(default_value = "", long)]
    prefix: String,

    /// Name of the Stream Deck profile
    #[structopt(long)]
    name: String,

    /// Device UUID for the Stream Deck
    #[structopt(default_value = "", long)]
    device_uuid: String,

    /// Whether to include the name of the emote on each key
    #[structopt(long)]
    include_labels: bool,

    /// Output path to save the profile to
    #[structopt(long)]
    out: PathBuf,

    /// The Stream Deck model to generate the profile for
    #[structopt(long, possible_values = &["standard", "xl", "mini"])]
    model: DeviceModel,
}
