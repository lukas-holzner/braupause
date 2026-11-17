//! Foto-Import: dreht nach EXIF-Ausrichtung, skaliert auf eine webfreundliche
//! Kantenlänge, entfernt Metadaten (inkl. GPS) durch Neu-Kodieren und legt die
//! Datei unter `assets/batches/<ID>/` ab. Pfade im JSON sind relativ zur Repo-Wurzel.

use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageReader};

pub const EDGE_CHOICES: [u32; 3] = [1024, 1600, 2048];
pub const DEFAULT_EDGE: u32 = 1600;
pub const JPEG_QUALITY: u8 = 82;
pub const FILE_FILTER: &[&str] = &["jpg", "jpeg", "png", "webp", "heic", "heif", "tif", "tiff", "gif"];

pub struct Imported {
    pub content_url: String,
    pub caption: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub source_name: String,
}

pub fn batch_dir(repo: &Path, batch_id: &str) -> PathBuf {
    repo.join("assets").join("batches").join(batch_id)
}

pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| FILE_FILTER.contains(&e.to_ascii_lowercase().as_str()))
}

/// Importiert eine Bilddatei in den Asset-Ordner des Suds.
pub fn import(repo: &Path, batch_id: &str, src: &Path, max_edge: u32) -> Result<Imported, String> {
    let source_name = src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let img = decode(src).map_err(|e| format!("{source_name}: {e}"))?;
    let img = if img.width().max(img.height()) > max_edge {
        img.resize(max_edge, max_edge, FilterType::Lanczos3)
    } else {
        img
    };
    let keep_png = has_transparency(&img);
    let ext = if keep_png { "png" } else { "jpg" };

    let dir = batch_dir(repo, batch_id);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let stem = src.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let file_name = unique_name(&dir, &slugify(&stem), ext);
    let target = dir.join(&file_name);

    // Erst in eine versteckte Teildatei schreiben und dann umbenennen, damit nie eine
    // halb geschriebene Datei unter dem endgültigen Namen sichtbar ist (Vorschau-Cache!).
    let partial = dir.join(format!(".{file_name}.part"));
    let write = || -> Result<(), String> {
        let file = fs::File::create(&partial).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        if keep_png {
            img.write_to(&mut writer, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        } else {
            img.to_rgb8()
                .write_with_encoder(JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY))
                .map_err(|e| e.to_string())?;
        }
        let file = writer.into_inner().map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&partial, &target).map_err(|e| e.to_string())
    };
    if let Err(e) = write() {
        let _ = fs::remove_file(&partial);
        return Err(format!("{source_name}: Speichern fehlgeschlagen: {e}"));
    }
    let bytes = fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
    Ok(Imported {
        content_url: format!("assets/batches/{batch_id}/{file_name}"),
        caption: caption_from_stem(&stem),
        width: img.width(),
        height: img.height(),
        bytes,
        source_name,
    })
}

fn decode(src: &Path) -> Result<DynamicImage, String> {
    match decode_with_image_crate(src) {
        Ok(img) => Ok(img),
        Err(e) => convert_with_sips(src).ok_or(e),
    }
}

fn decode_with_image_crate(src: &Path) -> Result<DynamicImage, String> {
    let reader = ImageReader::open(src)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut decoder = reader.into_decoder().map_err(|e| format!("Format nicht unterstützt ({e})"))?;
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    Ok(img)
}

/// Fallback für Formate wie HEIC (iPhone): macOS-Bordmittel `sips` wandelt nach JPEG.
fn convert_with_sips(src: &Path) -> Option<DynamicImage> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("braupause-import-{}-{}.jpg", std::process::id(), rand_suffix()));
    let ok = std::process::Command::new("sips")
        .args(["-s", "format", "jpeg", "-s", "formatOptions", "best"])
        .arg(src)
        .arg("--out")
        .arg(&tmp)
        .output()
        .is_ok_and(|o| o.status.success());
    let img = if ok { decode_with_image_crate(&tmp).ok() } else { None };
    let _ = fs::remove_file(&tmp);
    img
}

fn rand_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn has_transparency(img: &DynamicImage) -> bool {
    img.color().has_alpha() && img.to_rgba8().pixels().any(|p| p.0[3] < 255)
}

/// `Jolly Keg Blow-Off.JPG` → `jolly-keg-blow-off`
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c if c.is_ascii_alphanumeric() => out.push(c),
            _ => {
                if !out.is_empty() && !out.ends_with('-') {
                    out.push('-');
                }
            }
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() { "foto".into() } else { out }
}

fn unique_name(dir: &Path, stem: &str, ext: &str) -> String {
    let mut name = format!("{stem}.{ext}");
    let mut n = 2;
    while dir.join(&name).exists() {
        name = format!("{stem}-{n}.{ext}");
        n += 1;
    }
    name
}

