# Kitten TTS 😻 Rust fork 🦀

Kitten TTS is an open-source realistic text-to-speech model with just 15 million parameters, designed for lightweight deployment and high-quality voice synthesis.

*Currently in developer preview (Original author note)*

[Checkout original project git](https://github.com/KittenML/KittenTTS)


## ✨ Features

- **Ultra-lightweight**: Model size less than 25MB
- **CPU-optimized**: Runs without GPU on any device
- **High-quality voices**: Several premium voice options available
- **Fast inference**: Optimized for real-time speech synthesis

## 🦀 Fork Features

- CLI
- No hard espeak dependency
- Option of using any phonemizer


## 🚀 Quick Start

### Usage CLI
```
# Build package using Nix package manager
nix build
```

### Usage Lib

```
# Create development environment using Nix package manager
nix develop
```



 ### Basic Lib Usage 

```rust
let model = crate::KittenModel::model_builtin(crate::KittenVoice::default());
let inference = model
    .unwrap()
    .generate("This high quality TTS model works without a GPU".to_string());
let (waveform, _) = inference.unwrap();
let wav = crate::wav::save_array1_f32_as_wav(&waveform, "out/out.wav", None);
```
