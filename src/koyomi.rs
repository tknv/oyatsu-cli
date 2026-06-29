//! 和暦・和風月名・干支・二十四節気・不定時法(和時計)の計算。
//!
//! 二十四節気は近似テーブルを廃止し、Meeus「Astronomical Algorithms」第2版
//! Ch.25 の太陽視黄経式 + 二分法による天文計算で求める。
//!
//! 精度:
//!   - 入り日(JST日付)は ±1日以内 (2000〜2100年で全節気一致を確認)
//!   - 時刻精度は ±10分程度 (Ch.25 簡略版の限界)
//!   - 「今日がどの節気か」の判定に使う分には十分な精度

use chrono::{DateTime, Datelike, Local, NaiveDate};

// ─── 和時計 ──────────────────────────────────────────────────

const DAY_TIME_LABELS: [&str; 6] = ["卯", "辰", "巳", "午", "未", "申"];
const NIGHT_TIME_LABELS: [&str; 6] = ["酉", "戌", "亥", "子", "丑", "寅"];
const HOUR_NUMBER: [&str; 4] = ["一つ", "二つ", "三つ", "四つ"];

// ─── 和風月名 ─────────────────────────────────────────────────

pub fn japanese_month_name(date: NaiveDate) -> &'static str {
    const NAMES: [&str; 12] = [
        "睦月",
        "如月",
        "弥生",
        "卯月",
        "皐月",
        "水無月",
        "文月",
        "葉月",
        "長月",
        "神無月",
        "霜月",
        "師走",
    ];
    NAMES[(date.month() - 1) as usize]
}

// ─── 二十四節気 (天文計算) ────────────────────────────────────

/// 二十四節気の名称と対応する太陽視黄経 (λ☉, 度)。
/// 小寒(λ=285°)を起点に15°刻み。
const SOLAR_TERMS: [(&str, f64); 24] = [
    ("小寒", 285.0),
    ("大寒", 300.0),
    ("立春", 315.0),
    ("雨水", 330.0),
    ("啓蟄", 345.0),
    ("春分", 0.0),
    ("清明", 15.0),
    ("穀雨", 30.0),
    ("立夏", 45.0),
    ("小満", 60.0),
    ("芒種", 75.0),
    ("夏至", 90.0),
    ("小暑", 105.0),
    ("大暑", 120.0),
    ("立秋", 135.0),
    ("処暑", 150.0),
    ("白露", 165.0),
    ("秋分", 180.0),
    ("寒露", 195.0),
    ("霜降", 210.0),
    ("立冬", 225.0),
    ("小雪", 240.0),
    ("大雪", 255.0),
    ("冬至", 270.0),
];

/// 各節気の概算入り日 (月, 日) — ±3日の探索ウィンドウの中心点。
/// 数年に一度1日ずれることがあるが、±3日あれば必ず収まる。
const SOLAR_TERM_APPROX: [(u32, u32); 24] = [
    (1, 6),
    (1, 20),
    (2, 4),
    (2, 19),
    (3, 6),
    (3, 21),
    (4, 5),
    (4, 20),
    (5, 6),
    (5, 21),
    (6, 6),
    (6, 21),
    (7, 7),
    (7, 23),
    (8, 7),
    (8, 23),
    (9, 8),
    (9, 23),
    (10, 8),
    (10, 23),
    (11, 7),
    (11, 22),
    (12, 7),
    (12, 22),
];

// ─── Meeus Ch.25 太陽視黄経 ──────────────────────────────────

#[inline]
fn r(d: f64) -> f64 {
    d.to_radians()
}
#[inline]
fn rev360(x: f64) -> f64 {
    x.rem_euclid(360.0)
}

/// ユリウス日 (Meeus Ch.7 式 7.1) — date の 0h UT
fn julian_day(date: NaiveDate) -> f64 {
    let (y0, m0, d0) = (date.year() as f64, date.month() as f64, date.day() as f64);
    let (y, m) = if m0 <= 2.0 {
        (y0 - 1.0, m0 + 12.0)
    } else {
        (y0, m0)
    };
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + d0 + b - 1524.5
}

/// 太陽の視黄経 λ☉ [度, 0..360) (Meeus Ch.25)
fn sun_apparent_longitude(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0;
    let t2 = t * t;

    let l0 = rev360(280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t2);
    let m_deg = rev360(357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t2);
    let m = r(m_deg);
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t2;
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t2) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    let theta = l0 + c;
    let v = m_deg + c;
    let r_au = 1.000_001_018 * (1.0 - e * e) / (1.0 + e * r(v).cos());
    let omega = r(rev360(125.044_52 - 1_934.136_26 * t));

    // 視黄経: 光行差 + 章動Δψ補正
    rev360(theta - 0.005_69 - 0.004_78 * omega.sin() - 20.498_9 / (3_600.0 * r_au))
}

