use std::fs;

use anyhow::{Context, Result};
use image::Luma;
use qrcode::{
    EcLevel, QrCode,
    render::{svg, unicode},
};

use crate::{
    cli::{GlobalOptions, QrCommand, QrEcc},
    error, output,
};

pub fn run(cmd: &QrCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        QrCommand::Gen(args) => {
            let code = QrCode::with_error_correction_level(args.text.as_bytes(), ecc(args.ecc))
                .map_err(|err| error::msg(format!("failed to generate QR code: {err}")))?;
            if let Some(path) = &args.png {
                let image = code.render::<Luma<u8>>().min_dimensions(256, 256).build();
                image
                    .save(path)
                    .with_context(|| format!("failed to write PNG {}", path.display()))?;
                println!("{}", path.display());
                Ok(())
            } else if let Some(path) = &args.svg {
                let image = code.render::<svg::Color>().min_dimensions(256, 256).build();
                fs::write(path, image)
                    .with_context(|| format!("failed to write SVG {}", path.display()))?;
                println!("{}", path.display());
                Ok(())
            } else if global.json {
                output::print_json(&serde_json::json!({ "text": args.text }))
            } else {
                let image = code.render::<unicode::Dense1x2>().quiet_zone(true).build();
                println!("{image}");
                Ok(())
            }
        }
        QrCommand::Read(args) => {
            let img = image::open(&args.image_path)
                .with_context(|| format!("failed to open image {}", args.image_path.display()))?
                .to_luma8();
            let mut prepared = rqrr::PreparedImage::prepare(img);
            let grids = prepared.detect_grids();
            for grid in grids {
                if let Ok((_meta, content)) = grid.decode() {
                    return output::write_or_json(
                        global,
                        || {
                            println!("{content}");
                            Ok(())
                        },
                        &serde_json::json!({ "text": content }),
                    );
                }
            }
            Err(error::msg("no readable QR code found in image"))
        }
    }
}

fn ecc(value: QrEcc) -> EcLevel {
    match value {
        QrEcc::L => EcLevel::L,
        QrEcc::M => EcLevel::M,
        QrEcc::Q => EcLevel::Q,
        QrEcc::H => EcLevel::H,
    }
}
