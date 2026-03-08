use crate::aurex::dma::command::{DmaCommand, VramRegion};
use crate::aurex::dma::controller::DmaController;
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DMA_PER_FRAME: usize = 4;
const STAGING_START: usize = 0x20000;
const STAGING_CHUNK_BYTES: usize = 4096;
const MAX_UPLOAD_BYTES: usize = 384 * 1024;

const MANIFEST_KEYS: &[ManifestKeySpec] = &[
    ManifestKeySpec {
        key: "name",
        kind: ManifestKeyKind::OptionalSingle,
    },
    ManifestKeySpec {
        key: "game_id",
        kind: ManifestKeyKind::RequiredSingle,
    },
    ManifestKeySpec {
        key: "upload",
        kind: ManifestKeyKind::RequiredRepeat,
    },
];

#[derive(Copy, Clone)]
struct ManifestKeySpec {
    key: &'static str,
    kind: ManifestKeyKind,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ManifestKeyKind {
    OptionalSingle,
    RequiredSingle,
    RequiredRepeat,
}

#[derive(Debug)]
pub enum CartridgeResolveError {
    MissingManifest,
    InvalidManifest(String),
}

#[derive(Clone)]
struct Upload {
    region: VramRegion,
    dst_offset: usize,
    data: Vec<u8>,
    cursor: usize,
}

pub struct CartridgeRuntime {
    name: String,
    uploads: Vec<Upload>,
}

#[derive(Debug)]
pub struct CartridgeAuditEntry {
    pub cartridge_id: String,
    pub ok: bool,
    pub issue: Option<String>,
}

#[derive(Debug, Default)]
pub struct CartridgeAuditReport {
    pub entries: Vec<CartridgeAuditEntry>,
}

impl CartridgeAuditReport {
    pub fn valid_count(&self) -> usize {
        self.entries.iter().filter(|e| e.ok).count()
    }

    pub fn invalid_count(&self) -> usize {
        self.entries.len().saturating_sub(self.valid_count())
    }

    pub fn all_valid(&self) -> bool {
        self.invalid_count() == 0
    }

    pub fn to_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
        }

        let mut out = String::new();
        out.push_str("{\"valid\":");
        out.push_str(&self.valid_count().to_string());
        out.push_str(",\"invalid\":");
        out.push_str(&self.invalid_count().to_string());
        out.push_str(",\"entries\":[");

        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"cartridge_id\":\"");
            out.push_str(&esc(&e.cartridge_id));
            out.push_str("\",\"ok\":");
            out.push_str(if e.ok { "true" } else { "false" });
            out.push_str(",\"issue\":");
            match &e.issue {
                Some(issue) => {
                    out.push('"');
                    out.push_str(&esc(issue));
                    out.push('"');
                }
                None => out.push_str("null"),
            }
            out.push('}');
        }

        out.push_str("]}");
        out
    }
}

impl CartridgeRuntime {
    pub fn from_cartridge_id(cartridge_id: &str) -> Result<Self, CartridgeResolveError> {
        let manifest_path = Path::new("cartridges")
            .join(cartridge_id)
            .join("manifest.txt");

        if !manifest_path.exists() {
            return Err(CartridgeResolveError::MissingManifest);
        }

        Self::from_manifest_with_expected_id(&manifest_path, Some(cartridge_id))
            .map_err(CartridgeResolveError::InvalidManifest)
    }

    pub fn discover_default() -> Option<Self> {
        let manifest_path = Path::new("cartridges/default/manifest.txt");
        match Self::from_manifest_with_expected_id(manifest_path, None) {
            Ok(cart) => Some(cart),
            Err(err) => {
                eprintln!(
                    "No cartridge loaded from {}: {err}",
                    manifest_path.display()
                );
                None
            }
        }
    }

    pub fn audit_cartridges_root(root: &Path) -> CartridgeAuditReport {
        let mut report = CartridgeAuditReport::default();

        let mut dirs = Vec::new();
        if let Ok(read_dir) = fs::read_dir(root) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }

        dirs.sort();