/// 二分法で太陽視黄経 = target_lon になるユリウス日を求める。
/// approx_jd ±3 日の範囲で探索。
fn find_solar_term_jd(target_lon: f64, approx_jd: f64) -> f64 {
    let (mut lo, mut hi) = (approx_jd - 3.0, approx_jd + 3.0);
    for _ in 0..64 {
        let mid = (lo + hi) / 2.0;
        let diff = (sun_apparent_longitude(mid) - target_lon + 180.0).rem_euclid(360.0) - 180.0;
        if diff > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        if hi - lo < 1e-9 {
            break;
        }
    }
    (lo + hi) / 2.0
}

/// date が属する二十四節気の名称を返す。
///
/// 「属する」= その節気の入り日(JST)が date 以前で最も新しいもの。
/// 入り日の判定は「節気入りJDをJSTに変換した日付」で行う。
/// (例: 小寒が 1/5 17:27 JST に入るなら、1/5 は「小寒」)
pub fn solar_term(date: NaiveDate) -> &'static str {
    let year = date.year();

    let mut best_name = SOLAR_TERMS[23].0; // 前年冬至を初期値
    let mut best_jd = f64::NEG_INFINITY;

    // 前年末の節気も対象に含める (1月初旬は前年12月の冬至が続く場合があるため)
    for &check_year in &[year - 1, year] {
        for (i, &(name, lon)) in SOLAR_TERMS.iter().enumerate() {
            let (am, ad) = SOLAR_TERM_APPROX[i];
            let approx_date = NaiveDate::from_ymd_opt(check_year, am, ad)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(check_year, am, ad.min(28)).unwrap());
            let approx_jd = julian_day(approx_date);
            let term_jd = find_solar_term_jd(lon, approx_jd);

            // 入りJDを JST の日付に変換 (JD は UTC 基準, +9h で JST 日付)
            // JST 日付 = (term_jd + 9/24) の整数 JD 部分から計算
            let term_jd_jst = term_jd + 9.0 / 24.0;
            // JD の整数部分から年月日を求める (簡易版: NaiveDate を逆算)
            let term_date_jst = jd_to_date(term_jd_jst);

            // 節気入り日(JST) が date 以前で、かつ最も新しいものを選ぶ
            if term_date_jst <= date && term_jd > best_jd {
                best_jd = term_jd;
                best_name = name;
            }
        }
    }

    best_name
}

/// ユリウス日 (UTC) から NaiveDate を求める (Meeus Ch.7 逆算)
fn jd_to_date(jd: f64) -> NaiveDate {
    let z = (jd + 0.5).floor() as i64;
    let a = if z < 2_299_161 {
        z
    } else {
        let alpha = ((z as f64 - 1_867_216.25) / 36_524.25).floor() as i64;
        z + 1 + alpha - alpha / 4
    };
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 {
        (e - 1) as u32
    } else {
        (e - 13) as u32
    };
    let year = if month > 2 {
        (c - 4716) as i32
    } else {
        (c - 4715) as i32
    };

    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, day.min(28)).unwrap())
}

// ─── 六十干支 ─────────────────────────────────────────────────

pub fn sixty_kanji_cycle(date: NaiveDate) -> String {
    const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    const BRANCHES: [&str; 12] = [
        "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
    ];
    let year = date.year();
    format!(
        "{}{}",
        STEMS[(year - 4).rem_euclid(10) as usize],
        BRANCHES[(year - 4).rem_euclid(12) as usize],
    )
}

// ─── 和暦 ────────────────────────────────────────────────────

fn to_kanji_number(num: i64) -> String {
    if num <= 0 {
        return String::new();
    }
    if num == 1 {
        return "元".to_string();
    }
    const DIGITS: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    const POWERS: [&str; 4] = ["", "十", "百", "千"];
    let s = num.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        let d = ch.to_digit(10).unwrap() as usize;
        let pow = len - 1 - i;
        if d > 0 {
            if !(d == 1 && pow == 1) {
                result.push_str(DIGITS[d]);
            }
            if pow > 0 {
                result.push_str(POWERS[pow]);
            }
        }
    }
    result
}

