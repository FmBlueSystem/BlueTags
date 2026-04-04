use crate::models::{SourceName, SourceResult};
use std::path::Path;

// ort v2.0.0-rc.12 API — se activa cuando el modelo .onnx está presente
// El stub compila sin el modelo; la implementación real se habilita en Card 09b

pub struct EssentiaClassifier {
    _model_path: std::path::PathBuf,
}

impl EssentiaClassifier {
    /// Retorna None si el archivo no existe (graceful degradation).
    pub fn load(model_path: &Path) -> Option<Self> {
        if !model_path.exists() {
            eprintln!(
                "[essentia] modelo no encontrado en {}. Fuente deshabilitada.",
                model_path.display()
            );
            return None;
        }
        eprintln!("[essentia] modelo encontrado. Cargando...");
        Some(EssentiaClassifier {
            _model_path: model_path.to_path_buf(),
        })
    }

    /// Clasifica un archivo de audio. Retorna None si falla.
    pub fn analyze(&self, _audio_path: &Path) -> Option<SourceResult> {
        // TODO Card 09b: implementar inferencia ONNX con ort cuando el modelo esté disponible
        // Pipeline: symphonia decode → mel spectrogram → ort session.run() → top-1 genre
        None
    }
}