/// Bildunterschrift-Vorschlag aus dem Dateinamen; Kamera-Namen wie `IMG_1234` ergeben "".
pub fn caption_from_stem(stem: &str) -> String {
    let upper = stem.to_ascii_uppercase();
    let camera = ["IMG", "DSC", "DSCF", "PXL", "DCIM", "PHOTO", "SCREENSHOT", "IMAGE"]
        .iter()
        .any(|p| upper.starts_with(p))
        && stem.chars().filter(char::is_ascii_digit).count() >= 3;
    if camera || stem.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_') {
        return String::new();
    }
    let text = stem.replace(['-', '_'], " ");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = text.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Bildgrößen-Angabe ohne komplettes Dekodieren.
pub fn dimensions(path: &Path) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok()
}

/// Dateien im Asset-Ordner, die von keinem Bild-Eintrag referenziert werden.
pub fn orphans(repo: &Path, batch_id: &str, referenced: &[String]) -> Vec<String> {
    let Ok(entries) = fs::read_dir(batch_dir(repo, batch_id)) else { return Vec::new() };
    let mut out: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && is_image_file(&e.path()))
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| format!("assets/batches/{batch_id}/{}", e.file_name().to_string_lossy()))
        .filter(|url| !referenced.contains(url))
        .collect();
    out.sort();
    out
}

pub fn human_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_captions() {
        assert_eq!(slugify("Jolly Keg Blow-Off"), "jolly-keg-blow-off");
        assert_eq!(slugify("Gärprobe süß"), "gaerprobe-suess");
        assert_eq!(slugify("___"), "foto");
        assert_eq!(caption_from_stem("IMG_4711"), "");
        assert_eq!(caption_from_stem("PXL_20261117_101010"), "");
        assert_eq!(caption_from_stem("jungmet-probe_im glas"), "Jungmet probe im glas");
    }

    #[test]
    fn import_scales_and_reencodes() {
        let base = std::env::temp_dir().join(format!("braupause-img-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        // Großes Querformat-Foto
        let src = base.join("Großes Foto.png");
        image::RgbImage::from_fn(3000, 2000, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90]))
            .save(&src)
            .unwrap();
        let a = import(&base, "MET-2026-09", &src, 1600).unwrap();
        assert_eq!((a.width, a.height), (1600, 1067));
        assert_eq!(a.content_url, "assets/batches/MET-2026-09/grosses-foto.jpg");
        assert_eq!(dimensions(&base.join(&a.content_url)), Some((1600, 1067)));
        // Zweiter Import: eindeutiger Name, kleines Bild wird nicht vergrößert
        let small = base.join("Großes Foto.jpg");
        image::RgbImage::new(400, 300).save(&small).unwrap();
        let b = import(&base, "MET-2026-09", &small, 1600).unwrap();
        assert_eq!(b.content_url, "assets/batches/MET-2026-09/grosses-foto-2.jpg");
        assert_eq!((b.width, b.height), (400, 300));
        // Transparenz bleibt als PNG erhalten
        let logo = base.join("logo.png");
        image::RgbaImage::from_pixel(50, 50, image::Rgba([0, 0, 0, 0])).save(&logo).unwrap();
        assert!(import(&base, "MET-2026-09", &logo, 1600).unwrap().content_url.ends_with("logo.png"));
        // Kein Bild
        let txt = base.join("notiz.txt");
        fs::write(&txt, "kein Bild").unwrap();
        assert!(import(&base, "MET-2026-09", &txt, 1600).is_err());
        assert!(
            fs::read_dir(batch_dir(&base, "MET-2026-09"))
                .unwrap()
                .all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".part")),
            "keine Teildateien übrig"
        );
        let referenced = vec![a.content_url.clone()];
        assert_eq!(orphans(&base, "MET-2026-09", &referenced).len(), 2);
        let _ = fs::remove_dir_all(&base);
    }

    /// iPhone-Fotos (HEIC) laufen über den sips-Fallback.
    #[cfg(target_os = "macos")]
    #[test]
    fn heic_via_sips() {
        let base = std::env::temp_dir().join(format!("braupause-heic-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let jpg = base.join("quelle.jpg");
        image::RgbImage::from_pixel(2400, 1800, image::Rgb([200, 120, 40])).save(&jpg).unwrap();
        let heic = base.join("IMG_0042.HEIC");
        let ok = std::process::Command::new("sips")
            .args(["-s", "format", "heic"])
            .arg(&jpg)
            .arg("--out")
            .arg(&heic)
            .output()
            .is_ok_and(|o| o.status.success());
        if !ok {
            eprintln!("sips kann kein HEIC schreiben – Test übersprungen");
            return;
        }
        let r = import(&base, "MET-2026-09", &heic, 1024).unwrap();
        assert_eq!((r.width, r.height), (1024, 768));
        assert_eq!(r.content_url, "assets/batches/MET-2026-09/img-0042.jpg");
        assert_eq!(r.caption, "");
        let _ = fs::remove_dir_all(&base);
    }
}
