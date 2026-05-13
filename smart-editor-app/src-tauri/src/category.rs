//! 文档分类推断公共模块（v1.0 — 路径推断优先）

use crate::models::{BusinessDomain, ContentModule, DocAttr, ProjectPhase, SecurityLayer};

/// 根据文本内容推断偏离检查项的分类（商务 / 技术）
pub fn infer_category(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("资质")
        || lower.contains("报价")
        || lower.contains("付款")
        || lower.contains("保证金")
        || lower.contains("工期")
        || lower.contains("合同")
    {
        "商务".to_string()
    } else {
        "技术".to_string()
    }
}

/// 去除目录名中的数字前缀，如 "01-方案阶段" → "方案阶段"
fn strip_number_prefix(s: &str) -> String {
    if let Some(pos) = s.find('-') {
        let prefix = &s[..pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) {
            return s[pos + 1..].to_string();
        }
    }
    s.to_string()
}

/// 从文件路径解析各级目录名（去除数字前缀）
fn path_components(file_path: &str) -> Vec<String> {
    std::path::Path::new(file_path)
        .ancestors()
        .filter_map(|p| p.file_name())
        .filter_map(|s| s.to_str())
        .map(strip_number_prefix)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// 路径推断：一级目录 → ProjectPhase
fn infer_project_phase_from_path(components: &[String]) -> Option<ProjectPhase> {
    components.get(1).and_then(|dir| match dir.as_str() {
        "方案阶段" => Some(ProjectPhase::Proposal),
        "投标阶段" => Some(ProjectPhase::Bidding),
        "合同阶段" => Some(ProjectPhase::Contract),
        _ => None,
    })
}

/// 路径推断：二级目录 → BusinessDomain
fn infer_business_domain_from_path(components: &[String]) -> Option<BusinessDomain> {
    components.get(2).and_then(|dir| match dir.as_str() {
        "网络安全" => Some(BusinessDomain::NetworkSecurity),
        "应用安全" => Some(BusinessDomain::ApplicationSecurity),
        "数据安全" => Some(BusinessDomain::DataSecurity),
        "安全运营" => Some(BusinessDomain::SecurityOperation),
        "安全管理" => Some(BusinessDomain::SecurityManagement),
        _ => None,
    })
}

/// 路径推断：三级目录 → ContentModule
fn infer_content_module_from_path(components: &[String]) -> Option<ContentModule> {
    components.get(3).and_then(|dir| match dir.as_str() {
        "技术方案" => Some(ContentModule::TechnicalProposal),
        "商务条款" => Some(ContentModule::BusinessTerms),
        "实施计划" => Some(ContentModule::ImplementationPlan),
        "偏离说明" => Some(ContentModule::DeviationExplanation),
        "资质证明" => Some(ContentModule::QualificationProof),
        "投标应答" => Some(ContentModule::BidResponse),
        _ => None,
    })
}

/// 文件名前缀推断：【xxx】→ DocAttr
fn infer_doc_attr_from_prefix(file_name: &str) -> Option<DocAttr> {
    if let Some(after_open) = file_name.split('【').nth(1) {
        if let Some(prefix) = after_open.split('】').next() {
            return match prefix {
                "投标应答" => Some(DocAttr::BidResponse),
                "技术方案" => Some(DocAttr::TechnicalProposal),
                "实施方案" => Some(DocAttr::ImplementationPlan),
                "合同协议" => Some(DocAttr::ContractAgreement),
                "汇报材料" => Some(DocAttr::ReportMaterial),
                _ => None,
            };
        }
    }
    None
}

/// 文件名关键词推断（fallback）
fn infer_doc_attr_from_keywords(file_name: &str) -> Option<DocAttr> {
    let lower = file_name.to_lowercase();
    if lower.contains("投标") || lower.contains("应答") {
        Some(DocAttr::BidResponse)
    } else if lower.contains("技术方案") || lower.contains("设计方案") {
        Some(DocAttr::TechnicalProposal)
    } else if lower.contains("实施") {
        Some(DocAttr::ImplementationPlan)
    } else if lower.contains("合同") || lower.contains("协议") {
        Some(DocAttr::ContractAgreement)
    } else {
        None
    }
}

/// 安全层级标注：解析 [检测] / [防御] / [分析] / [治理]
fn infer_security_layer(file_name: &str) -> Option<SecurityLayer> {
    if file_name.contains("[检测]") {
        Some(SecurityLayer::Detection)
    } else if file_name.contains("[防御]") {
        Some(SecurityLayer::Defense)
    } else if file_name.contains("[分析]") {
        Some(SecurityLayer::Analysis)
    } else if file_name.contains("[治理]") {
        Some(SecurityLayer::Governance)
    } else {
        None
    }
}

/// 根据文件路径和文件名推断文档的多维分类属性
/// v1.0 规范：路径推断（优先级最高）→ 文件名前缀推断 → 文件名关键词推断
#[allow(clippy::type_complexity)]
pub fn infer_categories(
    file_path: &str,
    file_name: &str,
    _text: &str,
) -> (
    Option<DocAttr>,
    Option<BusinessDomain>,
    Option<ContentModule>,
    Option<ProjectPhase>,
    Option<SecurityLayer>,
) {
    let components = path_components(file_path);

    // 1. 路径推断（优先级最高）
    let project_phase = infer_project_phase_from_path(&components);
    let business_domain = infer_business_domain_from_path(&components);
    let content_module = infer_content_module_from_path(&components);

    // 2. 文件名前缀推断（fallback for DocAttr）
    let doc_attr = infer_doc_attr_from_prefix(file_name)
        .or_else(|| infer_doc_attr_from_keywords(file_name));

    // 3. 安全层级标注
    let security_layer = infer_security_layer(file_name);

    // DocAttr：根据 ContentModule 做二次 fallback
    let doc_attr = doc_attr.or(match content_module {
        Some(ContentModule::TechnicalProposal) => Some(DocAttr::TechnicalProposal),
        Some(ContentModule::ImplementationPlan) => Some(DocAttr::ImplementationPlan),
        Some(ContentModule::BidResponse) => Some(DocAttr::BidResponse),
        _ => None,
    });

    // 最终 fallback：ReportMaterial
    let doc_attr = doc_attr.or(Some(DocAttr::ReportMaterial));

    // 路径无法推断时，fallback 到文件名关键词推断
    let business_domain = business_domain.or_else(|| {
        let lower = file_name.to_lowercase();
        if lower.contains("等保") || lower.contains("等级保护") {
            Some(BusinessDomain::NetworkSecurity)
        } else if lower.contains("渗透") || lower.contains("漏洞") || lower.contains("代码审计") {
            Some(BusinessDomain::ApplicationSecurity)
        } else if lower.contains("数据") || lower.contains("数据库") || lower.contains("隐私") {
            Some(BusinessDomain::DataSecurity)
        } else if lower.contains("soc") || lower.contains("运营") || lower.contains("态势") {
            Some(BusinessDomain::SecurityOperation)
        } else if lower.contains("管理") || lower.contains("制度") || lower.contains("体系") {
            Some(BusinessDomain::SecurityManagement)
        } else {
            Some(BusinessDomain::NetworkSecurity)
        }
    });

    let content_module = content_module.or_else(|| {
        let lower = file_name.to_lowercase();
        if lower.contains("偏离") || lower.contains("差异") {
            Some(ContentModule::DeviationExplanation)
        } else if lower.contains("资质") || lower.contains("证书") || lower.contains("业绩") {
            Some(ContentModule::QualificationProof)
        } else if lower.contains("案例") || lower.contains("项目经历") {
            Some(ContentModule::CaseIntroduction)
        } else if lower.contains("实施") || lower.contains("计划") || lower.contains("进度") {
            Some(ContentModule::ImplementationPlan)
        } else if lower.contains("商务") || lower.contains("报价") || lower.contains("合同") {
            Some(ContentModule::BusinessTerms)
        } else {
            Some(ContentModule::TechnicalProposal)
        }
    });

    (
        doc_attr,
        business_domain,
        content_module,
        project_phase,
        security_layer,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BusinessDomain, ContentModule, DocAttr, ProjectPhase, SecurityLayer};

    // ========== 路径推断测试 ==========

    #[test]
    fn test_path_inference_proposal_network_technical() {
        let path = "/data/01-方案阶段/网络安全/技术方案/等保2.0方案.docx";
        let (attr, domain, module, phase, layer) = infer_categories(path, "等保2.0方案.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Proposal));
        assert_eq!(domain, Some(BusinessDomain::NetworkSecurity));
        assert_eq!(module, Some(ContentModule::TechnicalProposal));
        assert_eq!(attr, Some(DocAttr::TechnicalProposal));
        assert_eq!(layer, None);
    }

    #[test]
    fn test_path_inference_bidding_app_sec_bid_response() {
        let path = "/data/02-投标阶段/应用安全/投标应答/【投标应答】渗透测试.docx";
        let (attr, domain, module, phase, _layer) = infer_categories(path, "【投标应答】渗透测试.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::ApplicationSecurity));
        assert_eq!(module, Some(ContentModule::BidResponse));
        assert_eq!(attr, Some(DocAttr::BidResponse));
    }

    #[test]
    fn test_path_inference_contract_data_qualification() {
        let path = "/data/03-合同阶段/数据安全/资质证明/资质证书.pdf";
        let (attr, domain, module, phase, layer) = infer_categories(path, "资质证书.pdf", "");
        assert_eq!(phase, Some(ProjectPhase::Contract));
        assert_eq!(domain, Some(BusinessDomain::DataSecurity));
        assert_eq!(module, Some(ContentModule::QualificationProof));
    }

    #[test]
    fn test_path_inference_operation_implementation() {
        let path = "/data/投标阶段/安全运营/实施计划/实施进度.xlsx";
        let (attr, domain, module, phase, layer) = infer_categories(path, "实施进度.xlsx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::SecurityOperation));
        assert_eq!(module, Some(ContentModule::ImplementationPlan));
    }

    #[test]
    fn test_path_inference_management_business_terms() {
        let path = "/data/合同阶段/安全管理/商务条款/合同条款.docx";
        let (_attr, domain, module, phase, _layer) = infer_categories(path, "合同条款.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Contract));
        assert_eq!(domain, Some(BusinessDomain::SecurityManagement));
        assert_eq!(module, Some(ContentModule::BusinessTerms));
    }

    // ========== 文件名前缀推断测试 ==========

    #[test]
    fn test_filename_prefix_bid_response() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【投标应答】网络安全方案.docx", "");
        assert_eq!(attr, Some(DocAttr::BidResponse));
    }

    #[test]
    fn test_filename_prefix_technical_proposal() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【技术方案】等保三级设计.docx", "");
        assert_eq!(attr, Some(DocAttr::TechnicalProposal));
    }

    #[test]
    fn test_filename_prefix_contract_agreement() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【合同协议】服务合同.pdf", "");
        assert_eq!(attr, Some(DocAttr::ContractAgreement));
    }

    // ========== fallback 测试（路径无法推断时使用文件名） ==========

    #[test]
    fn test_fallback_when_no_path_match() {
        let path = "/data/文件.docx";
        let (attr, domain, module, phase, layer) =
            infer_categories(path, "偏离表-资质证书.xlsx", "");
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
        assert_eq!(domain, Some(BusinessDomain::NetworkSecurity));
        assert_eq!(module, Some(ContentModule::DeviationExplanation));
        assert_eq!(phase, None);
        assert_eq!(layer, None);
    }

    #[test]
    fn test_fallback_filename_keywords() {
        let path = "/tmp/unknown.docx";
        let (attr, domain, module, phase, _) =
            infer_categories(path, "等保2.0投标应答技术方案.docx", "");
        assert_eq!(attr, Some(DocAttr::BidResponse));
        assert_eq!(domain, Some(BusinessDomain::NetworkSecurity));
        assert_eq!(module, Some(ContentModule::TechnicalProposal));
        assert_eq!(phase, None);
    }

    // ========== 安全层级标注测试 ==========

    #[test]
    fn test_security_layer_detection() {
        let (_, _, _, _, layer) = infer_categories("", "[检测]漏洞扫描报告.docx", "");
        assert_eq!(layer, Some(SecurityLayer::Detection));
    }

    #[test]
    fn test_security_layer_defense() {
        let (_, _, _, _, layer) = infer_categories("", "[防御]WAF部署方案.docx", "");
        assert_eq!(layer, Some(SecurityLayer::Defense));
    }

    #[test]
    fn test_security_layer_analysis() {
        let (_, _, _, _, layer) = infer_categories("", "[分析]日志审计分析.docx", "");
        assert_eq!(layer, Some(SecurityLayer::Analysis));
    }

    #[test]
    fn test_security_layer_governance() {
        let (_, _, _, _, layer) = infer_categories("", "[治理]数据治理方案.docx", "");
        assert_eq!(layer, Some(SecurityLayer::Governance));
    }

    #[test]
    fn test_security_layer_none() {
        let (_, _, _, _, layer) = infer_categories("", "普通文档.docx", "");
        assert_eq!(layer, None);
    }

    // ========== 路径 + 安全层级组合测试 ==========

    #[test]
    fn test_path_and_security_layer_combined() {
        let path = "/data/02-投标阶段/网络安全/技术方案/[检测]漏洞扫描方案.docx";
        let (_attr, domain, module, phase, layer) = infer_categories(path, "[检测]漏洞扫描方案.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::NetworkSecurity));
        assert_eq!(module, Some(ContentModule::TechnicalProposal));
        assert_eq!(layer, Some(SecurityLayer::Detection));
    }
}
