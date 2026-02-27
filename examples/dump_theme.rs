/// 生成 syntax.bin 主题文件的工具
///
/// 用法:
///   cargo run --example dump_theme -- <theme_name_or_tmtheme_path> <output_path>
///
/// 示例（内置主题）:
///   cargo run --example dump_theme -- "base16-ocean.dark" /tmp/dark/syntax.bin
///   cargo run --example dump_theme -- "base16-ocean.light" /tmp/light/syntax.bin
///
/// 示例（.tmTheme 文件）:
///   cargo run --example dump_theme -- /path/to/MyTheme.tmTheme /tmp/my/syntax.bin
///
/// 可用内置主题名:
///   base16-ocean.dark, base16-ocean.light, base16-eighties.dark,
///   base16-mocha.dark, InspiredGitHub, Solarized (dark), Solarized (light)

use std::path::Path;
use syntect::highlighting::ThemeSet;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: dump_theme <theme_name_or_tmtheme_path> <output_path>");
        std::process::exit(1);
    }
    let theme_input = &args[1];
    let output_path = Path::new(&args[2]);

    let input_path = Path::new(theme_input);
    let theme = if input_path.extension().map_or(false, |e| e == "tmTheme") || input_path.exists() {
        // 从 .tmTheme 文件加载
        ThemeSet::get_theme(input_path).unwrap_or_else(|e| {
            eprintln!("Failed to load .tmTheme file '{}': {}", theme_input, e);
            std::process::exit(1);
        })
    } else {
        // 从内置主题加载
        let theme_set = ThemeSet::load_defaults();
        theme_set.themes.get(theme_input).cloned().unwrap_or_else(|| {
            eprintln!("Theme '{}' not found. Available themes:", theme_input);
            for name in theme_set.themes.keys() {
                eprintln!("  {}", name);
            }
            std::process::exit(1);
        })
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    syntect::dumps::dump_to_file(&theme, output_path).expect("Failed to dump theme");
    println!("Wrote theme to {}", output_path.display());
}
