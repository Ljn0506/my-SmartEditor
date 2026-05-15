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