        for dir in dirs {
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let manifest = dir.join("manifest.txt");
            if !manifest.exists() {
                report.entries.push(CartridgeAuditEntry {
                    cartridge_id: name.to_string(),
                    ok: false,
                    issue: Some("missing manifest.txt".to_string()),
                });
                continue;
            }

            match Self::from_manifest_with_expected_id(&manifest, Some(name)) {
                Ok(_) => report.entries.push(CartridgeAuditEntry {
                    cartridge_id: name.to_string(),
                    ok: true,
                    issue: None,
                }),
                Err(err) => report.entries.push(CartridgeAuditEntry {
                    cartridge_id: name.to_string(),
                    ok: false,
                    issue: Some(err),
                }),
            }
        }

        report
    }

    pub fn audit_default_cartridges() -> CartridgeAuditReport {
        Self::audit_cartridges_root(Path::new("cartridges"))
    }

    fn from_manifest_with_expected_id(
        path: &Path,
        expected_game_id: Option<&str>,
    ) -> Result<Self, String> {
        let root = path
            .parent()
            .ok_or_else(|| "manifest has no parent directory".to_string())?;
        let text = fs::read_to_string(path)
            .map_err(|e| format!("failed to read manifest {}: {e}", path.display()))?;

        let mut name = String::from("unnamed");
        let mut game_id: Option<String> = None;
        let mut uploads = Vec::new();
        let mut seen_name = false;
        let mut seen_game_id = false;

        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("name=") {
                if seen_name {
                    return Err(format!(
                        "manifest line {}: duplicate name field",
                        line_no + 1
                    ));
                }
                seen_name = true;
                name = rest.trim().to_string();
                continue;
            }

            if let Some(rest) = line.strip_prefix("game_id=") {
                if seen_game_id {
                    return Err(format!(
                        "manifest line {}: duplicate game_id field",
                        line_no + 1
                    ));
                }
                seen_game_id = true;
                let id = rest.trim();
                if id.is_empty() {
                    return Err(format!("manifest line {}: empty game_id", line_no + 1));
                }

                let valid = id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
                if !valid {
                    return Err(format!(
                        "manifest line {}: invalid game_id '{}'",
                        line_no + 1,
                        id
                    ));
                }

                game_id = Some(id.to_string());
                continue;
            }

            if let Some(rest) = line.strip_prefix("upload=") {
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() != 3 {
                    return Err(format!(
                        "manifest line {}: upload expects 3 fields region,dst,file",
                        line_no + 1
                    ));
                }

                let region = parse_region(parts[0]).ok_or_else(|| {
                    format!(
                        "manifest line {}: invalid region '{}",
                        line_no + 1,
                        parts[0]
                    )
                })?;

                let dst_offset = parts[1].parse::<usize>().map_err(|_| {
                    format!(
                        "manifest line {}: invalid dst offset '{}'",
                        line_no + 1,
                        parts[1]
                    )
                })?;

                let mut file_path = PathBuf::from(root);
                file_path.push(parts[2]);
                let data = fs::read(&file_path).map_err(|e| {
                    format!(
                        "manifest line {}: failed reading {}: {e}",
                        line_no + 1,
                        file_path.display()
                    )
                })?;

                uploads.push(Upload {
                    region,
                    dst_offset,
                    data,
                    cursor: 0,
                });
                continue;
            }

            if let Some((candidate_key, _)) = line.split_once('=') {
                if !manifest_key_known(candidate_key.trim()) {
                    return Err(format!(
                        "manifest line {}: unknown key '{}'",
                        line_no + 1,
                        candidate_key.trim()
                    ));
                }
            }

            return Err(format!(
                "manifest line {}: unknown entry '{}'",
                line_no + 1,
                line
            ));
        }

        validate_manifest_key_requirements(seen_game_id, uploads.is_empty())?;

        for upload in &uploads {
            validate_upload_budget(upload)?;
        }

        if let Some(expected) = expected_game_id {
            let parsed =
                game_id.ok_or_else(|| "manifest missing required game_id field".to_string())?;
            if parsed != expected {
                return Err(format!(
                    "manifest game_id '{}' does not match requested cartridge_id '{}'",
                    parsed, expected
                ));
            }
        }

        if uploads.is_empty() {
            return Err("manifest contains no upload entries".to_string());
        }

        Ok(Self { name, uploads })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn queue_frame_dma(&mut self, dma: &mut DmaController, wram: &mut Wram, vram: &Vram) {
        let mut issued = 0usize;

        for upload in &mut self.uploads {
            while issued < MAX_DMA_PER_FRAME && upload.cursor < upload.data.len() {
                let remain = upload.data.len() - upload.cursor;
                let chunk_len = remain.min(STAGING_CHUNK_BYTES);

                let src_offset = STAGING_START + issued * STAGING_CHUNK_BYTES;
                let src = &upload.data[upload.cursor..upload.cursor + chunk_len];
                wram.memory_mut()[src_offset..src_offset + chunk_len].copy_from_slice(src);

                let cmd = DmaCommand::new(
                    upload.region,
                    src_offset,
                    upload.dst_offset + upload.cursor,
                    chunk_len,
                );
                if !dma.request(cmd, wram, vram) {
                    return;
                }

                upload.cursor += chunk_len;
                issued += 1;
            }

            if issued >= MAX_DMA_PER_FRAME {
                break;
            }
        }
    }

    pub fn uploads_complete(&self) -> bool {
        self.uploads.iter().all(|u| u.cursor >= u.data.len())
    }
}

