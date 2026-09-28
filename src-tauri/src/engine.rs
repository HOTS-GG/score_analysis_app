use std::path::{Path, PathBuf};

fn validate_absolute_path(path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    if path.trim().is_empty() || !p.is_absolute() {
        return Err("유효하지 않은 경로입니다.".to_string());
    }
    Ok(p.to_path_buf())
}

pub fn validate_existing_path(path: &str, not_found_message: &str) -> Result<(), String> {
    let p = validate_absolute_path(path)?;
    p.canonicalize().map_err(|_| not_found_message.to_string())?;
    Ok(())
}

pub fn validate_parent_dir_path(path: &str, missing_parent_message: &str) -> Result<(), String> {
    let p = validate_absolute_path(path)?;
    p.parent()
        .ok_or_else(|| "유효하지 않은 경로입니다.".to_string())?
        .canonicalize()
        .map_err(|_| missing_parent_message.to_string())?;
    Ok(())
}

/// 셀 문자열을 실수로 변환. 빈 문자열, '-', '결시', '미응시' 등은 None (0으로 치환 금지).
pub fn parse_f64(s: &str) -> Option<f64> {
    let t = s.trim().replace(',', "");
    if t.is_empty() {
        return None;
    }
    match t.as_str() {
        "-" | "—" | "결시" | "미응시" | "결" | "N/A" | "NA" | "null" | "NULL" => None,
        _ => t.trim_end_matches('%').trim().parse::<f64>().ok(),
    }
}

pub fn parse_i64(s: &str) -> Option<i64> {
    parse_f64(s).map(|v| v.round() as i64)
}

/// 성취도 문자 정규화 (a → A, "A(90)" → A)
pub fn parse_achievement(s: &str) -> Option<String> {
    let t = s.trim().to_uppercase();
    let c = t.chars().next()?;
    if ('A'..='E').contains(&c) {
        Some(c.to_string())
    } else {
        None
    }
}