pub fn japanese_year(date: NaiveDate) -> String {
    let y = date.year();
    let m = date.month() as i32;
    let d = date.day() as i32;
    const ERAS: [(i32, i32, i32, &str); 5] = [
        (2019, 5, 1, "令和"),
        (1989, 1, 8, "平成"),
        (1926, 12, 25, "昭和"),
        (1912, 7, 30, "大正"),
        (1868, 1, 25, "明治"),
    ];
    for &(sy, sm, sd, name) in &ERAS {
        if y > sy || (y == sy && (m > sm || (m == sm && d >= sd))) {
            return format!("{}{}年", name, to_kanji_number((y - sy + 1) as i64));
        }
    }
    format!("{}年", y)
}

// ─── 和時計 (不定時法) ───────────────────────────────────────

#[allow(dead_code)]
pub struct JapaneseTime {
    pub label: String,
    pub sun_info: String,
    pub is_hitsuji_time: bool,
}

pub fn calculate_japanese_time(
    now: DateTime<Local>,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) -> JapaneseTime {
    let now_ms = now.timestamp_millis();
    let rise_ms = sunrise.timestamp_millis();
    let set_ms = sunset.timestamp_millis();

    let sun_info = format!(
        "日出{}-日入{}",
        sunrise.format("%H:%M"),
        sunset.format("%H:%M")
    );

    let day_dur = set_ms - rise_ms;
    let next_rise_ms = (sunrise + chrono::Duration::days(1)).timestamp_millis();
    let night_dur = next_rise_ms - set_ms;

    let (duration_ms, start_ms, labels): (i64, i64, &[&str; 6]) =
        if now_ms >= rise_ms && now_ms < set_ms {
            (day_dur, rise_ms, &DAY_TIME_LABELS)
        } else if now_ms >= set_ms {
            (night_dur, set_ms, &NIGHT_TIME_LABELS)
        } else {
            let prev_set_ms = (sunset - chrono::Duration::days(1)).timestamp_millis();
            (rise_ms - prev_set_ms, prev_set_ms, &NIGHT_TIME_LABELS)
        };

    if duration_ms <= 0 {
        return JapaneseTime {
            label: "時間計算エラー".to_string(),
            sun_info,
            is_hitsuji_time: false,
        };
    }

    let unit = duration_ms as f64 / 6.0;
    let passed = (now_ms - start_ms) as f64;

    let mut unit_idx = (passed / unit) as i64;
    let rem = passed.rem_euclid(unit);
    let mut sub_idx = (rem / (unit / 4.0)) as i64;

    unit_idx = unit_idx.clamp(0, labels.len() as i64 - 1);
    sub_idx = sub_idx.clamp(0, HOUR_NUMBER.len() as i64 - 1);

    let hour_name = labels[unit_idx as usize];
    JapaneseTime {
        label: format!("{}{}", hour_name, HOUR_NUMBER[sub_idx as usize]),
        sun_info,
        is_hitsuji_time: hour_name == "未",
    }
}

