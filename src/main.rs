mod profile;
mod youtube;

use crate::profile::{DeviceModel, ProfilesWithImages};
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::Path;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let device_uuid = "".to_owned();
    let emote_prefix = "pomu";
    let profile_name = "Pomu".to_owned();
    let out_path = Path::new("./out");

    // TODO: Read from file (use `-` for stdin)
    let mut html = String::new();
    std::io::stdin().read_to_string(&mut html)?;
    let emotes = youtube::parse_emotes(&html)?;

    let profiles = ProfilesWithImages::new(
        DeviceModel::Standard,
        device_uuid.into(),
        profile_name,
        emotes,
        emote_prefix,
        true, // TODO: Fix this to true
    )
    .await?;

    let ProfilesWithImages {
        manifests,
        images_by_name,
    } = profiles;

    let mut current_path = out_path.to_path_buf();
    let mut is_root = true;
    for (uuid, manifest) in manifests {
        if is_root {
            is_root = false;
        } else {
            // Nested profiles have an additional `Profiles` directory
            current_path.push("Profiles");
        }

        current_path.push(format!("{}.sdProfile", uuid.to_string().to_uppercase()));

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

            if let Some(state) = action.states.first() {
                let label = &state.title;
                if let Some(bytes) = images_by_name.get(label) {
                    let img_file_path = img_path.join("state0.png");
                    fs::write(&img_file_path, bytes)
                        .with_context(|| format!("Failed to write image {:?}", &img_file_path))?;
                }
            }
        }
    }

    Ok(())
}

#[derive(StructOpt)]
struct Args {}
