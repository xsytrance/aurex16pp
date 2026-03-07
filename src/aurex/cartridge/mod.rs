use crate::aurex::dma::command::{DmaCommand, VramRegion};
use crate::aurex::dma::controller::DmaController;
use crate::aurex::ppu::vram::Vram;
use crate::aurex::wram::Wram;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DMA_PER_FRAME: usize = 4;
const STAGING_START: usize = 0x20000;
const STAGING_CHUNK_BYTES: usize = 4096;

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

impl CartridgeRuntime {
    pub fn discover_default() -> Option<Self> {
        let manifest_path = Path::new("cartridges/default/manifest.txt");
        match Self::from_manifest(manifest_path) {
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

    fn from_manifest(path: &Path) -> Result<Self, String> {
        let root = path
            .parent()
            .ok_or_else(|| "manifest has no parent directory".to_string())?;
        let text = fs::read_to_string(path)
            .map_err(|e| format!("failed to read manifest {}: {e}", path.display()))?;

        let mut name = String::from("unnamed");
        let mut uploads = Vec::new();

        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("name=") {
                name = rest.trim().to_string();
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

            return Err(format!(
                "manifest line {}: unknown entry '{}",
                line_no + 1,
                line
            ));
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
