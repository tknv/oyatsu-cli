//! 月相 (朔望) の計算。
//!
//! Jean Meeus 「Astronomical Algorithms」第2版 (Willmann-Bell, 1998) Ch.49
//! 「Phases of the Moon」に準拠し、朔(New)・上弦(First Quarter)・
//! 望(Full)・下弦(Last Quarter) それぞれの正確な瞬時 (時刻付き) を求める。
//!
//! sunrise.rs / koyomi.rs と同様に ΔT (地球時-世界時の差, ≈70秒) は
//! 無視する (日付レベルの判定には影響しない)。
//!
//! 精度: Meeus の周期項をほぼ全て実装しているため、瞬時の誤差は数分程度。
//! 「その朔望がどの日付(JST)に入るか」の判定には十分な精度。

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};

use crate::koyomi;

// ─────────────────────────────────────────────
// 朔望の種別
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    /// 朔 (新月)
    New,
    /// 上弦
    FirstQuarter,
    /// 望 (満月)
    Full,
    /// 下弦
    LastQuarter,
}

impl MoonPhase {
    pub fn name_ja(self) -> &'static str {
        match self {
            MoonPhase::New => "朔",
            MoonPhase::FirstQuarter => "上弦",
            MoonPhase::Full => "望",
            MoonPhase::LastQuarter => "下弦",
        }
    }

    pub fn name_en(self) -> &'static str {
        match self {
            MoonPhase::New => "New Moon",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::Full => "Full Moon",
            MoonPhase::LastQuarter => "Last Quarter",
        }
    }

    /// 0=朔, 1=上弦, 2=望, 3=下弦
    fn from_offset(offset: u8) -> Self {
        match offset {
            0 => MoonPhase::New,
            1 => MoonPhase::FirstQuarter,
            2 => MoonPhase::Full,
            3 => MoonPhase::LastQuarter,
            _ => unreachable!("phase offset は 0..=3"),
        }
    }
}

// ─────────────────────────────────────────────
// ユーティリティ
// ─────────────────────────────────────────────

#[inline]
fn r(d: f64) -> f64 {
    d.to_radians()
}
#[inline]
fn rev360(x: f64) -> f64 {
    x.rem_euclid(360.0)
}

// ─────────────────────────────────────────────
// k (朔望月インデックス) の近似値
// ─────────────────────────────────────────────

/// 指定日に対応する、朔望月インデックス k の近似値 (Meeus 式 49.1 の逆算)。
///
/// k = 0 は 2000年1月6日頃の朔に対応する。k が 0.25 刻みで
/// 朔(+0.00)・上弦(+0.25)・望(+0.50)・下弦(+0.75) に対応する。
fn approx_k(date: NaiveDate) -> f64 {
    let year_frac = date.year() as f64 + date.ordinal() as f64 / 365.25;
    (year_frac - 2000.0) * 12.368_5
}

// ─────────────────────────────────────────────
// Meeus Ch.49: 平均要素
// ─────────────────────────────────────────────

struct PhaseAngles {
    e: f64,
    m: f64,
    mp: f64,
    f: f64,
    omega: f64,
    a: [f64; 14],
}

