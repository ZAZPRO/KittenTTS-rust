use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Cursor},
    path::Path,
};

use ndarray::{Array1, Array2, ArrayView1, Axis, s};
use npyz::npz::NpzArchive;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
};
use phonemize::Phonemizer;
use thiserror::Error;

pub mod phonemize;
pub mod wav;

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

pub type KittenTokens = HashMap<char, i64>;

#[derive(Debug)]
pub struct KittenModel {
    model: Session,
    voice: Array1<f32>,
    phonemizer: Phonemizer,
    tokens: KittenTokens,
}

impl KittenModel {
    pub fn get_tokens() -> KittenTokens {
        HashMap::from([
            ('$', 0),
            (';', 1),
            (':', 2),
            (',', 3),
            ('.', 4),
            ('!', 5),
            ('?', 6),
            ('¡', 7),
            ('¿', 8),
            ('—', 9),
            ('…', 10),
            ('"', 11),
            ('«', 12),
            ('»', 13),
            ('"', 14),
            ('"', 15),
            (' ', 16),
            ('A', 17),
            ('B', 18),
            ('C', 19),
            ('D', 20),
            ('E', 21),
            ('F', 22),
            ('G', 23),
            ('H', 24),
            ('I', 25),
            ('J', 26),
            ('K', 27),
            ('L', 28),
            ('M', 29),
            ('N', 30),
            ('O', 31),
            ('P', 32),
            ('Q', 33),
            ('R', 34),
            ('S', 35),
            ('T', 36),
            ('U', 37),
            ('V', 38),
            ('W', 39),
            ('X', 40),
            ('Y', 41),
            ('Z', 42),
            ('a', 43),
            ('b', 44),
            ('c', 45),
            ('d', 46),
            ('e', 47),
            ('f', 48),
            ('g', 49),
            ('h', 50),
            ('i', 51),
            ('j', 52),
            ('k', 53),
            ('l', 54),
            ('m', 55),
            ('n', 56),
            ('o', 57),
            ('p', 58),
            ('q', 59),
            ('r', 60),
            ('s', 61),
            ('t', 62),
            ('u', 63),
            ('v', 64),
            ('w', 65),
            ('x', 66),
            ('y', 67),
            ('z', 68),
            ('ɑ', 69),
            ('ɐ', 70),
            ('ɒ', 71),
            ('æ', 72),
            ('ɓ', 73),
            ('ʙ', 74),
            ('β', 75),
            ('ɔ', 76),
            ('ɕ', 77),
            ('ç', 78),
            ('ɗ', 79),
            ('ɖ', 80),
            ('ð', 81),
            ('ʤ', 82),
            ('ə', 83),
            ('ɘ', 84),
            ('ɚ', 85),
            ('ɛ', 86),
            ('ɜ', 87),
            ('ɝ', 88),
            ('ɞ', 89),
            ('ɟ', 90),
            ('ʄ', 91),
            ('ɡ', 92),
            ('ɠ', 93),
            ('ɢ', 94),
            ('ʛ', 95),
            ('ɦ', 96),
            ('ɧ', 97),
            ('ħ', 98),
            ('ɥ', 99),
            ('ʜ', 100),
            ('ɨ', 101),
            ('ɪ', 102),
            ('ʝ', 103),
            ('ɭ', 104),
            ('ɬ', 105),
            ('ɫ', 106),
            ('ɮ', 107),
            ('ʟ', 108),
            ('ɱ', 109),
            ('ɯ', 110),
            ('ɰ', 111),
            ('ŋ', 112),
            ('ɳ', 113),
            ('ɲ', 114),
            ('ɴ', 115),
            ('ø', 116),
            ('ɵ', 117),
            ('ɸ', 118),
            ('θ', 119),
            ('œ', 120),
            ('ɶ', 121),
            ('ʘ', 122),
            ('ɹ', 123),
            ('ɺ', 124),
            ('ɾ', 125),
            ('ɻ', 126),
            ('ʀ', 127),
            ('ʁ', 128),
            ('ɽ', 129),
            ('ʂ', 130),
            ('ʃ', 131),
            ('ʈ', 132),
            ('ʧ', 133),
            ('ʉ', 134),
            ('ʊ', 135),
            ('ʋ', 136),
            ('ⱱ', 137),
            ('ʌ', 138),
            ('ɣ', 139),
            ('ɤ', 140),
            ('ʍ', 141),
            ('χ', 142),
            ('ʎ', 143),
            ('ʏ', 144),
            ('ʑ', 145),
            ('ʐ', 146),
            ('ʒ', 147),
            ('ʔ', 148),
            ('ʡ', 149),
            ('ʕ', 150),
            ('ʢ', 151),
            ('ǀ', 152),
            ('ǁ', 153),
            ('ǂ', 154),
            ('ǃ', 155),
            ('ˈ', 156),
            ('ˌ', 157),
            ('ː', 158),
            ('ˑ', 159),
            ('ʼ', 160),
            ('ʴ', 161),
            ('ʰ', 162),
            ('ʱ', 163),
            ('ʲ', 164),
            ('ʷ', 165),
            ('ˠ', 166),
            ('ˤ', 167),
            ('˞', 168),
            ('↓', 169),
            ('↑', 170),
            ('→', 171),
            ('↗', 172),
            ('↘', 173),
            ('\'', 174),
            ('̩', 175),
            ('\'', 176),
            ('ᵻ', 177),
        ])
    }

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
        let phonemizer = Phonemizer::from_file(dictionary_path)
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;

