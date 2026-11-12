//! Icon generator: downloads NPM icon packages, converts SVGs to path data,
//! and emits Rust source files with `pub const NAME: &str = "M12..."`.
//!
//! Run from the workspace root:
//!     cargo run -p icon-generator
//!     cargo run -p icon-generator -- material fontawesome
//!     cargo run -p icon-generator -- --root /path/to/workspace

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use roxmltree::{Document, Node};
use serde::Deserialize;
use tar::Archive;

// =============================================================================
// Configuration
// =============================================================================

#[derive(Debug, Clone)]
struct Style {
    tar_prefix: &'static str,
    class_name: &'static str,
    suffix: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct Pack {
    id: &'static str,
    project: &'static str,
    module: &'static str,
    root_struct: Option<&'static str>,
    npm: &'static str,
    license: &'static str,
    notice: &'static str,
    styles: &'static [Style],
}

static PACKS: &[Pack] = &[
    Pack {
        id: "material",
        project: "Material",
        module: "mud_egui_icons_material",
        root_struct: Some("MaterialIcons"),
        npm: "@material-design-icons/svg",
        license: "Apache License 2.0",
        notice: "Icons are copyright Google and licensed under the Apache License 2.0 \
                 (https://www.apache.org/licenses/LICENSE-2.0).",
        styles: &[
            Style { tar_prefix: "package/filled/", class_name: "Filled", suffix: None },
            Style { tar_prefix: "package/outlined/", class_name: "Outlined", suffix: None },
        ],
    },
    Pack {
        id: "fontawesome",
        project: "FontAwesome",
        module: "mud_egui_icons_font_awesome",
        root_struct: Some("FontAwesomeIcons"),
        npm: "@fortawesome/fontawesome-free",
        license: "CC BY 4.0",
        notice: "Font Awesome Free icons are copyright Fonticons, Inc. and licensed under CC BY 4.0, \
                 which REQUIRES attribution (https://fontawesome.com/license/free). \
                 Keep this notice with any distribution.",
        styles: &[
            Style { tar_prefix: "package/svgs/solid/", class_name: "Solid", suffix: None },
            Style { tar_prefix: "package/svgs/regular/", class_name: "Regular", suffix: None },
            Style { tar_prefix: "package/svgs/brands/", class_name: "Brands", suffix: None },
        ],
    },
    Pack {
        id: "fluent",
        project: "Fluent",
        module: "mud_egui_icons_fluent",
        root_struct: Some("FluentIcons"),
        npm: "@fluentui/svg-icons",
        license: "MIT",
        notice: "Fluent UI System Icons are copyright Microsoft Corporation and licensed under \
                 the MIT License (https://github.com/microsoft/fluentui-system-icons/blob/main/LICENSE). \
                 Only the 24px variants are used (20px when an icon has no 24px version).",
        styles: &[
            Style { tar_prefix: "package/icons/", class_name: "Regular", suffix: Some("regular") },
            Style { tar_prefix: "package/icons/", class_name: "Filled", suffix: Some("filled") },
        ],
    },
    Pack {
        id: "simpleicons",
        project: "SimpleIcons",
        module: "mud_egui_icons_simple_icons",
        root_struct: None,
        npm: "simple-icons",
        license: "CC0 1.0",
        notice: "Icons are released under CC0 1.0. The logos themselves are trademarks of their \
                 respective owners; use them only in line with each owner's brand guidelines.",
        styles: &[
            Style { tar_prefix: "package/icons/", class_name: "BrandIcons", suffix: None },
        ],
    },
];

const SIMPLE_ICONS_DATA: &str = "package/data/simple-icons.json";

// =============================================================================
// Entry point
// =============================================================================

fn main() {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut wanted: Vec<String> = Vec::new();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        if args[i].eq_ignore_ascii_case("--root") && i + 1 < args.len() {
            root = PathBuf::from(&args[i + 1]);
            i += 2;
        } else {
            wanted.push(args[i].to_lowercase());
            i += 1;
        }
    }

    let selected: Vec<&Pack> = if wanted.is_empty() || wanted.iter().any(|s| s == "all") {
        PACKS.iter().collect()
    } else {
        PACKS.iter().filter(|p| wanted.contains(&p.id.to_string())).collect()
    };

