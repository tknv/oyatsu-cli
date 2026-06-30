// SPDX-FileCopyrightText: 2026 Watanabe Takanobu
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
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
和時計時刻(不定時法) \- Japanese clock time (seasonal time)
.IP \(bu 2
二十四節気 \- 24 solar terms
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
2. ~/.config/oyatsu (line1: latitude, line2: longitude)
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
    let mut man_path = PathBuf::from(manifest_dir);
    man_path.push("oyatsu.1");

    // 5. ファイル書き込み
    let mut file = File::create(&man_path)
        .unwrap_or_else(|e| panic!("Failed to create man page file at {:?}: {}", man_path, e));
    file.write_all(man_content.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to write man page content: {}", e));
}
