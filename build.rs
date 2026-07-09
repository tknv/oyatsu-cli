// SPDX-FileCopyrightText: 2026 Watanabe Takanobu
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    generate_man_page();
    generate_holidays_data();
}

// ─── MAN ページ生成 ─────────────────────────────────────────

fn generate_man_page() {
    // 1. build.rs 自体や Cargo.toml が変更されたときだけ再実行するよう Cargo に通知
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // 2. Cargo.toml から情報を取得
    let version = env!("CARGO_PKG_VERSION");
    let license = env!("CARGO_PKG_LICENSE");
    let authors = env!("CARGO_PKG_AUTHORS");

    // 3. MAN ページのメッセージ（Roff フォーマット）を構築
    let man_content = format!(
        r#".TH OYATSU 1 "{version}"
.SH NAME
oyatsu \- Display and Japanese clock time (seasonal time) calculated by location CLI
.SH SYNOPSIS
.B oyatsu
[\fB\-\-lat\fR \fIlatitude\fR] [\fB\-\-lon\fR \fIlongitude\fR] [\fB\-\-lang\fR \fIja|en\fR]
.br
.B oyatsu
[\fB\-V\fR | \fB\-\-version\fR] [\fB\-h\fR | \fB\-\-help\fR]
.SH DESCRIPTION
A command-line tool for tracking snack time.
.IP \(bu 2
和暦 \- Japanese calendar
.IP \(bu 2
和風月名 \- Japanese-style month names
.IP \(bu 2
日付 \- day
.IP \(bu 2
和時計時刻(不定時法) \- Japanese clock time (seasonal time)
.IP \(bu 2
二十四節気 \- 24 solar terms
.IP \(bu 2
雑節 \- Other season marks
.IP \(bu 2
干支 \- Chinese zodiac signs
.IP \(bu 2
おやつ時通知 \- Notice Oyatsu-doki
.PP
During the \(oqHitsuji\(cq hour (the eighth hour, roughly 13:00\(en15:00 in modern terms), the word \(oqOyatsu(snack)\(cq is displayed at the beginning of the output.
.PP
おやつの時間を把握する為のコマンドラインツール。
「未」の刻(八つ時、現代のおよそ13:00〜15:00)には先頭に「おやつ」と表示されます。
.SH OPTIONS
.TP
.BR \-\-lat " " \fIlatitude\fR
Latitude (decimal degrees, N positive)
.TP
.BR \-\-lon " " \fIlongitude\fR
Longitude (decimal degrees, E positive)
.TP
.BR \-\-lang " " \fIlang\fR
Output language: ja (default) | en
.TP
.BR \-h ", " \-\-help
Display the help and exit.
.TP
.BR \-V ", " \-\-version
Display the version information and exit.
.SH LOCATION PRIORITY
1. \-\-lat / \-\-lon options
.br
2. ~/.config/oyatsu (line1: latitude, line2: longitude),
.br
  PATH
.br
    Mac ~/Library/Application Support/oyatsu, Win %APPDATA%\\oyatsu
.br
3. Tokyo Station default
.SH LANGUAGE PRIORITY
1. \-\-lang option
.br
2. OYATSU_LANG environment variable
.br
3. ja (Japanese, default)
.SH AUTHOR
{authors}
.SH LICENSE
{license}"#
    );

    // 4. 生成先を決定する
    // 本来 Rust の作法では `OUT_DIR`（target/ 内の一時フォルダ）への出力が推奨されますが、
    // パッケージメンテナーや Makefile での扱いやすさを考慮し、
    // プロジェクトのルートディレクトリ（Cargo.toml がある場所）に直接書き出します。
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut man_path = PathBuf::from(&manifest_dir);
    man_path.push("oyatsu.1");

    // 5. ファイル書き込み
    let mut file = File::create(&man_path)
        .unwrap_or_else(|e| panic!("Failed to create man page file at {:?}: {}", man_path, e));
    file.write_all(man_content.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to write man page content: {}", e));
}

// ─── 祝日データ生成 ─────────────────────────────────────────
//
// data/cal-YYYY.csv (フォーマット: name,month,day) を読み込み、
// パース・検証した上で静的配列としてコード生成し、
// $OUT_DIR/holidays_data.rs に書き出す。
// src/holidays.rs から include!(concat!(env!("OUT_DIR"), "/holidays_data.rs"))
// で取り込まれる。
//
// 新しい年のデータを追加する場合は data/cal-YYYY.csv を置くだけでよい
// (このディレクトリを自動走査するため build.rs 自体の変更は不要)。

fn generate_holidays_data() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = PathBuf::from(&manifest_dir).join("data");

    // data/ ディレクトリ自体の増減 (新しい年のファイル追加など) でも
    // 再実行されるようにする。
    println!("cargo:rerun-if-changed={}", data_dir.display());

    let mut csv_paths: Vec<PathBuf> = fs::read_dir(&data_dir)
        .unwrap_or_else(|e| panic!("Failed to read data dir {:?}: {}", data_dir, e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("csv"))
        .collect();
    csv_paths.sort();

    let mut entries = String::new();

    for path in &csv_paths {
        // ファイルごとに変更検知させる (data/ ディレクトリの監視だけでは
        // ファイル内容の変更が検知されない環境があるための保険)。
        println!("cargo:rerun-if-changed={}", path.display());

        let year = parse_year_from_filename(path);
        let content =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

        let mut seen: HashSet<(u32, u32)> = HashSet::new();

        for (lineno, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(3, ',');
            let name = parts.next().unwrap_or("").trim();
            let month_str = parts.next().unwrap_or_else(|| {
                panic!(
                    "{}:{}: missing month field: {:?}",
                    path.display(),
                    lineno + 1,
                    raw_line
                )
            });
            let day_str = parts.next().unwrap_or_else(|| {
                panic!(
                    "{}:{}: missing day field: {:?}",
                    path.display(),
                    lineno + 1,
                    raw_line
                )
            });

            let month: u32 = month_str.trim().parse().unwrap_or_else(|_| {
                panic!(
                    "{}:{}: invalid month {:?} in line {:?}",
                    path.display(),
                    lineno + 1,
                    month_str,
                    raw_line
                )
            });
            let day: u32 = day_str.trim().parse().unwrap_or_else(|_| {
                panic!(
                    "{}:{}: invalid day {:?} in line {:?}",
                    path.display(),
                    lineno + 1,
                    day_str,
                    raw_line
                )
            });

            if !(1..=12).contains(&month) {
                panic!(
                    "{}:{}: month out of range (1-12): {}",
                    path.display(),
                    lineno + 1,
                    month
                );
            }
            if !(1..=31).contains(&day) {
                panic!(
                    "{}:{}: day out of range (1-31): {}",
                    path.display(),
                    lineno + 1,
                    day
                );
            }
            if !seen.insert((month, day)) {
                panic!(
                    "{}:{}: duplicate date {}/{} in {} data",
                    path.display(),
                    lineno + 1,
                    month,
                    day,
                    year
                );
            }

            // name は Debug フォーマットでエスケープ済み文字列リテラルとして出力する。
            entries.push_str(&format!("    ({year}, {name:?}, {month}, {day}),\n"));
        }
    }

    let code = format!(
        "/// (年, 名称 (空文字列は名称のない休日), 月, 日) の一覧。\n\
         /// build.rs が data/cal-YYYY.csv から自動生成する。\n\
         pub static HOLIDAYS: &[(i32, &str, u32, u32)] = &[\n{entries}];\n"
    );

    write_generated_file(&code, "holidays_data.rs");
}

fn parse_year_from_filename(path: &Path) -> i32 {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| panic!("unreadable file name: {:?}", path));
    stem.strip_prefix("cal-")
        .unwrap_or_else(|| {
            panic!(
                "unexpected data file name: {:?} (expected cal-YYYY.csv)",
                path
            )
        })
        .parse()
        .unwrap_or_else(|_| panic!("cannot parse year from file name: {:?}", path))
}

fn write_generated_file(content: &str, filename: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join(filename);
    fs::write(&dest_path, content)
        .unwrap_or_else(|e| panic!("Failed to write {:?}: {}", dest_path, e));
}