    if selected.is_empty() {
        eprintln!(
            "Nothing to do. Known packs: {}",
            PACKS.iter().map(|p| p.id).collect::<Vec<_>>().join(", ")
        );
        std::process::exit(2);
    }

    let cache = root.join("tools").join("icon-sources").join("cache");
    if let Err(e) = fs::create_dir_all(&cache) {
        eprintln!("Failed to create cache dir: {}", e);
        std::process::exit(1);
    }
    let _ = fs::write(cache.join(".gitignore"), "*\n");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .user_agent("Mud.Egui.Icons.Generator")
        .build()
        .expect("HTTP client");

    let mut failures = 0;
    for pack in &selected {
        if let Err(e) = generate(&client, pack, &root, &cache) {
            eprintln!("[{}] FAILED: {}", pack.id, e);
            failures += 1;
        }
    }

    std::process::exit(if failures == 0 { 0 } else { 1 });
}

// =============================================================================
// Pack generation
// =============================================================================

fn generate(
    client: &reqwest::blocking::Client,
    pack: &Pack,
    root: &Path,
    cache: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = root.join("crates").join(pack.module).join("src");
    if !out_dir.is_dir() {
        return Err(format!(
            "Directory '{}' does not exist. Run from workspace root or use --root.",
            out_dir.display()
        )
        .into());
    }

    println!("[{}] resolving {} ...", pack.id, pack.npm);
    let (version, tarball) = fetch_package(client, pack.npm, cache)?;
    println!("[{}] version {}, reading archive ...", pack.id, version);

    let files = read_text_entries(&tarball, |name| {
        name == SIMPLE_ICONS_DATA
            || (name.ends_with(".svg")
                && pack.styles.iter().any(|s| name.starts_with(s.tar_prefix))
                && (pack.id != "fluent"
                    || name.contains("_24_")
                    || name.contains("_20_")))
    })?;

    let mut skipped: Vec<String> = Vec::new();

    for style in pack.styles {
        let mut items: Vec<Item> = Vec::new();

        for (stem, text) in select_sources(&files, style) {
            match try_convert(&text) {
                Ok((path, title)) => {
                    let display = if pack.id == "simpleicons"
                        && title.as_ref().map_or(false, |t| !t.trim().is_empty())
                    {
                        title.unwrap()
                    } else {
                        stem.clone()
                    };
                    items.push(Item { key: stem, display, path });
                }
                Err(problem) => {
                    skipped.push(format!("{}/{}: {}", style.class_name, stem, problem));
                }
            }
        }

        if items.is_empty() {
            return Err(format!(
                "No icons were discovered under '{}' in {} {}. \
                 The NPM package layout may have changed upstream.",
                style.tar_prefix, pack.npm, version
            )
            .into());
        }

        items.sort_by(|a, b| a.display.cmp(&b.display));
        let identifiers = make_unique(items.iter().map(|i| i.display.clone()), style.class_name);
        let members: Vec<(String, String, String)> = items
            .iter()
            .zip(identifiers.iter())
            .map(|(item, id)| (id.clone(), item.display.clone(), item.path.clone()))
            .collect();

        write_file(
            &out_dir,
            style.class_name,
            &emit(pack, &version, style.class_name, &members),
        )?;
        println!("[{}] {}: {} icons", pack.id, style.class_name, members.len());

        if pack.id == "simpleicons" {
            write_brand_colors(pack, &version, &out_dir, &files, &items, &identifiers)?;
        }
    }

    let notice = format!(
        "# Third-party notice\n\nGenerated from npm package `{}` version `{}`.\n\n\
         License: **{}**\n\n{}\n",
        pack.npm, version, pack.license, pack.notice
    );
    fs::write(out_dir.join("THIRD-PARTY-NOTICE.md"), notice)?;

    if !skipped.is_empty() {
        println!("[{}] skipped {} icon(s):", pack.id, skipped.len());
        for line in &skipped {
            println!("    {}", line);
        }
    }

    Ok(())
}

struct Item {
    key: String,
    display: String,
    path: String,
}