/// k (0.25刻み) から、平均朔望瞬時 JDE(TD) と各補正用平均角を求める (式 49.1〜49.3)。
fn mean_jde_and_angles(k: f64) -> (f64, PhaseAngles) {
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let jde0 = 2_451_550.097_66
        + 29.530_588_861 * k
        + 0.000_154_37 * t2
        - 0.000_000_150 * t3
        + 0.000_000_000_73 * t4;

    let e = 1.0 - 0.002_516 * t - 0.000_0074 * t2;

    let m = rev360(2.5534 + 29.105_356_69 * k - 0.000_0218 * t2 - 0.000_000_11 * t3);
    let mp = rev360(
        201.5643 + 385.816_935_28 * k + 0.010_7582 * t2 + 0.000_012_38 * t3
            - 0.000_000_058 * t4,
    );
    let f = rev360(
        160.7108 + 390.670_502_84 * k - 0.001_6118 * t2 - 0.000_002_27 * t3
            + 0.000_000_011 * t4,
    );
    let omega = rev360(124.7746 - 1.563_755_88 * k + 0.002_0672 * t2 + 0.000_002_15 * t3);

    let a = [
        rev360(299.77 + 0.107_408 * k - 0.009_173 * t2), // A1
        rev360(251.88 + 0.016_321 * k),                  // A2
        rev360(251.83 + 26.651_886 * k),                 // A3
        rev360(349.42 + 36.412_478 * k),                 // A4
        rev360(84.66 + 18.206_239 * k),                  // A5
        rev360(141.74 + 53.303_771 * k),                 // A6
        rev360(207.14 + 2.453_732 * k),                  // A7
        rev360(154.84 + 7.306_860 * k),                  // A8
        rev360(34.52 + 27.261_239 * k),                  // A9
        rev360(207.19 + 0.121_824 * k),                  // A10
        rev360(291.34 + 1.844_379 * k),                  // A11
        rev360(161.72 + 24.198_154 * k),                 // A12
        rev360(239.56 + 25.513_099 * k),                 // A13
        rev360(331.55 + 3.592_518 * k),                  // A14
    ];

    (
        jde0,
        PhaseAngles {
            e,
            m,
            mp,
            f,
            omega,
            a,
        },
    )
}

/// 朔・望に共通の周期補正項 (式 49.1)。
fn correction_new_full(p: &PhaseAngles) -> f64 {
    let (e, m, mp, f, omega) = (p.e, r(p.m), r(p.mp), r(p.f), r(p.omega));
    -0.407_20 * mp.sin()
        + 0.172_41 * e * m.sin()
        + 0.016_08 * (2.0 * mp).sin()
        + 0.010_39 * (2.0 * f).sin()
        + 0.007_39 * e * (mp - m).sin()
        - 0.005_14 * e * (mp + m).sin()
        + 0.002_08 * e * e * (2.0 * m).sin()
        - 0.001_11 * (mp - 2.0 * f).sin()
        - 0.000_57 * (mp + 2.0 * f).sin()
        + 0.000_56 * e * (2.0 * mp + m).sin()
        - 0.000_42 * (3.0 * mp).sin()
        + 0.000_42 * e * (m + 2.0 * f).sin()
        + 0.000_38 * e * (m - 2.0 * f).sin()
        - 0.000_24 * e * (2.0 * mp - m).sin()
        - 0.000_17 * omega.sin()
        - 0.000_07 * (mp + 2.0 * m).sin()
        + 0.000_04 * (2.0 * mp - 2.0 * f).sin()
        + 0.000_04 * (3.0 * m).sin()
        + 0.000_03 * (mp + m - 2.0 * f).sin()
        + 0.000_03 * (2.0 * mp + 2.0 * f).sin()
        - 0.000_03 * (mp + m + 2.0 * f).sin()
        + 0.000_03 * (mp - m + 2.0 * f).sin()
        - 0.000_02 * (mp - m - 2.0 * f).sin()
        - 0.000_02 * (3.0 * mp + m).sin()
        + 0.000_02 * (4.0 * mp).sin()
}

