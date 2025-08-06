use std::{
    fmt::Display,
    fs::File,
    io::{self, Cursor, Write},
    path::Path,
};

use ndarray::{Array1, Array2, ArrayView1, Axis};
use npyz::npz::NpzArchive;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
};
use phonemize::Phonemizer;
use thiserror::Error;

mod phonemize;

static MODEL: &[u8] = include_bytes!("../model-files/kitten_tts_nano_v0_1.onnx");
static VOICES: &[u8] = include_bytes!("../model-files/voices.npz");

#[derive(Error, Debug, Clone)]
pub enum KittenError {
    #[error("failed to load model: {0}")]
    ModelLoad(String),
    #[error("failed to execute model: {0}")]
    ModelExecute(String),
    #[error("failed to save model result: {0}")]
    ModelResultSave(String),
}

fn _save_wav_f32(data: &Array1<f32>, filename: &str, sample_rate: u32) -> Result<(), io::Error> {
    let mut file = File::create(format!("{filename}.wav"))?;

    let num_samples = data.len() as u32;
    let num_channels = 1u16;
    let bits_per_sample = 32u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = num_samples * (bits_per_sample as u32 / 8);
    let file_size = 36 + data_size;

    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&3u16.to_le_bytes())?;
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    for &sample in data {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub enum KittenVoice {
    TwoM,
    TwoF,
    ThreeM,
    ThreeF,
    FourM,
    FourF,
    #[default]
    FiveM,
    FiveF,
}

impl Display for KittenVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let voice_str = match self {
            KittenVoice::TwoM => "2-m",
            KittenVoice::TwoF => "2-f",
            KittenVoice::ThreeM => "3-m",
            KittenVoice::ThreeF => "3-f",
            KittenVoice::FourM => "4-m",
            KittenVoice::FourF => "4-f",
            KittenVoice::FiveM => "5-m",
            KittenVoice::FiveF => "5-f",
        };

        write!(f, "expr-voice-{voice_str}")
    }
}

#[derive(Debug)]
pub struct KittenModel {
    model: Session,
    voice: Array1<f32>,
    phonemizer: Phonemizer,
}

impl KittenModel {
    pub fn model_from_files<P: AsRef<Path>>(
        model_path: P,
        voices_path: P,
        dictionary_path: P,
        voice: KittenVoice,
    ) -> Result<Self, KittenError> {
        let model = Session::builder()
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        let mut voices_npz =
            NpzArchive::open(voices_path).map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        let voice_string = voice.to_string();
        let voice_npy = voices_npz
            .by_name(voice_string.as_str())
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        if voice_npy.is_none() {
            return Err(KittenError::ModelLoad(
                "Failed to load npy voice file from npz archive".to_string(),
            ));
        }

        let voice_data: Array1<f32> = voice_npy
            .unwrap()
            .data::<f32>()
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .flatten()
            .collect();

        let phonemizer = Phonemizer::from_file(dictionary_path)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;

        Ok(Self {
            model,
            voice: voice_data,
            phonemizer,
        })
    }

    pub fn model_builtin(voice: KittenVoice) -> Result<Self, KittenError> {
        let model = Session::builder()
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .commit_from_memory(MODEL)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        let mut reader = Cursor::new(VOICES);
        let mut voices_npz =
            NpzArchive::new(&mut reader).map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        let voice_string = voice.to_string();
        let voice_npy = voices_npz
            .by_name(voice_string.as_str())
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        if voice_npy.is_none() {
            return Err(KittenError::ModelLoad(
                "Failed to load npy voice file from npz archive".to_string(),
            ));
        }

        let voice_data: Array1<f32> = voice_npy
            .unwrap()
            .data::<f32>()
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .flatten()
            .collect();
        let phonemizer = Phonemizer::new().map_err(|e| KittenError::ModelLoad(e.to_string()))?;

        Ok(Self {
            model,
            voice: voice_data,
            phonemizer,
        })
    }

    pub fn generate(&mut self, text: String) -> Result<(Array1<f32>, Array1<i64>), KittenError> {
        let phonemized: String = text
            .split_whitespace()
            .map(|word| self.phonemizer.phonemize(word))
            .collect();

        let text_array: Array1<i64> = phonemized.chars().map(|c| c as i64).collect();
        let text_input: Array2<i64> = text_array.insert_axis(Axis(0));
        let text_tensor =
            Tensor::from_array(text_input).map_err(|e| KittenError::ModelExecute(e.to_string()))?;
        let style_input: Array2<f32> = self.voice.clone().insert_axis(Axis(0));
        let style_tensor = Tensor::from_array(style_input)
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?;
        let speed_tensor = Tensor::from_array(Array1::from_vec(vec![1.0_f32]))
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?;

        let outputs = self
            .model
            .run(ort::inputs![
            "input_ids" => text_tensor,
            "style" => style_tensor,
            "speed" => speed_tensor
            ])
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?;

        let waveform: ArrayView1<f32> = outputs["waveform"]
            .try_extract_array::<f32>()
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?
            .into_dimensionality()
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?;
        let duration: ArrayView1<i64> = outputs["duration"]
            .try_extract_array::<i64>()
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?
            .into_dimensionality()
            .map_err(|e| KittenError::ModelExecute(e.to_string()))?;

        Ok((waveform.to_owned(), duration.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_files() {
        let res = KittenModel::model_from_files(
            "./model-files/kitten_tts_nano_v0_1.onnx",
            "./model-files/voices.npz",
            "./model-files/cmudict.dict",
            KittenVoice::default(),
        );
        println!("{res:?}");
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn model_builtin() {
        let res = KittenModel::model_builtin(KittenVoice::default());
        println!("{res:?}");
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn generate() {
        let model = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(model.is_ok(), true);
        let res = model
            .unwrap()
            .generate("This high quality TTS model works without a GPU".to_string());
        println!("{res:?}");
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn save() {
        let model = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(model.is_ok(), true);
        let inference = model
            .unwrap()
            .generate("This high quality TTS model works without a GPU".to_string());
        assert_eq!(inference.is_ok(), true);
        let (waveform, _) = inference.unwrap();

        let res = _save_wav_f32(&waveform, "out/out", 22000);
        println!("{res:?}");
        assert_eq!(res.is_ok(), true);
    }
}
