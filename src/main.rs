mod profile;
mod youtube;

use crate::profile::{DeviceModel, ProfilesWithImages};
use color_eyre::eyre::{Result, WrapErr};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::from_args();

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
        DeviceModel::Standard,
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
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    #[structopt(default_value = "", long)]
    prefix: String,
    #[structopt(long)]
    name: String,
    #[structopt(default_value = "", long)]
    device_uuid: String,
    #[structopt(long)]
    include_labels: bool,
    #[structopt(long)]
    out: PathBuf,
}
