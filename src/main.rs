//! oyatsu — 和暦・干支・和風月名・日付・日干支・和時計(不定時法)・二十四節気・雑節を表示する CLI。
//!
//! 元は Android ウィジェットアプリ Oyatsu (https://github.com/tknv/Oyatsu) の
//! ロジックを Rust に移植したもの。

mod config;
mod holidays;
mod i18n;
mod koyomi;
mod moonface;
mod sunrise;

use chrono::Datelike;
use i18n::Lang;

const DEFAULT_LATITUDE: f64 = 35.681444600642514;
const DEFAULT_LONGITUDE: f64 = 139.76579265965165;

// ─── 引数 ────────────────────────────────────────────────────

struct Args {
    lat: Option<f64>,
    lon: Option<f64>,
    lang: Option<Lang>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut lang: Option<Lang> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-lat" | "--lat" => {
                let v = args.next().ok_or("-lat requires a numeric value")?;
                lat = Some(
                    v.parse::<f64>()
                        .map_err(|_| format!("cannot parse latitude: {v}"))?,
                );
            }
            "-lon" | "--lon" => {
                let v = args.next().ok_or("-lon requires a numeric value")?;
                lon = Some(
                    v.parse::<f64>()
                        .map_err(|_| format!("cannot parse longitude: {v}"))?,
                );
            }
            "--lang" | "-lang" => {
                let v = args.next().ok_or("--lang requires a value (ja or en)")?;
                lang = Some(Lang::from_str(&v)?);
            }
            "-V" | "--version" => {
                println!("oyatsu {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(Args { lat, lon, lang })
}

// ─── 言語決定 ─────────────────────────────────────────────────
//
// 優先順位: --lang > OYATSU_LANG 環境変数 > Ja (デフォルト)

fn resolve_lang(args: &Args) -> Lang {
    if let Some(l) = args.lang {
        return l;
    }
    if let Ok(v) = std::env::var("OYATSU_LANG") {
        if let Ok(l) = Lang::from_str(v.trim()) {
            return l;
        }
    }
    Lang::Ja
}

// ─── 位置情報 ─────────────────────────────────────────────────
//
// 優先順位: --lat/--lon > ~/.config/oyatsu > デフォルト(東京駅)

fn resolve_location(args: &Args) -> (f64, f64) {
    if let (Some(lat), Some(lon)) = (args.lat, args.lon) {
        return (lat, lon);
    }
    if let Some((file_lat, file_lon)) = config::load_location() {
        return (args.lat.unwrap_or(file_lat), args.lon.unwrap_or(file_lon));
    }
    if args.lat.is_some() || args.lon.is_some() {
        return (
            args.lat.unwrap_or(DEFAULT_LATITUDE),
            args.lon.unwrap_or(DEFAULT_LONGITUDE),
        );
    }
    (DEFAULT_LATITUDE, DEFAULT_LONGITUDE)
}

fn is_valid_latitude(v: f64) -> bool {
    (-90.0..=90.0).contains(&v)
}
fn is_valid_longitude(v: f64) -> bool {
    (-180.0..=180.0).contains(&v)
}

// ─── ヘルプ ──────────────────────────────────────────────────

fn print_usage() {
    let prog = std::env::args()
        .next()
        .unwrap_or_else(|| "oyatsu".to_string());
    eprintln!(
        "Usage: {prog} [options]\n\n\
         Options:\n\
           --lat <latitude>   Latitude  (decimal degrees, N positive)\n\
           --lon <longitude>  Longitude (decimal degrees, E positive)\n\
           --lang <lang>      Output language: ja (default) | en\n\
           -V, --version      Show version information\n\
           -h, --help         Show this help\n\n\
         Location priority:\n\
           1. --lat / --lon options\n\
           2. ~/.config/oyatsu  (line1: latitude, line2: longitude) Mac: ~/Library/Application Support/oyatsu, Win: %APPDATA%\\oyatsu\n\
           3. Tokyo Station default\n\n\
         Language priority:\n\
           1. --lang option\n\
           2. OYATSU_LANG environment variable\n\
           3. ja (Japanese, default)\n\n\
         Examples:\n\
           {prog}\n\
           {prog} --lang en\n\
           {prog} --lat 35.0116 --lon 135.7681\n\
           OYATSU_LANG=en {prog}\n"
    );
}

// ─── テスト 中今 ───────────────────────────────────────────────
//
// OYATSU_NOW="2026-04-18 15:23:00" ./target/release/oyatsu

fn now() -> chrono::DateTime<chrono::Local> {
    if let Ok(t) = std::env::var("OYATSU_NOW") {
        // NaiveDateTime形式の文字列を解析し、Localタイムゾーンに割り当てる
        if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(&t, "%Y-%m-%d %H:%M:%S") {
            return ndt.and_local_timezone(chrono::Local).unwrap();
        }
    }
    chrono::Local::now()
}

// ─── メイン ──────────────────────────────────────────────────

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            print_usage();
            std::process::exit(1);
        }
    };

    let lang = resolve_lang(&args);
    let txt = i18n::texts(lang);

    let (latitude, longitude) = resolve_location(&args);

    if !is_valid_latitude(latitude) || !is_valid_longitude(longitude) {
        eprintln!("{}: {latitude}, {longitude}", txt.error_invalid_loc);
        std::process::exit(1);
    }

    let now = now();
    let today = now.date_naive();

    // ── 暦情報 ──
    let ja_year = koyomi::japanese_year(today);
    let ja_month = koyomi::japanese_month_name(today);
    let ja_term = koyomi::solar_term(today);
    let ja_zassetsu = koyomi::zassetsu(today);
    let ja_eto = koyomi::sixty_kanji_cycle(today);
    let ja_day_eto = koyomi::day_kanji_cycle(today);

    // ── 日付・休日 ──
    let ja_day = koyomi::kanji_day(today);
    let holiday = holidays::lookup(today);

    // 月名の次に表示する「(イベント名) 日付」部分
    let date_display_ja = {
        let base = match &holiday {
            Some(holidays::Holiday::Named(name)) => format!("{name} {ja_day}"),
            Some(holidays::Holiday::Unnamed) => {
                format!("{} {ja_day}", i18n::texts(Lang::Ja).holiday)
            }
            None => ja_day.clone(),
        };
        format!("{base}{ja_day_eto}")
    };
    let date_display_en = {
        let en_day = i18n::ordinal_day_en(today.day());
        let base = match &holiday {
            Some(holidays::Holiday::Named(name)) => format!("{} {en_day}", i18n::holiday_en(name)),
            Some(holidays::Holiday::Unnamed) => {
                format!("{} {en_day}", i18n::texts(Lang::En).holiday)
            }
            None => en_day,
        };
        let day_eto_en = i18n::eto_en(&ja_day_eto);
        format!("{base} {day_eto_en}")
    };

    // ── 日の出・日の入り ──
    let sunrise = sunrise::official_sunrise(today, latitude, longitude);
    let sunset = sunrise::official_sunset(today, latitude, longitude);

    let sunset = match (sunrise, sunset) {
        (Some(sr), Some(ss)) if ss.timestamp_millis() < sr.timestamp_millis() => {
            Some(ss + chrono::Duration::days(1))
        }
        (_, ss) => ss,
    };

    match (sunrise, sunset) {
        (Some(sr), Some(ss)) => {
            let jt = koyomi::calculate_japanese_time(now, sr, ss);

            // ── 次に来る朔望 (朔・上弦・望・下弦) ──
            let moon_msg = match lang {
                Lang::Ja => moonface::describe_ja(now, today),
                Lang::En => moonface::describe_en(now, today),
            };

            // ── 言語別出力 ──
            let output = match lang {
                Lang::Ja => {
                    let prefix = if jt.is_hitsuji_time {
                        format!("{} ", txt.oyatsu)
                    } else {
                        String::new()
                    };
                    let zassetsu_str = ja_zassetsu.map(|z| format!(" [{z}]")).unwrap_or_default();
                    format!(
                        "{prefix}{ja_year}{ja_eto}歳 {ja_month} {date_display_ja} {} {ja_term}{zassetsu_str}",
                        jt.label
                    )
                }
                Lang::En => {
                    let era = i18n::era_en(&ja_year);
                    let month = i18n::month_name_en(ja_month);
                    let jikoku = i18n::jikoku_en(&jt.label);
                    let term = i18n::solar_term_en(ja_term);
                    let zassetsu_str = ja_zassetsu
                        .map(|z| format!(" [{}]", i18n::zassetsu_en(z)))
                        .unwrap_or_default();
                    let eto = i18n::eto_en(&ja_eto);
                    let prefix = if jt.is_hitsuji_time {
                        format!("{} ", txt.oyatsu)
                    } else {
                        String::new()
                    };
                    format!(
                        "{prefix}{era} ({eto}) {month} {date_display_en} {jikoku} {term}{zassetsu_str}"
                    )
                }
            };
            println!("{output}");

            // 補足情報 (stderr)
            let sun_info = match lang {
                Lang::Ja => format!(
                    "({}{}-{}{} {})",
                    txt.sunrise_label,
                    sr.format("%H:%M"),
                    txt.sunset_label,
                    ss.format("%H:%M"),
                    moon_msg,
                ),
                Lang::En => format!(
                    "({} {}, {} {}, {})",
                    txt.sunrise_label,
                    sr.format("%H:%M"),
                    txt.sunset_label,
                    ss.format("%H:%M"),
                    moon_msg,
                ),
            };
            eprintln!("{sun_info}");
        }
        _ => {
            eprintln!("{}", txt.error_no_sun);
            // 日の出なし時は時刻なしで出力
            let output = match lang {
                Lang::Ja => {
                    format!("{ja_year}{ja_eto}歳 {ja_month} {date_display_ja} {ja_term}")
                }
                Lang::En => format!(
                    "{} ({})  {}  {}  {}",
                    i18n::era_en(&ja_year),
                    i18n::eto_en(&ja_eto),
                    i18n::month_name_en(ja_month),
                    date_display_en,
                    i18n::solar_term_en(ja_term),
                ),
            };
            println!("{output}");
            std::process::exit(1);
        }
    }
}