/// 上弦・下弦に共通の周期補正項 (式 49.1, quarters用)。
fn correction_quarter(p: &PhaseAngles) -> f64 {
    let (e, m, mp, f, omega) = (p.e, r(p.m), r(p.mp), r(p.f), r(p.omega));
    -0.628_01 * mp.sin()
        + 0.171_72 * e * m.sin()
        - 0.011_83 * e * (mp + m).sin()
        + 0.008_62 * (2.0 * mp).sin()
        + 0.008_04 * (2.0 * f).sin()
        + 0.004_54 * e * (mp - m).sin()
        + 0.002_04 * e * e * (2.0 * m).sin()
        - 0.001_80 * (mp - 2.0 * f).sin()
        - 0.000_70 * (mp + 2.0 * f).sin()
        - 0.000_40 * (3.0 * mp).sin()
        - 0.000_34 * e * (2.0 * mp - m).sin()
        + 0.000_32 * e * (m + 2.0 * f).sin()
        + 0.000_32 * e * (m - 2.0 * f).sin()
        - 0.000_28 * e * e * (mp + 2.0 * m).sin()
        + 0.000_27 * e * (2.0 * mp + m).sin()
        - 0.000_17 * omega.sin()
        - 0.000_05 * (mp - m - 2.0 * f).sin()
        + 0.000_04 * (2.0 * mp + 2.0 * f).sin()
        - 0.000_04 * (mp + m + 2.0 * f).sin()
        + 0.000_04 * (mp - 2.0 * m).sin()
        + 0.000_03 * (mp + m - 2.0 * f).sin()
        + 0.000_03 * (3.0 * m).sin()
        + 0.000_02 * (2.0 * mp - 2.0 * f).sin()
        + 0.000_02 * (mp - m + 2.0 * f).sin()
        - 0.000_02 * (mp + 3.0 * m).sin()
}

/// 上弦・下弦の非対称性を補正する追加項 W (式 49.1)。
/// 上弦では +W、下弦では -W を加える。
fn quarter_w(p: &PhaseAngles) -> f64 {
    let (e, m, mp, f) = (p.e, r(p.m), r(p.mp), r(p.f));
    0.003_06 - 0.000_38 * e * m.cos() + 0.000_26 * mp.cos() - 0.000_02 * (mp - m).cos()
        + 0.000_02 * (mp + m).cos()
        + 0.000_02 * (2.0 * f).cos()
}

/// 全ての朔望に共通の惑星摂動補正項 (式 49.1 末尾)。
fn planetary_correction(p: &PhaseAngles) -> f64 {
    const COEFF: [f64; 14] = [
        0.000_325, 0.000_165, 0.000_164, 0.000_126, 0.000_110, 0.000_062, 0.000_060, 0.000_056,
        0.000_047, 0.000_042, 0.000_040, 0.000_037, 0.000_035, 0.000_023,
    ];
    COEFF
        .iter()
        .zip(p.a.iter())
        .map(|(c, a)| c * r(*a).sin())
        .sum()
}

/// k_base (朔=整数) と phase_offset (0=朔,1=上弦,2=望,3=下弦) から
/// その朔望の瞬時を JDE(TD) で返す。
fn phase_jde(k_base: f64, phase_offset: u8) -> f64 {
    let k = k_base + phase_offset as f64 * 0.25;
    let (jde0, p) = mean_jde_and_angles(k);
    let planetary = planetary_correction(&p);

    match phase_offset {
        0 | 2 => jde0 + correction_new_full(&p) + planetary,
        1 => jde0 + correction_quarter(&p) + quarter_w(&p) + planetary,
        3 => jde0 + correction_quarter(&p) - quarter_w(&p) + planetary,
        _ => unreachable!("phase offset は 0..=3"),
    }
}

// ─────────────────────────────────────────────
// JDE(TD) → DateTime<Utc> 変換 (Meeus Ch.7 逆算 + 時刻)
// ─────────────────────────────────────────────

