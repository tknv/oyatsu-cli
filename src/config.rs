//! `~/.config/oyatsu` の読み込み。
//!
//! フォーマット: 1行目に緯度、2行目に経度を書いたテキストファイル。
//! 例:
//! ```text
//! 35.681444600642514
//! 139.76579265965165
//! ```

use std::path::PathBuf;

/// `~/.config/oyatsu` のパスを返す。
/// `$HOME` が取得できない場合は `None`。
pub fn config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("oyatsu");
    Some(path)
}

/// 設定ファイルを読み込み、(緯度, 経度) を返す。
/// ファイルが存在しない、もしくはパースできない場合は `None`。
pub fn load_location() -> Option<(f64, f64)> {
    let path = config_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    let mut lines = content.lines().map(str::trim).filter(|l| !l.is_empty());

    let lat = lines.next()?.parse::<f64>().ok()?;
    let lon = lines.next()?.parse::<f64>().ok()?;
    Some((lat, lon))
}
