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

// ─── 日付 (漢数字表記) ────────────────────────────────────────

/// 日付の「日」を伝統的な漢数字表記にする (廿を用いる)。
///
/// 例: 2日→"二日", 10日→"十日", 13日→"十三日",
///     20日→"廿日", 25日→"廿五日", 31日→"三十一日"
pub fn kanji_day(date: NaiveDate) -> String {
    const DIGITS: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let day = date.day();
    let body = match day {
        1..=9 => DIGITS[day as usize].to_string(),
        10 => "十".to_string(),
        11..=19 => format!("十{}", DIGITS[(day - 10) as usize]),
        20 => "廿".to_string(),
        21..=29 => format!("廿{}", DIGITS[(day - 20) as usize]),
        30 => "三十".to_string(),
        31 => "三十一".to_string(),
        _ => day.to_string(),
    };
    format!("{body}日")
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

// ─── 社日計算ヘルパー ──────────────────────────────────────────

/// 指定した日付の「日の十干」を返す (0:甲, 1:乙, 2:丙, 3:丁, 4:戊, 5:己, 6:庚, 7:辛, 8:壬, 9:癸)
fn jikkan_of_date(date: NaiveDate) -> i64 {
    // 0h UT のユリウス日を取得し、0.5 を足して正午のユリウス日(整数)にする
    let jd_0h = julian_day(date);
    let jd_12h = (jd_0h + 0.5).round() as i64;
    // 基準日: 2000年1月1日 (JD=2451545) は「戊(4)」
    (jd_12h + 9).rem_euclid(10)
}

/// その年の社日（春社または秋社）を計算する
fn calc_shanichi(year: i32, is_spring: bool) -> NaiveDate {
    // 春分(0°) と 秋分(180°) の黄経と概算月日
    let (lon, m, d) = if is_spring {
        (0.0, 3, 21)
    } else {
        (180.0, 9, 23)
    };

    // 天文計算による正確な春分・秋分点 (UTC) のユリウス日
    let approx_date = NaiveDate::from_ymd_opt(year, m, d)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, m, 28).unwrap());
    let term_jd_utc = find_solar_term_jd(lon, julian_day(approx_date));

    // JST (+9時間) に変換
    let term_jd_jst = term_jd_utc + 9.0 / 24.0;
    let eq_date = jd_to_date(term_jd_jst);

    // JSTでの「時刻 (0.0 〜 24.0)」を計算
    // ユリウス日は正午(12:00)が整数となるため、+0.5 の端数から時刻を求める
    let hours = (term_jd_jst + 0.5).rem_euclid(1.0) * 24.0;

    // 春分・秋分当日の十干を求める
    let jikkan = jikkan_of_date(eq_date);

    // 戊(インデックス4) の日までの差分日数を計算 (-4 〜 +5)
    let mut offset = (4 - jikkan).rem_euclid(10);
    if offset > 5 {
        offset -= 10; // -4, -3, -2, -1 のいずれかにする
    } else if offset == 5 {
        // 春分・秋分が「癸(9)」の日だった場合、前後5日がどちらも戊となる
        // 【明治14年以後のルール】午前中なら前(-5日)、午後なら後(+5日)
        if hours < 12.0 {
            offset = -5;
        } else {
            offset = 5;
        }
    }

    // 春分/秋分の日付にオフセットを加算して社日を返す
    eq_date + chrono::Duration::days(offset)
}

// ─── 雑節 ────────────────────────────────────────────────────

