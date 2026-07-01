//! 多言語対応 (i18n)。
//!
//! 優先順位: --lang オプション > OYATSU_LANG 環境変数 > 日本語(デフォルト)

// ─── 言語 ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ja,
    En,
}

impl Lang {
    /// 文字列から言語を解析する。不明な文字列は Err。
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "ja" | "japanese" => Ok(Lang::Ja),
            "en" | "english" => Ok(Lang::En),
            other => Err(format!(
                "unsupported language: '{other}'  (supported: ja, en)"
            )),
        }
    }
}

// ─── 和暦 → 英語 ─────────────────────────────────────────────

/// 和暦を英語表記にする (例: "令和八年" → "Reiwa 8")。
/// koyomi::japanese_year() の文字列を受け取り変換する。
pub fn era_en(ja_year: &str) -> String {
    const ERA_MAP: &[(&str, &str)] = &[
        ("令和", "Reiwa"),
        ("平成", "Heisei"),
        ("昭和", "Showa"),
        ("大正", "Taisho"),
        ("明治", "Meiji"),
    ];

    for &(ja, en) in ERA_MAP {
        if let Some(rest) = ja_year.strip_prefix(ja) {
            // rest は "八年" や "三十一年" など。kanji → arabic に変換。
            let num_part = rest.trim_end_matches('年');
            let n = kanji_num_to_u32(num_part);
            return format!("{en} {n}");
        }
    }

    // 元号が見つからなければ西暦表記として数字部分をそのまま返す
    let digits: String = ja_year.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        return digits;
    }
    ja_year.to_string()
}

/// 漢数字 (元年〜九千九百九十九年) → u32
fn kanji_num_to_u32(s: &str) -> u32 {
    if s == "元" {
        return 1;
    }

    // 千・百・十・一の位を順に処理
    let mut result = 0u32;
    let mut current = 0u32; // 今読んでいる数字 (十の前の数字など)

    const DIGIT_MAP: &[(&str, u32)] = &[
        ("一", 1),
        ("二", 2),
        ("三", 3),
        ("四", 4),
        ("五", 5),
        ("六", 6),
        ("七", 7),
        ("八", 8),
        ("九", 9),
    ];
    const POWER_MAP: &[(&str, u32)] = &[("千", 1000), ("百", 100), ("十", 10)];

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // 一文字ずつ試す (漢字は1文字=3バイトなので chars() で扱う)
        let ch = chars[i].to_string();

        // 位の漢字か？
        if let Some(&(_, power)) = POWER_MAP.iter().find(|&&(k, _)| k == ch.as_str()) {
            result += if current == 0 { power } else { current * power };
            current = 0;
            i += 1;
            continue;
        }
        // 数字の漢字か？
        if let Some(&(_, d)) = DIGIT_MAP.iter().find(|&&(k, _)| k == ch.as_str()) {
            current = d;
            i += 1;
            continue;
        }
        i += 1;
    }
    result + current // 一の位の余り
}

// ─── 和風月名 → 英語 ─────────────────────────────────────────

pub fn month_name_en(ja: &str) -> &'static str {
    match ja {
        "睦月" => "Mutsuki (Jan)",
        "如月" => "Kisaragi (Feb)",
        "弥生" => "Yayoi (Mar)",
        "卯月" => "Uzuki (Apr)",
        "皐月" => "Satsuki (May)",
        "水無月" => "Minazuki (Jun)",
        "文月" => "Fumizuki (Jul)",
        "葉月" => "Hazuki (Aug)",
        "長月" => "Nagatsuki (Sep)",
        "神無月" => "Kannazuki (Oct)",
        "霜月" => "Shimotsuki (Nov)",
        "師走" => "Shiwasu (Dec)",
        _ => "(unknown month)",
    }
}

// ─── 和時計 → 英語 ───────────────────────────────────────────