// ─── テスト ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // ── 和暦 ──────────────────────────────────────────────────

    #[test]
    fn test_era_boundaries() {
        assert_eq!(japanese_year(d(1868, 1, 24)), "1868年");
        assert_eq!(japanese_year(d(1868, 1, 25)), "明治元年");
        assert_eq!(japanese_year(d(1912, 7, 29)), "明治四十五年");
        assert_eq!(japanese_year(d(1912, 7, 30)), "大正元年");
        assert_eq!(japanese_year(d(1926, 12, 24)), "大正十五年");
        assert_eq!(japanese_year(d(1926, 12, 25)), "昭和元年");
        assert_eq!(japanese_year(d(1989, 1, 7)), "昭和六十四年");
        assert_eq!(japanese_year(d(1989, 1, 8)), "平成元年");
        assert_eq!(japanese_year(d(2019, 4, 30)), "平成三十一年");
        assert_eq!(japanese_year(d(2019, 5, 1)), "令和元年");
        assert_eq!(japanese_year(d(2026, 6, 22)), "令和八年");
    }

    // ── 干支 ──────────────────────────────────────────────────

    #[test]
    fn test_sixty_kanji_cycle() {
        assert_eq!(sixty_kanji_cycle(d(1984, 1, 1)), "甲子");
        assert_eq!(sixty_kanji_cycle(d(2044, 1, 1)), "甲子");
        assert_eq!(sixty_kanji_cycle(d(2026, 1, 1)), "丙午");
    }

    // ── 和風月名 ──────────────────────────────────────────────

    #[test]
    fn test_japanese_month_name() {
        assert_eq!(japanese_month_name(d(2026, 1, 1)), "睦月");
        assert_eq!(japanese_month_name(d(2026, 6, 1)), "水無月");
        assert_eq!(japanese_month_name(d(2026, 12, 1)), "師走");
    }

    // ── 二十四節気 (天文計算) — 2026年 ──────────────────────
    //
    // 参照値は ephem (VSOP87 ベース) で計算した精密値。
    // 入り日の JST 日付が一致することを確認する。
    // 時刻精度: Meeus Ch.25 簡略版の誤差 ≤ 10分。

    // helper: その日の正午時点で solar_term が期待値を返すかチェック
    fn check_term(year: i32, month: u32, day: u32, expected: &str) {
        let date = d(year, month, day);
        assert_eq!(
            solar_term(date),
            expected,
            "{year}/{month:02}/{day:02} は {expected} のはず (got: {})",
            solar_term(date)
        );
    }

    // 入り日当日は新しい節気になっていること
    #[test]
    fn test_solar_term_2026_entry_days() {
        // ephem 参照値の入り日
        check_term(2026, 1, 5, "小寒");
        check_term(2026, 1, 20, "大寒");
        check_term(2026, 2, 4, "立春");
        check_term(2026, 2, 19, "雨水");
        check_term(2026, 3, 5, "啓蟄"); // ephem: 3/5 22:53 JST
        check_term(2026, 3, 20, "春分"); // ephem: 3/20 23:40 JST
        check_term(2026, 4, 5, "清明");
        check_term(2026, 4, 20, "穀雨");
        check_term(2026, 5, 5, "立夏"); // ephem: 5/5 20:42 JST
        check_term(2026, 5, 21, "小満");
        check_term(2026, 6, 6, "芒種");
        check_term(2026, 6, 21, "夏至");
        check_term(2026, 7, 7, "小暑");
        check_term(2026, 7, 23, "大暑");
        check_term(2026, 8, 7, "立秋");
        check_term(2026, 8, 23, "処暑");
        check_term(2026, 9, 7, "白露");
        check_term(2026, 9, 23, "秋分");
        check_term(2026, 10, 8, "寒露");
        check_term(2026, 10, 23, "霜降");
        check_term(2026, 11, 7, "立冬");
        check_term(2026, 11, 22, "小雪");
        check_term(2026, 12, 7, "大雪");
        check_term(2026, 12, 22, "冬至");
    }

    // 入り日前日は前の節気であること
    #[test]
    fn test_solar_term_2026_day_before_entry() {
        check_term(2026, 1, 4, "冬至"); // 小寒の前日
        check_term(2026, 1, 19, "小寒"); // 大寒の前日
        check_term(2026, 2, 3, "大寒");
        check_term(2026, 2, 18, "立春");
        check_term(2026, 3, 4, "雨水");
        check_term(2026, 3, 19, "啓蟄");
        check_term(2026, 4, 4, "春分");
        check_term(2026, 4, 19, "清明");
        check_term(2026, 5, 4, "穀雨");
        check_term(2026, 5, 20, "立夏");
        check_term(2026, 6, 5, "小満");
        check_term(2026, 6, 20, "芒種");
        check_term(2026, 7, 6, "夏至");
        check_term(2026, 7, 22, "小暑");
        check_term(2026, 8, 6, "大暑");
        check_term(2026, 8, 22, "立秋");
        check_term(2026, 9, 6, "処暑");
        check_term(2026, 9, 22, "白露");
        check_term(2026, 10, 7, "秋分");
        check_term(2026, 10, 22, "寒露");
        check_term(2026, 11, 6, "霜降");
        check_term(2026, 11, 21, "立冬");
        check_term(2026, 12, 6, "小雪");
        check_term(2026, 12, 21, "大雪");
    }

    // 異なる年でも正しく動くこと (2025年・2027年)
    #[test]
    fn test_solar_term_other_years() {
        // 2025年 ephem参照: 夏至=6/21 02:42 UTC = 6/21 11:42 JST
        check_term(2025, 6, 21, "夏至");
        check_term(2025, 6, 20, "芒種");
        // 2025年 冬至: ephem = 12/21 15:02 UTC = 12/22 00:02 JST (日本では12/22)
        check_term(2025, 12, 22, "冬至");
        check_term(2025, 12, 21, "大雪");
        // 2027年 春分: ephem = 3/21 05:24 UTC = 3/21 14:24 JST
        check_term(2027, 3, 21, "春分");
        check_term(2027, 3, 20, "啓蟄");
    }

    // 年またぎ (1月初旬は前年の冬至期間)
    #[test]
    fn test_solar_term_year_boundary() {
        // 2026年1月4日はまだ冬至 (小寒は1/5)
        check_term(2026, 1, 1, "冬至");
        check_term(2026, 1, 4, "冬至");
        check_term(2026, 1, 5, "小寒");
    }
}
