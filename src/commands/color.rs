use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{ColorCommand, ColorFormat, GlobalOptions, PaletteScheme},
    error, output,
};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Copy, Debug)]
struct Hsl {
    h: f64,
    s: f64,
    l: f64,
}

pub fn run(cmd: &ColorCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        ColorCommand::Convert(args) => {
            let rgb = parse_color(&args.value)?;
            let converted = format_color(rgb, args.to);
            output::write_or_json(
                global,
                || {
                    println!("{converted}");
                    Ok(())
                },
                &serde_json::json!({ "output": converted }),
            )
        }
        ColorCommand::Palette(args) => {
            if args.count == 0 {
                return Err(error::msg("--count must be greater than zero"));
            }
            let base = parse_hex(&args.base_hex)?;
            let colors = palette(base, args.scheme, args.count)
                .into_iter()
                .map(format_hex)
                .collect::<Vec<_>>();
            output::write_or_json(
                global,
                || {
                    for color in &colors {
                        println!("{color}");
                    }
                    Ok(())
                },
                &serde_json::json!({ "colors": colors }),
            )
        }
    }
}

pub fn parse_color(value: &str) -> Result<Rgb> {
    let value = value.trim();
    if value.starts_with('#') || value.len() == 6 || value.len() == 3 {
        parse_hex(value)
    } else if let Some(inner) = value.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
        let nums = parse_numbers(inner)?;
        if nums.len() != 3 {
            return Err(error::msg("rgb() requires three components"));
        }
        Ok(Rgb {
            r: to_u8(nums[0])?,
            g: to_u8(nums[1])?,
            b: to_u8(nums[2])?,
        })
    } else if let Some(inner) = value.strip_prefix("hsl(").and_then(|v| v.strip_suffix(')')) {
        let nums = parse_numbers(&inner.replace('%', ""))?;
        if nums.len() != 3 {
            return Err(error::msg("hsl() requires three components"));
        }
        Ok(hsl_to_rgb(Hsl {
            h: nums[0],
            s: nums[1] / 100.0,
            l: nums[2] / 100.0,
        }))
    } else if let Some(inner) = value
        .strip_prefix("cmyk(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let nums = parse_numbers(&inner.replace('%', ""))?;
        if nums.len() != 4 {
            return Err(error::msg("cmyk() requires four components"));
        }
        Ok(cmyk_to_rgb(
            nums[0] / 100.0,
            nums[1] / 100.0,
            nums[2] / 100.0,
            nums[3] / 100.0,
        ))
    } else {
        Err(error::msg(
            "unsupported color format; expected hex, rgb(), hsl(), or cmyk()",
        ))
    }
}

fn parse_numbers(input: &str) -> Result<Vec<f64>> {
    input
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f64>()
                .map_err(|_| error::msg(format!("invalid number `{}`", part.trim())))
        })
        .collect()
}

fn to_u8(value: f64) -> Result<u8> {
    if (0.0..=255.0).contains(&value) {
        Ok(value.round() as u8)
    } else {
        Err(error::msg("RGB components must be between 0 and 255"))
    }
}

fn parse_hex(value: &str) -> Result<Rgb> {
    let value = value.trim().trim_start_matches('#');
    let expanded = match value.len() {
        3 => value.chars().flat_map(|ch| [ch, ch]).collect::<String>(),
        6 => value.to_string(),
        _ => return Err(error::msg("hex color must be #RGB or #RRGGBB")),
    };
    let bytes =
        hex::decode(expanded).map_err(|err| error::msg(format!("invalid hex color: {err}")))?;
    Ok(Rgb {
        r: bytes[0],
        g: bytes[1],
        b: bytes[2],
    })
}

fn format_color(rgb: Rgb, format: ColorFormat) -> String {
    match format {
        ColorFormat::Hex => format_hex(rgb),
        ColorFormat::Rgb => format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b),
        ColorFormat::Hsl => {
            let hsl = rgb_to_hsl(rgb);
            format!(
                "hsl({:.0}, {:.0}%, {:.0}%)",
                hsl.h,
                hsl.s * 100.0,
                hsl.l * 100.0
            )
        }
        ColorFormat::Cmyk => {
            let (c, m, y, k) = rgb_to_cmyk(rgb);
            format!(
                "cmyk({:.0}%, {:.0}%, {:.0}%, {:.0}%)",
                c * 100.0,
                m * 100.0,
                y * 100.0,
                k * 100.0
            )
        }
    }
}

fn format_hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

fn rgb_to_hsl(rgb: Rgb) -> Hsl {
    let r = f64::from(rgb.r) / 255.0;
    let g = f64::from(rgb.g) / 255.0;
    let b = f64::from(rgb.b) / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return Hsl { h: 0.0, s: 0.0, l };
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < f64::EPSILON {
        60.0 * (((g - b) / d) % 6.0)
    } else if (max - g).abs() < f64::EPSILON {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    Hsl {
        h: h.rem_euclid(360.0),
        s,
        l,
    }
}

fn hsl_to_rgb(hsl: Hsl) -> Rgb {
    let c = (1.0 - (2.0 * hsl.l - 1.0).abs()) * hsl.s;
    let h = hsl.h.rem_euclid(360.0) / 60.0;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = hsl.l - c / 2.0;
    Rgb {
        r: ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        g: ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        b: ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}

fn rgb_to_cmyk(rgb: Rgb) -> (f64, f64, f64, f64) {
    let r = f64::from(rgb.r) / 255.0;
    let g = f64::from(rgb.g) / 255.0;
    let b = f64::from(rgb.b) / 255.0;
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        (0.0, 0.0, 0.0, 1.0)
    } else {
        (
            (1.0 - r - k) / (1.0 - k),
            (1.0 - g - k) / (1.0 - k),
            (1.0 - b - k) / (1.0 - k),
            k,
        )
    }
}

fn cmyk_to_rgb(c: f64, m: f64, y: f64, k: f64) -> Rgb {
    Rgb {
        r: (255.0 * (1.0 - c) * (1.0 - k)).round().clamp(0.0, 255.0) as u8,
        g: (255.0 * (1.0 - m) * (1.0 - k)).round().clamp(0.0, 255.0) as u8,
        b: (255.0 * (1.0 - y) * (1.0 - k)).round().clamp(0.0, 255.0) as u8,
    }
}

fn palette(base: Rgb, scheme: PaletteScheme, count: usize) -> Vec<Rgb> {
    let hsl = rgb_to_hsl(base);
    (0..count)
        .map(|idx| {
            let offset = match scheme {
                PaletteScheme::Complementary => {
                    if idx % 2 == 0 {
                        0.0
                    } else {
                        180.0
                    }
                }
                PaletteScheme::Analogous => {
                    let center = (count.saturating_sub(1) as f64) / 2.0;
                    (idx as f64 - center) * 30.0
                }
                PaletteScheme::Triadic => [0.0, 120.0, 240.0][idx % 3],
            };
            hsl_to_rgb(Hsl {
                h: hsl.h + offset,
                ..hsl
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_red() {
        assert_eq!(format_hex(parse_color("#f00").unwrap()), "#ff0000");
    }
}