/// 和時計ラベルを英語に変換する。
/// "未三つ" → "Hitsuji 3" のような形式。
pub fn jikoku_en(ja: &str) -> String {
    // 最初の1〜2文字が刻名、残りが「一つ」〜「四つ」
    const JIKOKU_MAP: &[(&str, &str)] = &[
        ("卯", "U"),
        ("辰", "Tatsu"),
        ("巳", "Mi"),
        ("午", "Uma"),
        ("未", "Hitsuji"),
        ("申", "Saru"),
        ("酉", "Tori"),
        ("戌", "Inu"),
        ("亥", "I"),
        ("子", "Ne"),
        ("丑", "Ushi"),
        ("寅", "Tora"),
    ];
    const KOSU_MAP: &[(&str, &str)] = &[
        ("一つ", "Hitotsu"),
        ("二つ", "Futatsu"),
        ("三つ", "Mitsu"),
        ("四つ", "Yotsu"),
    ];

    for &(ja_k, en_k) in JIKOKU_MAP {
        if let Some(rest) = ja.strip_prefix(ja_k) {
            for &(ja_n, en_n) in KOSU_MAP {
                if rest == ja_n {
                    return format!("{en_k} {en_n}");
                }
            }
            return en_k.to_string();
        }
    }
    ja.to_string()
}

// ─── 二十四節気 → 英語 ───────────────────────────────────────

pub fn solar_term_en(ja: &str) -> &'static str {
    match ja {
        "小寒" => "Minor Cold",
        "大寒" => "Major Cold",
        "立春" => "Start of Spring",
        "雨水" => "Rain Water",
        "啓蟄" => "Awakening of Insects",
        "春分" => "Vernal Equinox",
        "清明" => "Clear and Bright",
        "穀雨" => "Grain Rain",
        "立夏" => "Start of Summer",
        "小満" => "Grain Buds",
        "芒種" => "Grain in Ear",
        "夏至" => "Summer Solstice",
        "小暑" => "Minor Heat",
        "大暑" => "Major Heat",
        "立秋" => "Start of Autumn",
        "処暑" => "End of Heat",
        "白露" => "White Dew",
        "秋分" => "Autumnal Equinox",
        "寒露" => "Cold Dew",
        "霜降" => "Frost's Descent",
        "立冬" => "Start of Winter",
        "小雪" => "Minor Snow",
        "大雪" => "Major Snow",
        "冬至" => "Winter Solstice",
        _ => "(unknown term)",
    }
}

// ─── 干支 → 英語 ─────────────────────────────────────────────

pub fn eto_en(ja: &str) -> String {
    const STEM_MAP: &[(&str, &str)] = &[
        ("甲", "Kinoe"),
        ("乙", "Kinoto"),
        ("丙", "Hinoe"),
        ("丁", "Hinoto"),
        ("戊", "Tsuchinoe"),
        ("己", "Tsuchinoto"),
        ("庚", "Kanoe"),
        ("辛", "Kanoto"),
        ("壬", "Mizunoe"),
        ("癸", "Mizunoto"),
    ];
    const BRANCH_MAP: &[(&str, &str)] = &[
        ("子", "Ne"),
        ("丑", "Ushi"),
        ("寅", "Tora"),
        ("卯", "U"),
        ("辰", "Tatsu"),
        ("巳", "Mi"),
        ("午", "Uma"),
        ("未", "Hitsuji"),
        ("申", "Saru"),
        ("酉", "Tori"),
        ("戌", "Inu"),
        ("亥", "I"),
    ];

    let mut result = ja.to_string();
    let chars: Vec<char> = ja.chars().collect();

    if chars.len() == 2 {
        let stem = chars[0].to_string();
        let branch = chars[1].to_string();
        let en_s = STEM_MAP
            .iter()
            .find(|&&(k, _)| k == stem.as_str())
            .map(|&(_, v)| v)
            .unwrap_or(&stem);
        let en_b = BRANCH_MAP
            .iter()
            .find(|&&(k, _)| k == branch.as_str())
            .map(|&(_, v)| v)
            .unwrap_or(&branch);
        result = format!("{en_s}-{en_b}");
    }
    result
}

// ─── 固定テキスト ─────────────────────────────────────────────

#[allow(dead_code)]
pub struct Texts {
    pub oyatsu: &'static str,
    pub sun_info_fmt: &'static str, // "sunrise HH:MM - sunset HH:MM" 等
    pub sunrise_label: &'static str,
    pub sunset_label: &'static str,
    pub error_no_sun: &'static str,
    pub error_invalid_loc: &'static str,
    pub error_unknown_arg: &'static str,
    pub error_missing_lat: &'static str,
    pub error_missing_lon: &'static str,
    pub error_bad_lat: &'static str,
    pub error_bad_lon: &'static str,
    pub error_prefix: &'static str,
}