fn select_sources<'a>(
    files: &'a HashMap<String, String>,
    style: &Style,
) -> Vec<(String, String)> {
    let in_folder: Vec<(&String, &String)> = files
        .iter()
        .filter(|(name, _)| {
            name.starts_with(style.tar_prefix)
                && name.ends_with(".svg")
                && !name[style.tar_prefix.len()..].contains('/')
        })
        .collect();

    if style.suffix.is_none() {
        return in_folder
            .into_iter()
            .map(|(name, text)| {
                let stem = Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                (stem, text.clone())
            })
            .collect();
    }

    let tail = format!("_{}", style.suffix.unwrap());
    let mut best: HashMap<String, (u32, String)> = HashMap::new();

    for (name, text) in &in_folder {
        let stem = Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if !stem.ends_with(&tail) {
            continue;
        }

        let without_style = &stem[..stem.len() - tail.len()];
        let split = match without_style.rfind('_') {
            Some(i) if i > 0 => i,
            _ => continue,
        };

        let base_name = &without_style[..split];
        let size_str = &without_style[split + 1..];
        let size: u32 = match size_str.parse() {
            Ok(n) if n == 20 || n == 24 => n,
            _ => continue,
        };

        let entry = best.get(base_name);
        let should_replace = match entry {
            Some((cur, _)) => size == 24 && *cur != 24,
            None => true,
        };
        if should_replace {
            best.insert(base_name.to_string(), (size, (*text).clone()));
        }
    }

    best.into_iter()
        .map(|(name, (_, text))| (name, text))
        .collect()
}

fn write_brand_colors(
    pack: &Pack,
    version: &str,
    out_dir: &Path,
    files: &HashMap<String, String>,
    ordered: &[Item],
    identifiers: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let json = match files.get(SIMPLE_ICONS_DATA) {
        Some(j) => j,
        None => {
            println!(
                "[{}] no {} in the package; BrandColors not generated.",
                pack.id, SIMPLE_ICONS_DATA
            );
            return Ok(());
        }
    };

    #[derive(Deserialize)]
    struct IconEntry {
        title: Option<String>,
        hex: Option<String>,
        slug: Option<String>,
    }
    #[derive(Deserialize)]
    struct Root {
        icons: Vec<IconEntry>,
    }

    let parsed: Result<Root, _> = serde_json::from_str(json);
    let entries: Vec<IconEntry> = match parsed {
        Ok(r) => r.icons,
        Err(_) => serde_json::from_str::<Vec<IconEntry>>(json)?,
    };

    let mut color_by_slug: HashMap<String, String> = HashMap::new();
    for entry in &entries {
        let key = entry
            .slug
            .clone()
            .or_else(|| entry.title.as_ref().map(|t| title_to_slug(t)));
        if let (Some(hex), Some(key)) = (&entry.hex, key) {
            color_by_slug.insert(key, hex.clone());
        }
    }

    let mut members: Vec<(String, String, String)> = Vec::new();
    for (index, item) in ordered.iter().enumerate() {
        if let Some(hex) = color_by_slug.get(&item.key) {
            members.push((
                identifiers[index].clone(),
                item.display.clone(),
                format!("#{}", hex.to_uppercase()),
            ));
        }
    }

    write_file(
        out_dir,
        "BrandColors",
        &emit(pack, version, "BrandColors", &members),
    )?;
    println!(
        "[{}] BrandColors: {} colors ({} icons without a color)",
        pack.id,
        members.len(),
        ordered.len() - members.len()
    );
    Ok(())
}

// =============================================================================
// NPM fetch + tarball reading
// =============================================================================

#[derive(Deserialize)]
struct NpmMetadata {
    version: String,
    dist: NpmDist,
}

#[derive(Deserialize)]
struct NpmDist {
    tarball: String,
}