fn jde_to_datetime_utc(jde: f64) -> DateTime<Utc> {
    let jd05 = jde + 0.5;
    let z = jd05.floor();
    let day_frac_part = jd05 - z;
    let z_i = z as i64;

    let a = if z_i < 2_299_161 {
        z_i
    } else {
        let alpha = ((z - 1_867_216.25) / 36_524.25).floor() as i64;
        z_i + 1 + alpha - alpha / 4
    };
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day_int = b - d - (30.6001 * e as f64).floor() as i64;
    let month = if e < 14 { (e - 1) as u32 } else { (e - 13) as u32 };
    let year = if month > 2 { (c - 4716) as i32 } else { (c - 4715) as i32 };

    let date = NaiveDate::from_ymd_opt(year, month, (day_int as u32).clamp(1, 28))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 28).unwrap());
    // clampで28日に丸めた場合の補正 (29〜31日をここで加算する)
    let extra_days = day_int - date.day() as i64;

    let secs_total = (day_frac_part * 86_400.0).round() as i64;
    let base = date
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let ndt = base + Duration::days(extra_days) + Duration::seconds(secs_total);
    DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)
}

// ─────────────────────────────────────────────
// 公開 API
// ─────────────────────────────────────────────

pub struct NextMoonEvent {
    pub phase: MoonPhase,
    pub datetime: DateTime<Local>,
}

/// `after` 以降で最初に到来する朔望 (朔・上弦・望・下弦のいずれか) を返す。
pub fn next_phase(after: DateTime<Local>) -> NextMoonEvent {
    let k_center = approx_k(after.date_naive()).round();

    let mut best: Option<(DateTime<Local>, MoonPhase)> = None;

    // ±2 朔望月分探索すれば必ず「次の朔望」が見つかる (安全マージン込み)。
    for k_off in -2..=2 {
        let k_base = k_center + k_off as f64;
        for phase_offset in 0..4u8 {
            let jde = phase_jde(k_base, phase_offset);
            let dt_local = jde_to_datetime_utc(jde).with_timezone(&Local);

            if dt_local >= after {
                let is_better = match &best {
                    None => true,
                    Some((best_dt, _)) => dt_local < *best_dt,
                };
                if is_better {
                    best = Some((dt_local, MoonPhase::from_offset(phase_offset)));
                }
            }
        }
    }

    let (datetime, phase) = best.expect("探索範囲(±2朔望月)内に朔望が見つからない");
    NextMoonEvent { phase, datetime }
}

// ─────────────────────────────────────────────
// メッセージ整形
// ─────────────────────────────────────────────

/// 例: "三日後に朔", "十二日後に上弦", "今夜は望", "今夜は下弦"
pub fn format_ja(phase: MoonPhase, days_until: i64) -> String {
    let name = phase.name_ja();
    if days_until <= 0 {
        format!("今夜は{name}")
    } else {
        format!("{}日後に{name}", koyomi::kanji_number(days_until as u32))
    }
}

/// 例: "New Moon in 3 days", "First Quarter in 12 days",
///     "Tonight: Full Moon", "Tonight: Last Quarter"
pub fn format_en(phase: MoonPhase, days_until: i64) -> String {
    let name = phase.name_en();
    match days_until {
        d if d <= 0 => format!("Tonight: {name}"),
        1 => format!("{name} tomorrow"),
        d => format!("{name} in {d} days"),
    }
}

/// `today` から見て次に来る朔望を日本語メッセージにする。
///
/// `today` には通常 `now.date_naive()` (main.rs で使われている暦日) を渡す。
pub fn describe_ja(after: DateTime<Local>, today: NaiveDate) -> String {
    let ev = next_phase(after);
    let days = (ev.datetime.date_naive() - today).num_days();
    format_ja(ev.phase, days)
}

/// `today` から見て次に来る朔望を英語メッセージにする。
pub fn describe_en(after: DateTime<Local>, today: NaiveDate) -> String {
    let ev = next_phase(after);
    let days = (ev.datetime.date_naive() - today).num_days();
    format_en(ev.phase, days)
}

