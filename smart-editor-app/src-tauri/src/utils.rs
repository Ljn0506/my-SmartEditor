use crate::error::{AppError, Result};

pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(AppError::Validation("路径不能为空".to_string()));
    }
    let p = std::path::Path::new(path);
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::Validation("非法文件路径".to_string()));
    }
    Ok(())
}

/// 验证路径是否在允许的根目录范围内
/// 用于防止路径遍历攻击超出预期的访问边界
pub fn validate_path_within(path: &str, allowed_root: &str) -> Result<()> {
    validate_path(path)?;
    let allowed = std::path::Path::new(allowed_root);
    let target = std::path::Path::new(path);

    // 统一转为绝对路径
    let allowed_abs = if allowed.is_absolute() {
        allowed.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(allowed)
    };
    let target_abs = if target.is_absolute() {
        target.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(target)
    };

    // 清理 . 和 .. 组件（不跟随符号链接，避免平台差异）
    let allowed_clean: std::path::PathBuf = allowed_abs.components().collect();
    let target_clean: std::path::PathBuf = target_abs.components().collect();

    if !target_clean.starts_with(&allowed_clean) {
        return Err(AppError::Validation(
            "路径不在允许的根目录范围内".to_string(),
        ));
    }
    Ok(())
}

/// 检查 IP 是否为私有/本地/内网地址
pub fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_link_local()
                || v4.is_private()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.octets() == [0, 0, 0, 0]
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unicast_link_local()
                || v6.is_unique_local()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_empty() {
        assert!(validate_path("").is_err());
    }

    #[test]
    fn test_validate_path_parent_dir() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("foo/../../bar").is_err());
        assert!(validate_path("/tmp/../etc").is_err());
    }

    #[test]
    fn test_validate_path_current_dir_ok() {
        assert!(validate_path("./foo.txt").is_ok());
        assert!(validate_path("docs/./readme.md").is_ok());
    }

    #[test]
    fn test_validate_path_normal() {
        assert!(validate_path("docs/readme.md").is_ok());
        assert!(validate_path("/absolute/path/file.txt").is_ok());
        assert!(validate_path("templates/等保方案.docx").is_ok());
    }

    #[test]
    fn test_validate_path_within_ok() {
        let tmp_dir = std::env::temp_dir().join("smart_editor_validate_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();
        std::fs::create_dir_all(tmp_dir.join("subdir")).unwrap();
        std::fs::write(tmp_dir.join("subdir/file.txt"), "hello").unwrap();

        let root = tmp_dir.to_str().unwrap();
        assert!(validate_path_within(&format!("{}/subdir/file.txt", root), root).is_ok());
        assert!(validate_path_within(&format!("{}/file.txt", root), root).is_ok());

        std::fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    fn test_validate_path_within_outside() {
        let tmp_dir = std::env::temp_dir().join("smart_editor_validate_test2");
        let outside_dir = std::env::temp_dir().join("smart_editor_validate_outside");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let _ = std::fs::remove_dir_all(&outside_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();
        std::fs::create_dir_all(&outside_dir).unwrap();
        std::fs::write(outside_dir.join("secret.txt"), "secret").unwrap();

        let root = tmp_dir.to_str().unwrap();
        let outside_file = outside_dir.join("secret.txt").to_string_lossy().to_string();
        assert!(validate_path_within(&outside_file, root).is_err());

        std::fs::remove_dir_all(&tmp_dir).unwrap();
        std::fs::remove_dir_all(&outside_dir).unwrap();
    }

    #[test]
    fn test_validate_path_within_parent_dir_blocked() {
        let tmp_dir = std::env::temp_dir().join("smart_editor_validate_test3");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let root = tmp_dir.to_str().unwrap();
        // ParentDir 应先被 validate_path 拦截
        assert!(validate_path_within(&format!("{}/../secret.txt", root), root).is_err());

        std::fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    fn test_is_private_ip_v4_loopback() {
        assert!(is_private_ip("127.0.0.1".parse().unwrap()));
        assert!(is_private_ip("127.255.255.255".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v4_private_ranges() {
        assert!(is_private_ip("10.0.0.0".parse().unwrap()));
        assert!(is_private_ip("10.255.255.255".parse().unwrap()));
        assert!(is_private_ip("172.16.0.0".parse().unwrap()));
        assert!(is_private_ip("172.31.255.255".parse().unwrap()));
        assert!(is_private_ip("192.168.0.0".parse().unwrap()));
        assert!(is_private_ip("192.168.255.255".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v4_link_local() {
        assert!(is_private_ip("169.254.0.0".parse().unwrap()));
        assert!(is_private_ip("169.254.255.255".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v4_public() {
        assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip("1.1.1.1".parse().unwrap()));
        assert!(!is_private_ip("223.255.255.255".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v6_loopback() {
        assert!(is_private_ip("::1".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v6_link_local() {
        assert!(is_private_ip("fe80::1".parse().unwrap()));
        assert!(is_private_ip("fe80::1234:56ff:fe78:9abc".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v6_unique_local() {
        assert!(is_private_ip("fc00::1".parse().unwrap()));
        assert!(is_private_ip("fd00::1".parse().unwrap()));
        assert!(is_private_ip("fdff:ffff::1".parse().unwrap()));
    }

    #[test]
    fn test_is_private_ip_v6_public() {
        assert!(!is_private_ip("2001:4860:4860::8888".parse().unwrap()));
        assert!(!is_private_ip("2606:4700:4700::1111".parse().unwrap()));
    }
}