fn manifest_key_known(key: &str) -> bool {
    MANIFEST_KEYS.iter().any(|spec| spec.key == key)
}

fn validate_manifest_key_requirements(
    seen_game_id: bool,
    upload_missing: bool,
) -> Result<(), String> {
    for key in MANIFEST_KEYS {
        match key.kind {
            ManifestKeyKind::RequiredSingle if key.key == "game_id" && !seen_game_id => {
                return Err("manifest missing required game_id field".to_string());
            }
            ManifestKeyKind::RequiredRepeat if key.key == "upload" && upload_missing => {
                return Err("manifest contains no upload entries".to_string());
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_upload_budget(upload: &Upload) -> Result<(), String> {
    if upload.data.len() > MAX_UPLOAD_BYTES {
        return Err(format!(
            "upload to {:?} exceeds max per-upload byte budget ({})",
            upload.region, MAX_UPLOAD_BYTES
        ));
    }

    let region_cap = region_capacity_bytes(upload.region);
    let end = upload
        .dst_offset
        .checked_add(upload.data.len())
        .ok_or_else(|| {
            format!(
                "upload to {:?} has overflowing destination range",
                upload.region
            )
        })?;
    if end > region_cap {
        return Err(format!(
            "upload to {:?} writes beyond region capacity (dst {} + bytes {} > cap {})",
            upload.region,
            upload.dst_offset,
            upload.data.len(),
            region_cap
        ));
    }

    if matches!(upload.region, VramRegion::Palettes)
        && (!upload.dst_offset.is_multiple_of(2) || !upload.data.len().is_multiple_of(2))
    {
        return Err("palette upload requires even dst offset and even byte count".to_string());
    }

    Ok(())
}

fn region_capacity_bytes(region: VramRegion) -> usize {
    match region {
        VramRegion::BgTiles => 384 * 1024,
        VramRegion::Bg0Tilemap => 64 * 64 * 2,
        VramRegion::Bg1Tilemap => 64 * 64 * 2,
        VramRegion::SpriteTiles => 384 * 1024,
        VramRegion::Mode7Tex => 64 * 1024,
        VramRegion::Palettes => 4096 * 2,
        VramRegion::AudioRam | VramRegion::Reserved => 0,
    }
}

fn parse_region(s: &str) -> Option<VramRegion> {
    match s {
        "BgTiles" => Some(VramRegion::BgTiles),
        "Bg0Tilemap" => Some(VramRegion::Bg0Tilemap),
        "Bg1Tilemap" => Some(VramRegion::Bg1Tilemap),
        "SpriteTiles" => Some(VramRegion::SpriteTiles),
        "Mode7Tex" => Some(VramRegion::Mode7Tex),
        "Palettes" => Some(VramRegion::Palettes),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CartridgeResolveError, CartridgeRuntime};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("aurex_cart_test_{stamp}"))
    }

    #[test]
    fn from_cartridge_id_requires_game_id_field() {
        let old = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("cartridges/test_game")).expect("mkdir");
        fs::write(
            root.join("cartridges/test_game/manifest.txt"),
            "name=TEST\nupload=BgTiles,0,tile.bin\n",
        )
        .expect("write manifest");
        fs::write(root.join("cartridges/test_game/tile.bin"), [0u8; 32]).expect("write tile");

        std::env::set_current_dir(&root).expect("chdir root");
        let result = CartridgeRuntime::from_cartridge_id("test_game");
        std::env::set_current_dir(old).expect("restore cwd");

        assert!(matches!(
            result,
            Err(CartridgeResolveError::InvalidManifest(_))
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_cartridges_reports_valid_and_invalid_entries() {
        let root = unique_temp_dir();
        let carts = root.join("cartridges");
        fs::create_dir_all(carts.join("good_game")).expect("mkdir good");
        fs::create_dir_all(carts.join("bad_game")).expect("mkdir bad");
        fs::create_dir_all(carts.join("missing_manifest")).expect("mkdir missing");

        fs::write(
            carts.join("good_game/manifest.txt"),
            "name=GOOD
game_id=good_game
upload=BgTiles,0,tile.bin
",
        )
        .expect("write good manifest");
        fs::write(carts.join("good_game/tile.bin"), [0u8; 32]).expect("write good tile");

        fs::write(
            carts.join("bad_game/manifest.txt"),
            "name=BAD
game_id=wrong_id
upload=BgTiles,0,tile.bin
",
        )
        .expect("write bad manifest");
        fs::write(carts.join("bad_game/tile.bin"), [0u8; 32]).expect("write bad tile");

        let report = CartridgeRuntime::audit_cartridges_root(&carts);
        assert_eq!(report.entries.len(), 3);
        assert_eq!(report.valid_count(), 1);
        assert_eq!(report.invalid_count(), 2);
        assert!(!report.all_valid());

        let json = report.to_json();
        assert!(json.contains("\"valid\":1"));
        assert!(json.contains("\"invalid\":2"));
        assert!(json.contains("\"cartridge_id\":\"good_game\""));

        let good = report
            .entries
            .iter()
            .find(|e| e.cartridge_id == "good_game")
            .expect("good entry");
        assert!(good.ok);

        let missing = report
            .entries
            .iter()
            .find(|e| e.cartridge_id == "missing_manifest")
            .expect("missing entry");
        assert!(!missing.ok);
        assert!(
            missing
                .issue
                .as_deref()
                .unwrap_or("")
                .contains("missing manifest")
        );

        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn from_cartridge_id_rejects_mismatched_game_id() {
        let old = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("cartridges/test_game")).expect("mkdir");
        fs::write(
            root.join("cartridges/test_game/manifest.txt"),
            "name=TEST\ngame_id=other_game\nupload=BgTiles,0,tile.bin\n",
        )
        .expect("write manifest");
        fs::write(root.join("cartridges/test_game/tile.bin"), [0u8; 32]).expect("write tile");

        std::env::set_current_dir(&root).expect("chdir root");
        let result = CartridgeRuntime::from_cartridge_id("test_game");
        std::env::set_current_dir(old).expect("restore cwd");

        assert!(matches!(
            result,
            Err(CartridgeResolveError::InvalidManifest(_))
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn from_manifest_rejects_upload_out_of_region_budget() {
        let old = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("cartridges/test_game")).expect("mkdir");
        fs::write(
            root.join("cartridges/test_game/manifest.txt"),
            "name=TEST\ngame_id=test_game\nupload=Palettes,8191,palette.bin\n",
        )
        .expect("write manifest");
        fs::write(root.join("cartridges/test_game/palette.bin"), [0u8; 2]).expect("write data");

        std::env::set_current_dir(&root).expect("chdir root");
        let result = CartridgeRuntime::from_cartridge_id("test_game");
        std::env::set_current_dir(old).expect("restore cwd");

        let err = match result {
            Err(CartridgeResolveError::InvalidManifest(err)) => err,
            _ => panic!("expected invalid manifest error"),
        };
        assert!(err.contains("palette upload requires even dst offset"));
        let _ = fs::remove_dir_all(root);
    }
}