// ─────────────────────────────────────────────
// テスト
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn local(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    // ── 整形ロジック (天文計算に依存しない純粋関数) ──────────────

    #[test]
    fn test_format_ja() {
        assert_eq!(format_ja(MoonPhase::New, 3), "三日後に朔");
        assert_eq!(format_ja(MoonPhase::FirstQuarter, 12), "十二日後に上弦");
        assert_eq!(format_ja(MoonPhase::Full, 0), "今夜は望");
        assert_eq!(format_ja(MoonPhase::LastQuarter, 0), "今夜は下弦");
        // 既に過ぎている(負)場合も「今夜は」扱いにする
        assert_eq!(format_ja(MoonPhase::New, -1), "今夜は朔");
    }

    #[test]
    fn test_format_en() {
        assert_eq!(format_en(MoonPhase::New, 3), "New Moon in 3 days");
        assert_eq!(
            format_en(MoonPhase::FirstQuarter, 12),
            "First Quarter in 12 days"
        );
        assert_eq!(format_en(MoonPhase::Full, 0), "Tonight: Full Moon");
        assert_eq!(format_en(MoonPhase::LastQuarter, 0), "Tonight: Last Quarter");
        assert_eq!(format_en(MoonPhase::New, 1), "New Moon tomorrow");
    }

    // ── 朔望計算の自己無矛盾性チェック ──────────────────────────
    //
    // 絶対時刻の精密な参照値には依存せず、
    // 「朔→上弦→望→下弦の順で周期的に出現するか」
    // 「朔から次の朔までの間隔が朔望月(≈29.53日)に近いか」を検証する。

    #[test]
    fn test_phase_sequence_and_spacing() {
        let mut t = local(2026, 1, 1, 0, 0);
        let mut phases = Vec::new();
        for _ in 0..12 {
            let ev = next_phase(t);
            phases.push((ev.phase, ev.datetime));
            t = ev.datetime + Duration::minutes(10);
        }

        // 探索開始時点でどの朔望に当たるかは基準日に依存するため、
        // 最初に見つかった朔望を起点として、以降 朔→上弦→望→下弦→朔... の
        // 順で正しく循環しているかどうかを検証する。
        let cycle = [
            MoonPhase::New,
            MoonPhase::FirstQuarter,
            MoonPhase::Full,
            MoonPhase::LastQuarter,
        ];
        let start_idx = cycle
            .iter()
            .position(|p| *p == phases[0].0)
            .expect("最初の朔望が周期に含まれていない");
        for (i, (phase, _)) in phases.iter().enumerate() {
            let expected = cycle[(start_idx + i) % 4];
            assert_eq!(*phase, expected, "順序が異なる (index {i})");
        }

        let new_moons: Vec<DateTime<Local>> = phases
            .iter()
            .filter(|(p, _)| *p == MoonPhase::New)
            .map(|(_, d)| *d)
            .collect();
        for pair in new_moons.windows(2) {
            let gap_days = (pair[1] - pair[0]).num_minutes() as f64 / 1440.0;
            assert!(
                (gap_days - 29.530_59).abs() < 0.5,
                "朔望月の間隔が不自然 (gap = {gap_days} 日)"
            );
        }
    }

    // 探索の起点そのものが朔望の瞬間である場合、その瞬間自身を返すこと
    // (「今夜は◯◯」の判定に必要な境界条件)。
    #[test]
    fn test_next_phase_includes_boundary() {
        let ev = next_phase(local(2026, 1, 1, 0, 0));
        let ev2 = next_phase(ev.datetime);
        assert_eq!(ev.phase, ev2.phase);
        assert_eq!(ev.datetime, ev2.datetime);
    }

    // ── メッセージ生成 (describe_ja / describe_en) の日数計算 ──────

    #[test]
    fn test_describe_ja_today_boundary() {
        let ev = next_phase(local(2026, 1, 1, 0, 0));
        let today = ev.datetime.date_naive();
        // 朔望当日を基準日とした場合は「今夜は」になること
        let msg = describe_ja(local(2026, 1, 1, 0, 0), today);
        assert!(msg.starts_with("今夜は"), "msg = {msg}");
    }
}
