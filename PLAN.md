# music-tagger — Plan de implementación

CLI en Rust que etiqueta archivos de audio (FLAC/MP3/AIFF/WAV) con año, década,
género y sub-género usando voting multi-fuente con umbral de confianza configurable.

**Stack**: Rust 2021 · tokio (HTTP async) · rayon (CPU) · lofty · ort (ONNX) · SQLite WAL

---

## Cards

- [x] **Card 01 — Project scaffold**
  Cargo.toml con todas las dependencias, estructura de carpetas, .env.example,
  src/models.rs con structs base, src/config.rs leyendo .env con dotenvy.
  `cargo check` debe pasar sin errores.
  Dependencias: lofty, ort, musicbrainz_rs, rusty-chromaprint, tokio, rayon,
  rusqlite, r2d2, r2d2_sqlite, governor, clap, reqwest, serde, serde_json,
  indicatif, colored, backoff, anyhow, thiserror, dotenvy.

- [ ] **Card 02 — SQLite cache layer**
  Schema: track_cache (fingerprint PK, TTL 90d), genre_cache (genre_slug PK, TTL 180d),
  mb_genre_mapping (source_tag PK). WAL mode + r2d2 pool.
  cache.rs: struct CachePool wrapping Arc<r2d2::Pool>, métodos get_track/set_track/get_genre/set_genre.

- [ ] **Card 03 — MB Genre Mapping bootstrap**
  CSV bundleado via include_str!("../../data/mb_genre_mapping.csv").
  mb_mapping.rs: fn bootstrap(pool) inserta en SQLite si tabla vacía.
  fn lookup(pool, source_tag) → Option<MbGenre>: normaliza a lowercase, busca en SQLite.
  Ejecutar bootstrap() en main() antes de procesar archivos.

- [ ] **Card 04 — Source: MusicBrainz**
  musicbrainz_rs: recording lookup con genres+tags+first-release-date.
  User-Agent: "music-tagger/1.0 (contacto@email.com)". Rate limit 1req/s built-in.
  Retornar SourceResult. Si falla → None, nunca panic.

- [ ] **Card 05 — Source: Discogs**
  2 req/track obligatorio: GET /database/search → results[0].id,
  luego GET /releases/{id} → genres[], styles[], year.
  Auth: "Authorization: Discogs token=TOKEN". Rate limit 1req/s (60/min, 2req/track).

- [ ] **Card 06 — Source: Last.fm**
  track.getInfo?artist=X&track=Y&api_key=Z&format=json.
  Filtrar tags de ruido: "seen live", "beautiful", "favorite", "amazing", etc.
  Conservar solo tags que matcheen géneros conocidos. Rate limit 5req/s.

- [ ] **Card 07 — Source: AcoustID**
  rusty-chromaprint fingerprint (Rust puro, sin fpcalc).
  POST api.acoustid.org/v2/lookup → MBID → pasar a MusicBrainz.
  Rate limit 3req/s. Si falla → None, sin panic.

- [ ] **Card 08 — Source: Wikipedia**
  GET /api/rest_v1/page/summary/{genre_slug}.
  Extraer parent_genre con regex "subgenre of ([A-Za-z ]+)".
  Extraer origin_decade con regex "originated in the (\d{4}s|\d{4})".
  Cache en genre_cache con TTL 180d. Rol: confidence boost, no fuente primaria.

- [ ] **Card 09 — Source: ONNX/Essentia**
  ort session con Discogs400 model (data/models/genre_discogs400.onnx).
  Inicializar UNA sola vez en main(), pasar por Arc.
  Input: mel spectrogram (128 mel bins, patches 128 frames).
  Output: Vec<f32> 400 probabilidades → top-1 género + top-1 subgénero.
  Si .onnx ausente → deshabilitar gracefully, loggear warning.

- [ ] **Card 10 — Voting logic**
  voter.rs: HashMap<String, f32> acumulando scores por valor candidato.
  WEIGHTS_TRACK y WEIGHTS_GENRE según tabla del spec.
  score_normalizado = score_ganador / score_max_posible.
  Si < CONFIDENCE_THRESHOLD (0.65) → needs_review = true.
  fn to_decade(year: u32) → String { format!("{}0s", year / 10) }.

- [ ] **Card 11 — Tag writer**
  writer.rs con lofty:
  MP3:  TagType::Id3v2       → TDRC, TCON, TXXX:SUBGENRE, TGRP
  FLAC: TagType::VorbisComments → DATE, GENRE, SUBGENRE, GROUPING
  AIFF: TagType::Id3v2       → igual que MP3
  WAV:  TagType::RiffInfo o Id3v2 según archivo.
  Nunca escribir si needs_review = true (salvo --force).

- [ ] **Card 12 — CLI interface**
  clap derive: comandos audit / tag / retry.
  Flags: --dry-run, --write, --force, --skip-existing,
         --confidence 0.65, --jobs 8, --no-essentia, --no-acoustid, --log ./run.json.
  Output por track: [✓] Written / [?] NEEDS_REVIEW / [-] Skipped.

- [ ] **Card 13 — Rate limiting + backoff**
  governor token bucket por fuente, Arc compartido entre rayon threads.
  MusicBrainz: 1req/s | Discogs: 1req/s | LastFm: 5req/s | AcoustID: 3req/s | Wikipedia: 1req/s.
  Retry exponencial en 429/503 con crate backoff. Máximo 3 reintentos, luego skip con log.

- [ ] **Card 14 — Tokio + Rayon bridge**
  rayon::par_iter() sobre Vec<PathBuf> para CPU (audio, voting).
  Dentro de cada rayon thread: tokio::runtime::Runtime::new()?.block_on(fetch_all_sources()).
  Arc<RateLimiter> y Arc<CachePool> compartidos entre todos los threads.

- [ ] **Card 15 — Integration tests**
  tests/integration_test.rs con fixtures FLAC/MP3 reales.
  Mock de APIs con wiremock. Verificar VoteResult correcto.
  Test: writer escribe tags → leer con lofty y verificar valores.
  Test: --dry-run no modifica el archivo. cargo test debe pasar.

---

## Reglas del workflow

1. Una card a la vez. No avanzar sin revisión.
2. Al terminar cada card: `cargo check` (o `cargo test` en Card 15) debe pasar.
3. Cada card termina con un commit convencional: `feat(card-01): project scaffold`.
4. Si una card falla 3 veces: escalar, no agregar más fixes encima.
