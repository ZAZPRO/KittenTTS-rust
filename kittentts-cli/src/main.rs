use std::{
    io::{self, IsTerminal, Read},
    path::PathBuf,
};

use anyhow::{Result, bail};
use clap::Parser;
use kittentts_lib::{KittenModel, KittenVoice, wav};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    text: Option<String>,
    #[arg(short, long, value_name = "OUT_WAV_FILE")]
    wav: PathBuf,
    #[arg(short, long)]
    phonems: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let text = match cli.text {
        Some(text) => text,
        None => {
            if io::stdin().is_terminal() {
                bail!(
                    "No text provided. Either provide text as an argument or pipe it to stdin.\nExample: echo \"hello\" | kittentts-cli --wav out.wav\nExample: kittentts-cli \"hello\" --wav out.wav"
                );
            }
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let trimmed = buffer.trim();
            if trimmed.is_empty() {
                bail!("No text received");
            }
            trimmed.to_string()
        }
    };

    let mut model = KittenModel::model_builtin(KittenVoice::default())?;
    let out = if cli.phonems {
        model.generate_from_phonems(text.clone())?
    } else {
        model.generate(text.clone())?
    };
    wav::save_array1_f32_as_wav(&out.0, cli.wav, None)?;

    println!("Finished!");
    Ok(())
}