        Self::new(voice, &mut voices_npz, model, phonemizer)
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

        let phonemizer = Phonemizer::new().map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        Self::new(voice, &mut voices_npz, model, phonemizer)
    }

    pub fn new<R: io::Read + io::Seek>(
        voice: KittenVoice,
        npz: &mut NpzArchive<R>,
        model: Session,
        phonemizer: Phonemizer,
    ) -> Result<Self, KittenError> {
        let voice_string = voice.to_string();
        let npy = npz
            .by_name(voice_string.as_str())
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?;
        let voice_raw_array = if let Some(voice_raw) = npy {
            voice_raw
        } else {
            return Err(KittenError::ModelLoad(
                "Failed to load npy voice file from npz archive".to_string(),
            ));
        };

        let voice_data: Array1<f32> = voice_raw_array
            .data::<f32>()
            .map_err(|e| KittenError::ModelLoad(e.to_string()))?
            .flatten()
            .collect();
        let tokens = KittenModel::get_tokens();

        Ok(Self {
            model,
            voice: voice_data,
            phonemizer,
            tokens,
        })
    }

    pub fn generate(&mut self, text: String) -> Result<(Array1<f32>, Array1<i64>), KittenError> {
        let phonems: Vec<String> = text
            .split_whitespace()
            .flat_map(|word| self.phonemizer.phonemize(word))
            .collect();
        let phonemized = phonems.join(" ");
        self.generate_from_phonems(phonemized)
    }

    pub fn generate_from_phonems(
        &mut self,
        phonems: String,
    ) -> Result<(Array1<f32>, Array1<i64>), KittenError> {
        let text_array: Array1<i64> = phonems
            .chars()
            .flat_map(|c| self.tokens.get(&c))
            .cloned()
            .collect();

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

        let mut padded = Array1::zeros(waveform.len() + 2);
        padded
            .slice_mut(s![1..waveform.len() + 1])
            .assign(&waveform);

        Ok((padded, duration.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::wav::save_array1_f32_as_wav;

    use super::*;

    #[test]
    fn model_files() {
        let res = KittenModel::model_from_files(
            "./model-files/kitten_tts_nano_v0_1.onnx",
            "./model-files/voices.npz",
            "./model-files/cmu.dict",
            KittenVoice::default(),
        );
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn model_builtin() {
        let res = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn generate_from_phonems() {
        let model = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(model.is_ok(), true);
        let res = model.unwrap().generate_from_phonems(
            "ðɪs haɪ kwɔlᵻɾi tiːtiːɛs mɑːdəl wɜːks wɪðaʊt ɐ dʒiːpiːjuː ".to_string(),
        );
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn generate() {
        let model = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(model.is_ok(), true);
        let res = model
            .unwrap()
            .generate("This high quality TTS model works without a GPU".to_string());
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

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("out.wav");
        let res = save_array1_f32_as_wav(&waveform, file_path, None);
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn save_from_phonems() {
        let model = KittenModel::model_builtin(KittenVoice::default());
        assert_eq!(model.is_ok(), true);
        let inference = model.unwrap().generate_from_phonems(
            "ðɪs haɪ kwɔlᵻɾi tiːtiːɛs mɑːdəl wɜːks wɪðaʊt ɐ dʒiːpiːjuː ".to_string(),
        );
        assert_eq!(inference.is_ok(), true);
        let (waveform, _) = inference.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("phonems.wav");
        let res = save_array1_f32_as_wav(&waveform, file_path, None);
        assert_eq!(res.is_ok(), true);
    }
}
