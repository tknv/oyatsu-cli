//! oyatsu コンフィグの読み込み。
//!
//! フォーマット: 1行目に緯度、2行目に経度を書いたテキストファイル。
//! 例:
//! ```text
//! 35.681444600642514
//! 139.76579265965165
//! ```

use std::path::PathBuf;

/// OSごとの設定ファイルのパスを返す。
///
/// Linux:
///   $XDG_CONFIG_HOME/oyatsu
///   または ~/.config/oyatsu
///
/// macOS:
///   ~/Library/Application Support/oyatsu
///
/// Windows:
///   %APPDATA%\oyatsu
/// `$HOME` が取得できない場合は `None`。
pub fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var_os("APPDATA")?);
        path.push("oyatsu");
        return Some(path);
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = PathBuf::from(std::env::var_os("HOME")?);
        path.push("Library");
        path.push("Application Support");
        path.push("oyatsu");
        return Some(path);
    }

    // Linux / FreeBSD / OpenBSD / NetBSD など
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // XDG_CONFIG_HOME を優先
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            let mut path = PathBuf::from(xdg);
            path.push("oyatsu");
            return Some(path);
        }

        // 未設定なら ~/.config
        let mut path = PathBuf::from(std::env::var_os("HOME")?);
        path.push(".config");
        path.push("oyatsu");
        return Some(path);
    }
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