fn fetch_package(
    client: &reqwest::blocking::Client,
    package: &str,
    cache: &Path,
) -> Result<(String, PathBuf), Box<dyn std::error::Error>> {
    let url = format!(
        "https://registry.npmjs.org/{}/latest",
        package.replace("/", "%2F")
    );

    let metadata: NpmMetadata = client.get(&url).send()?.json()?;
    let safe_name = package.replace('/', "_").replace('@', "");
    let file = cache.join(format!("{}-{}.tgz", safe_name, metadata.version));

    if !file.exists() {
        let tmp = file.with_extension("part");
        let mut resp = client.get(&metadata.dist.tarball).send()?;
        let mut out = fs::File::create(&tmp)?;
        std::io::copy(&mut resp, &mut out)?;
        fs::rename(&tmp, &file)?;
    }

    Ok((metadata.version, file))
}

fn read_text_entries(
    tgz: &Path,
    want: impl Fn(&str) -> bool,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file = fs::File::open(tgz)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    let mut result = HashMap::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        if !want(&path) {
            continue;
        }
        let mut text = String::new();
        entry.read_to_string(&mut text)?;
        result.insert(path, text);
    }
    Ok(result)
}

// =============================================================================
// Code emission
// =============================================================================

fn emit(
    pack: &Pack,
    version: &str,
    class_name: &str,
    members: &[(String, String, String)],
) -> String {
    let mut out = String::with_capacity(members.len() * 128);

    // `_10K` etc. are still technically "non-upper-case" because they start
    // with an underscore. Keep the allow so generated icons don't warn.
    out.push_str("#![allow(non_upper_case_globals, non_snake_case)]\n\n");

    out.push_str("// <auto-generated>\n");
    out.push_str("// Generated by Mud.Egui.Icons.Generator. Do not edit.\n");
    out.push_str(&format!(
        "// Source: npm {}@{} ({})\n",
        pack.npm, version, pack.license
    ));
    out.push_str("// </auto-generated>\n\n");

    // No `pub mod ... { }` wrapper: `lib.rs` already declares `pub mod filled;`
    // so flat `pub const` items in this file become `filled::HOME` etc.
    for (identifier, doc, value) in members {
        out.push_str("/// ");
        out.push_str(doc);
        out.push('\n');
        out.push_str("pub const ");
        out.push_str(identifier);
        out.push_str(": &str = \"");
        out.push_str(value);
        out.push_str("\";\n");
    }

    out
}

fn write_file(out_dir: &Path, name: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = out_dir.join(format!("{}.rs", to_snake_case(name)));
    fs::write(&path, code)?;

    let lib_path = out_dir.join("lib.rs");
    if lib_path.exists() {
        let mut content = fs::read_to_string(&lib_path).unwrap_or_default();
        let module_decl = format!("pub mod {};", to_snake_case(name));
        if !content.lines().any(|l| l.trim() == module_decl) {
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&module_decl);
            content.push('\n');
            fs::write(&lib_path, content)?;
        }
    }

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_lowercase().next().unwrap());
        } else {
            out.push(c);
        }
    }
    out
}

fn to_identifier(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 1);
    let mut last_was_sep = false;

    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_uppercase());
            last_was_sep = false;
        } else if !out.is_empty() && !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    // Trim trailing separator, if any.
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        return "_".into();
    }
    if out.chars().next().unwrap().is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

fn make_unique(raws: impl Iterator<Item = String>, reserved: &str) -> Vec<String> {
    let mut used: HashSet<String> = HashSet::new();
    used.insert(reserved.to_string());
    let mut result = Vec::new();

    for raw in raws {
        let candidate = to_identifier(&raw);
        let mut unique = candidate.clone();
        let mut counter = 2;
        while !used.insert(unique.clone()) {
            unique = format!("{}_{}", candidate, counter);
            counter += 1;
        }
        result.push(unique);
    }
    result
}

