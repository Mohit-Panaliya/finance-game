#[derive(rust_embed::Embed)]
#[folder = "frontend/dist/"]
pub struct Assets;

/// Config files (default.yaml + production.yaml) for the Android build,
/// which has no on-disk config/ directory.
#[derive(rust_embed::Embed)]
#[folder = "config/"]
pub struct ConfigAssets;