/// 指定した日付が雑節に該当するか判定し、該当する場合はその名称を返す。
/// 毎日表示するものではないため Option で返す。
pub fn zassetsu(date: NaiveDate) -> Option<&'static str> {
    let y = date.year();

    // 太陽黄経(lon)と概算月日を与えて、その年の正確なJST日付を算出するヘルパー
    let get_term_date = |lon: f64, am: u32, ad: u32| -> NaiveDate {
        let approx_date = NaiveDate::from_ymd_opt(y, am, ad)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(y, am, 28).unwrap());
        let approx_jd = julian_day(approx_date);
        let term_jd = find_solar_term_jd(lon, approx_jd);
        jd_to_date(term_jd + 9.0 / 24.0) // +9時間でJSTに変換
    };

    // 基準となる日の計算
    let risshun = get_term_date(315.0, 2, 4);
    let shunbun = get_term_date(0.0, 3, 21);
    let shubun = get_term_date(180.0, 9, 23);

    // 1. 節分 (立春の前日)
    if date == risshun - chrono::Duration::days(1) {
        return Some("節分");
    }

    // 2. 八十八夜 (立春を1日目として88日目 = +87日)
    if date == risshun + chrono::Duration::days(87) {
        return Some("八十八夜");
    }

    // 3. 二百十日 (立春を1日目として210日目 = +209日)
    if date == risshun + chrono::Duration::days(209) {
        return Some("二百十日");
    }

    // 4. 彼岸入り (春分・秋分の3日前。暦面では入りのみが示される)
    if date == shunbun - chrono::Duration::days(3) || date == shubun - chrono::Duration::days(3) {
        return Some("彼岸入り");
    }

    // 5. 土用入り (立春・立夏・立秋・立冬の直前。太陽黄経297°, 27°, 117°, 207°)
    // ※ 冬の土用入り(297°)は1月なので、立春と同じ年の1月で計算可能
    let doyo_w = get_term_date(297.0, 1, 17);
    let doyo_sp = get_term_date(27.0, 4, 17);
    let doyo_su = get_term_date(117.0, 7, 19);
    let doyo_au = get_term_date(207.0, 10, 20);
    if date == doyo_w || date == doyo_sp || date == doyo_su || date == doyo_au {
        return Some("土用入り");
    }

    // 6. 入梅 (太陽黄経80°)
    let nyubai = get_term_date(80.0, 6, 11);
    if date == nyubai {
        return Some("入梅");
    }

    // 7. 半夏生 (太陽黄経100°)
    let hangesho = get_term_date(100.0, 7, 2);
    if date == hangesho {
        return Some("半夏生");
    }

    // 8. 社日 (春社・秋社)
    let shunsha = calc_shanichi(y, true);
    let shusha = calc_shanichi(y, false);
    if date == shunsha || date == shusha {
        return Some("社日");
    }

    None
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

    // ── 日付 (漢数字) ─────────────────────────────────────────

    #[test]
    fn test_kanji_day() {
        assert_eq!(kanji_day(d(2026, 1, 1)), "一日");
        assert_eq!(kanji_day(d(2026, 1, 2)), "二日");
        assert_eq!(kanji_day(d(2026, 1, 9)), "九日");
        assert_eq!(kanji_day(d(2026, 1, 10)), "十日");
        assert_eq!(kanji_day(d(2026, 1, 13)), "十三日");
        assert_eq!(kanji_day(d(2026, 1, 19)), "十九日");
        assert_eq!(kanji_day(d(2026, 1, 20)), "廿日");
        assert_eq!(kanji_day(d(2026, 1, 25)), "廿五日");
        assert_eq!(kanji_day(d(2026, 1, 29)), "廿九日");
        assert_eq!(kanji_day(d(2026, 1, 30)), "三十日");
        assert_eq!(kanji_day(d(2026, 1, 31)), "三十一日");
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

    // ── 雑節 ──────────────────────────────────────────────────
    //
    // 参照値は 国立天文台「暦要項」ベースの実際の雑節日 (2024〜2028年) と
    // 照合済み (節分・入梅・半夏生・土用入り・彼岸入り・社日など、
    // いずれも公表されている実日付と一致することを確認)。

    // 節分 (立春の前日)
    #[test]
    fn test_zassetsu_setsubun() {
        assert_eq!(zassetsu(d(2024, 2, 3)), Some("節分"));
        // 2025年は124年ぶりに2/2が節分になった年 (立春が2/3のため)
        assert_eq!(zassetsu(d(2025, 2, 2)), Some("節分"));
        assert_eq!(zassetsu(d(2026, 2, 3)), Some("節分"));

        // 前後の日は該当しない
        assert_eq!(zassetsu(d(2026, 2, 2)), None);
        assert_eq!(zassetsu(d(2026, 2, 4)), None); // 立春当日自体は雑節ではない
    }

    // 八十八夜 (立春から数えて88日目)
    #[test]
    fn test_zassetsu_hachijuhachiya() {
        assert_eq!(zassetsu(d(2024, 5, 1)), Some("八十八夜"));
        assert_eq!(zassetsu(d(2025, 5, 1)), Some("八十八夜"));
        assert_eq!(zassetsu(d(2026, 5, 2)), Some("八十八夜"));

        assert_eq!(zassetsu(d(2026, 5, 1)), None);
        assert_eq!(zassetsu(d(2026, 5, 3)), None);
    }

    // 二百十日 (立春から数えて210日目)
    #[test]
    fn test_zassetsu_nihyakutoka() {
        assert_eq!(zassetsu(d(2024, 8, 31)), Some("二百十日"));
        assert_eq!(zassetsu(d(2025, 8, 31)), Some("二百十日"));
        assert_eq!(zassetsu(d(2026, 9, 1)), Some("二百十日"));

        assert_eq!(zassetsu(d(2026, 8, 31)), None);
        assert_eq!(zassetsu(d(2026, 9, 2)), None);
    }

    // 彼岸入り (春分・秋分の3日前)
    #[test]
    fn test_zassetsu_higan_iri() {
        // 春の彼岸入り
        assert_eq!(zassetsu(d(2024, 3, 17)), Some("彼岸入り"));
        assert_eq!(zassetsu(d(2025, 3, 17)), Some("彼岸入り"));
        assert_eq!(zassetsu(d(2026, 3, 17)), Some("彼岸入り"));
        assert_eq!(zassetsu(d(2027, 3, 18)), Some("彼岸入り"));

        // 秋の彼岸入り
        assert_eq!(zassetsu(d(2024, 9, 19)), Some("彼岸入り"));
        assert_eq!(zassetsu(d(2025, 9, 20)), Some("彼岸入り"));
        assert_eq!(zassetsu(d(2026, 9, 20)), Some("彼岸入り"));

        // 前後の日は該当しない
        assert_eq!(zassetsu(d(2026, 3, 16)), None);
        assert_eq!(zassetsu(d(2026, 3, 18)), None);
    }

    // 土用入り (立春・立夏・立秋・立冬の直前、太陽黄経297°/27°/117°/207°)
    #[test]
    fn test_zassetsu_doyo_iri() {
        // 2026年: 冬・春・夏・秋の四回
        assert_eq!(zassetsu(d(2026, 1, 17)), Some("土用入り")); // 冬の土用
        assert_eq!(zassetsu(d(2026, 4, 17)), Some("土用入り")); // 春の土用
        assert_eq!(zassetsu(d(2026, 7, 20)), Some("土用入り")); // 夏の土用
        assert_eq!(zassetsu(d(2026, 10, 20)), Some("土用入り")); // 秋の土用

        // 他の年でもズレて正しく計算されること
        assert_eq!(zassetsu(d(2025, 7, 19)), Some("土用入り"));
        assert_eq!(zassetsu(d(2027, 10, 21)), Some("土用入り"));

        // 前後の日は該当しない
        assert_eq!(zassetsu(d(2026, 1, 16)), None);
        assert_eq!(zassetsu(d(2026, 1, 18)), None);
    }

    // 入梅 (太陽黄経80°)
    #[test]
    fn test_zassetsu_nyubai() {
        assert_eq!(zassetsu(d(2024, 6, 10)), Some("入梅"));
        assert_eq!(zassetsu(d(2025, 6, 11)), Some("入梅"));
        assert_eq!(zassetsu(d(2026, 6, 11)), Some("入梅"));

        assert_eq!(zassetsu(d(2026, 6, 10)), None);
        assert_eq!(zassetsu(d(2026, 6, 12)), None);
    }

    // 半夏生 (太陽黄経100°)
    #[test]
    fn test_zassetsu_hangesho() {
        assert_eq!(zassetsu(d(2024, 7, 1)), Some("半夏生"));
        assert_eq!(zassetsu(d(2025, 7, 1)), Some("半夏生"));
        assert_eq!(zassetsu(d(2026, 7, 2)), Some("半夏生"));

        assert_eq!(zassetsu(d(2026, 7, 1)), None);
        assert_eq!(zassetsu(d(2026, 7, 3)), None);
    }

    // 社日 (春分・秋分に最も近い戊の日)
    #[test]
    fn test_zassetsu_shanichi() {
        // 春社
        assert_eq!(zassetsu(d(2024, 3, 25)), Some("社日"));
        assert_eq!(zassetsu(d(2025, 3, 20)), Some("社日"));
        assert_eq!(zassetsu(d(2026, 3, 25)), Some("社日"));

        // 秋社
        assert_eq!(zassetsu(d(2024, 9, 21)), Some("社日"));
        assert_eq!(zassetsu(d(2025, 9, 26)), Some("社日"));
        assert_eq!(zassetsu(d(2026, 9, 21)), Some("社日"));

        // 前後の日(戊ではない日)は該当しない
        assert_eq!(zassetsu(d(2026, 3, 24)), None);
        assert_eq!(zassetsu(d(2026, 3, 26)), None);
    }

    // 雑節に該当しない、ごく普通の日は None を返すこと
    #[test]
    fn test_zassetsu_ordinary_day_returns_none() {
        assert_eq!(zassetsu(d(2026, 1, 1)), None);
        assert_eq!(zassetsu(d(2026, 6, 15)), None);
        assert_eq!(zassetsu(d(2026, 8, 15)), None);
        assert_eq!(zassetsu(d(2026, 12, 25)), None);
    }

    // 2026年全体を走査し、雑節に該当する日と名称の一覧が
    // 期待通りであることを一括で確認する回帰テスト。
    #[test]
    fn test_zassetsu_2026_full_year_scan() {
        let expected: [(NaiveDate, &str); 13] = [
            (d(2026, 1, 17), "土用入り"),
            (d(2026, 2, 3), "節分"),
            (d(2026, 3, 17), "彼岸入り"),
            (d(2026, 3, 25), "社日"),
            (d(2026, 4, 17), "土用入り"),
            (d(2026, 5, 2), "八十八夜"),
            (d(2026, 6, 11), "入梅"),
            (d(2026, 7, 2), "半夏生"),
            (d(2026, 7, 20), "土用入り"),
            (d(2026, 9, 1), "二百十日"),
            (d(2026, 9, 20), "彼岸入り"),
            (d(2026, 9, 21), "社日"),
            (d(2026, 10, 20), "土用入り"),
        ];

        let mut actual: Vec<(NaiveDate, &str)> = Vec::new();
        let mut day = d(2026, 1, 1);
        let end = d(2026, 12, 31);
        while day <= end {
            if let Some(name) = zassetsu(day) {
                actual.push((day, name));
            }
            day += chrono::Duration::days(1);
        }

        assert_eq!(actual, expected.to_vec());
    }
}