fn title_to_slug(title: &str) -> String {
    let replaced = title
        .to_lowercase()
        .replace('+', "plus")
        .replace('.', "dot")
        .replace('&', "and")
        .replace('đ', "d")
        .replace('ħ', "h")
        .replace('ı', "i")
        .replace('ĸ', "k")
        .replace('ŀ', "l")
        .replace('ł', "l")
        .replace('ß', "ss")
        .replace('ŧ', "t");

    replaced
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

// =============================================================================
// SVG conversion
// =============================================================================

fn try_convert(svg: &str) -> Result<(String, Option<String>), String> {
    let doc = Document::parse(svg).map_err(|e| format!("Invalid XML: {}", e))?;
    let root = doc.root_element();
    let mut parts = String::new();
    let mut title: Option<String> = None;

    for node in root.descendants().filter(|n| n.is_element()) {
        let name = node.tag_name().name();

        if name == "title" {
            title.get_or_insert_with(|| node.text().unwrap_or("").trim().to_string());
            continue;
        }

        if matches!(name, "line" | "polyline" | "use" | "image" | "text" | "mask" | "clipPath") {
            return Err(format!("Unsupported element <{}>", name));
        }

        if !matches!(name, "path" | "circle" | "ellipse" | "rect" | "polygon") {
            continue;
        }

        if is_invisible(&node) {
            continue;
        }

        if node.ancestors().any(|a| a.is_element() && a.attribute("transform").is_some()) {
            return Err("Transforms are not supported".into());
        }

        let data = match name {
            "path" => node.attribute("d").map(|s| s.to_string()),
            "circle" => ellipse_data(
                number(&node, "cx"),
                number(&node, "cy"),
                number(&node, "r"),
                number(&node, "r"),
            ),
            "ellipse" => ellipse_data(
                number(&node, "cx"),
                number(&node, "cy"),
                number(&node, "rx"),
                number(&node, "ry"),
            ),
            "rect" => rect_data(&node),
            "polygon" => polygon_data(&node),
            _ => None,
        };

        let data = match data {
            Some(d) if !d.trim().is_empty() => d,
            _ => return Err(format!("Malformed <{}>", name)),
        };

        if !parts.is_empty() {
            parts.push(' ');
        }
        parts.push_str(data.trim());
    }

    if parts.is_empty() {
        return Err("No visible shapes".into());
    }

    Ok((parts, title))
}

fn is_invisible(node: &Node) -> bool {
    if let Some(fill) = node.attribute("fill") {
        if fill.eq_ignore_ascii_case("none") {
            return true;
        }
    }
    if let Some(display) = node.attribute("display") {
        if display.eq_ignore_ascii_case("none") {
            return true;
        }
    }
    for name in ["opacity", "fill-opacity"] {
        if let Some(v) = node.attribute(name) {
            if let Ok(n) = v.parse::<f64>() {
                if n < 1.0 {
                    return true;
                }
            }
        }
    }
    false
}

fn ellipse_data(cx: f64, cy: f64, rx: f64, ry: f64) -> Option<String> {
    if rx <= 0.0 || ry <= 0.0 {
        return None;
    }
    Some(format!(
        "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0z",
        fmt(cx - rx),
        fmt(cy),
        fmt(rx),
        fmt(ry),
        fmt(2.0 * rx),
        fmt(rx),
        fmt(ry),
        fmt(-2.0 * rx)
    ))
}

fn rect_data(node: &Node) -> Option<String> {
    if node.attribute("rx").is_some() || node.attribute("ry").is_some() {
        return None;
    }
    let w = number(node, "width");
    let h = number(node, "height");
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let x = number(node, "x");
    let y = number(node, "y");
    Some(format!(
        "M{} {}h{}v{}h{}z",
        fmt(x),
        fmt(y),
        fmt(w),
        fmt(h),
        fmt(-w)
    ))
}

fn polygon_data(node: &Node) -> Option<String> {
    let raw = node.attribute("points")?;
    let tokens: Vec<&str> = raw
        .split(|c: char| c == ' ' || c == ',' || c == '\t' || c == '\r' || c == '\n')
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.len() < 6 || tokens.len() % 2 != 0 {
        return None;
    }
    let mut out = String::new();
    for i in (0..tokens.len()).step_by(2) {
        let x: f64 = tokens[i].parse().ok()?;
        let y: f64 = tokens[i + 1].parse().ok()?;
        out.push(if i == 0 { 'M' } else { 'L' });
        out.push_str(&fmt(x));
        out.push(' ');
        out.push_str(&fmt(y));
    }
    out.push('z');
    Some(out)
}

fn number(node: &Node, name: &str) -> f64 {
    node.attribute(name)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn fmt(value: f64) -> String {
    let rounded = (value * 10000.0).round() / 10000.0;
    let mut s = format!("{}", rounded);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}
