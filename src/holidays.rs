//! 祝日データの読み込み。
//!
//! `data/cal-YYYY.csv` (フォーマット: `name,month,day`、ヘッダ無し) は
//! build.rs によってビルド時にパース・検証され、`HOLIDAYS` という静的配列
//! としてコード生成される (`$OUT_DIR/holidays_data.rs`)。
//! 不正な月日や重複した日付などは、その時点でビルドエラーとして検出される。
//!
//! `name` が空文字列の場合は「名称のない休日」(振替休日など) を表す。
//!
//! 新しい年のデータを追加する場合は `data/cal-YYYY.csv` を作成するだけでよい
//! (build.rs がディレクトリを自動走査するため、build.rs 自体の変更は不要)。

use chrono::{Datelike, NaiveDate};

include!(concat!(env!("OUT_DIR"), "/holidays_data.rs"));

/// 祝日の種別。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holiday {
    /// 名称のある祝日 (例: "海の日")
    Named(String),
    /// 名称のない休日 (振替休日など)
    Unnamed,
}

/// 指定日が祝日であればその情報を返す。
pub fn lookup(date: NaiveDate) -> Option<Holiday> {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    HOLIDAYS
        .iter()
        .find(|&&(y, _, m, d)| y == year && m == month && d == day)
        .map(|&(_, name, _, _)| {
            if name.is_empty() {
                Holiday::Unnamed
            } else {
                Holiday::Named(name.to_string())
            }
        })
}

// ─── テスト ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn test_named_holiday() {
        assert_eq!(
            lookup(d(2026, 7, 20)),
            Some(Holiday::Named("海の日".to_string()))
        );
        assert_eq!(
            lookup(d(2027, 8, 11)),
            Some(Holiday::Named("山の日".to_string()))
        );
    }

    #[test]
    fn test_unnamed_holiday() {
        assert_eq!(lookup(d(2026, 5, 6)), Some(Holiday::Unnamed));
        assert_eq!(lookup(d(2026, 9, 22)), Some(Holiday::Unnamed));
        assert_eq!(lookup(d(2027, 3, 22)), Some(Holiday::Unnamed));
    }

    #[test]
    fn test_not_a_holiday() {
        assert_eq!(lookup(d(2026, 7, 9)), None);
    }

    #[test]
    fn test_unknown_year() {
        assert_eq!(lookup(d(2030, 1, 1)), None);
    }
}