pub const JA: Texts = Texts {
    oyatsu: "おやつ",
    sun_info_fmt: "", // 使わない (koyomi の format! で直接組む)
    sunrise_label: "日出",
    sunset_label: "日入",
    error_no_sun: "エラー: 指定された緯度経度では日の出・日の入りを計算できませんでした(極夜・白夜の可能性があります)。",
    error_invalid_loc: "エラー: 無効な緯度または経度です",
    error_unknown_arg: "不明な引数です",
    error_missing_lat: "-lat には数値を指定してください",
    error_missing_lon: "-lon には数値を指定してください",
    error_bad_lat: "緯度として解釈できません",
    error_bad_lon: "経度として解釈できません",
    error_prefix: "エラー",
};

pub const EN: Texts = Texts {
    oyatsu: "Oyatsu (snack time!)",
    sun_info_fmt: "",
    sunrise_label: "Sunrise",
    sunset_label: "Sunset",
    error_no_sun: "Error: could not compute sunrise/sunset (polar night or midnight sun?).",
    error_invalid_loc: "Error: invalid latitude or longitude",
    error_unknown_arg: "unknown argument",
    error_missing_lat: "--lat requires a numeric value",
    error_missing_lon: "--lon requires a numeric value",
    error_bad_lat: "cannot parse latitude",
    error_bad_lon: "cannot parse longitude",
    error_prefix: "Error",
};

pub fn texts(lang: Lang) -> &'static Texts {
    match lang {
        Lang::Ja => &JA,
        Lang::En => &EN,
    }
}

// ─── テスト ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanji_num() {
        assert_eq!(kanji_num_to_u32("元"), 1);
        assert_eq!(kanji_num_to_u32("八"), 8);
        assert_eq!(kanji_num_to_u32("十"), 10);
        assert_eq!(kanji_num_to_u32("十五"), 15);
        assert_eq!(kanji_num_to_u32("三十"), 30);
        assert_eq!(kanji_num_to_u32("三十一"), 31);
        assert_eq!(kanji_num_to_u32("六十四"), 64);
        assert_eq!(kanji_num_to_u32("百"), 100);
        assert_eq!(kanji_num_to_u32("二百"), 200);
    }

    #[test]
    fn test_era_en() {
        assert_eq!(era_en("令和八年"), "Reiwa 8");
        assert_eq!(era_en("令和元年"), "Reiwa 1");
        assert_eq!(era_en("平成三十一年"), "Heisei 31");
        assert_eq!(era_en("昭和六十四年"), "Showa 64");
        assert_eq!(era_en("大正元年"), "Taisho 1");
        assert_eq!(era_en("明治四十五年"), "Meiji 45");
    }

    #[test]
    fn test_jikoku_en() {
        assert_eq!(jikoku_en("未三つ"), "Hitsuji Mitsu");
        assert_eq!(jikoku_en("午一つ"), "Uma Hitotsu");
        assert_eq!(jikoku_en("子四つ"), "Ne Yotsu");
        assert_eq!(jikoku_en("卯二つ"), "U Futatsu");
    }

    #[test]
    fn test_solar_term_en() {
        assert_eq!(solar_term_en("夏至"), "Summer Solstice");
        assert_eq!(solar_term_en("冬至"), "Winter Solstice");
        assert_eq!(solar_term_en("春分"), "Vernal Equinox");
        assert_eq!(solar_term_en("秋分"), "Autumnal Equinox");
    }

    #[test]
    fn test_eto_en() {
        assert_eq!(eto_en("丙午"), "Hinoe-Uma");
        assert_eq!(eto_en("甲子"), "Kinoe-Ne");
        assert_eq!(eto_en("癸亥"), "Mizunoto-I");
    }

    #[test]
    fn test_lang_parse() {
        assert_eq!(Lang::from_str("ja").unwrap(), Lang::Ja);
        assert_eq!(Lang::from_str("en").unwrap(), Lang::En);
        assert_eq!(Lang::from_str("EN").unwrap(), Lang::En);
        assert_eq!(Lang::from_str("Japanese").unwrap(), Lang::Ja);
        assert!(Lang::from_str("fr").is_err());
    }
}
