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

/// 在路径组件中找到 NAS 分类根目录（一级目录）的位置
/// 一级目录特征：名称包含 "方案阶段" / "投标阶段" / "合同阶段" / "通用素材"
fn find_phase_root(components: &[String]) -> Option<usize> {
    for (i, comp) in components.iter().enumerate() {
        match comp.as_str() {
            "方案阶段" | "投标阶段" | "合同阶段" | "通用素材" => return Some(i),
            _ => {}
        }
    }
    None
}

fn map_dir_to_module(dir: &str) -> Option<ContentModule> {
    match dir {
        "技术方案" => Some(ContentModule::TechnicalProposal),
        "商务条款" | "商务模板" => Some(ContentModule::BusinessTerms),
        "实施计划" => Some(ContentModule::ImplementationPlan),
        "偏离说明" => Some(ContentModule::DeviationExplanation),
        "资质证明" => Some(ContentModule::QualificationProof),
        "投标应答" => Some(ContentModule::BidResponse),
        "案例介绍" => Some(ContentModule::CaseIntroduction),
        "产品资料" => Some(ContentModule::ProductMaterial),
        _ => None,
    }
}

/// 路径推断：一级目录 → ProjectPhase
fn infer_project_phase_from_path(components: &[String], root_idx: Option<usize>) -> Option<ProjectPhase> {
    root_idx.and_then(|idx| {
        match components.get(idx)?.as_str() {
            "方案阶段" => Some(ProjectPhase::Proposal),
            "投标阶段" => Some(ProjectPhase::Bidding),
            "合同阶段" => Some(ProjectPhase::Contract),
            "通用素材" => None,
            _ => None,
        }
    })
}

/// 路径推断：二级目录 → BusinessDomain
fn infer_business_domain_from_path(components: &[String], root_idx: Option<usize>) -> Option<BusinessDomain> {
    let idx = root_idx?;
    if components.get(idx)? == "通用素材" {
        return None;
    }
    components.get(idx + 1).and_then(|dir| match dir.as_str() {
        "网络安全" => Some(BusinessDomain::NetworkSecurity),
        "应用安全" => Some(BusinessDomain::ApplicationSecurity),
        "数据安全" => Some(BusinessDomain::DataSecurity),
        "安全运营" => Some(BusinessDomain::SecurityOperation),
        "安全管理" => Some(BusinessDomain::SecurityManagement),
        _ => None,
    })
}

/// 路径推断：三级目录（标准结构）或二级目录（通用素材）→ ContentModule
fn infer_content_module_from_path(components: &[String], root_idx: Option<usize>) -> Option<ContentModule> {
    let idx = root_idx?;
    let offset = if components.get(idx)? == "通用素材" { 1 } else { 2 };
    components.get(idx + offset).and_then(|dir| map_dir_to_module(dir))
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
fn infer_doc_attr_from_keywords(lower: &str) -> Option<DocAttr> {
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
const DOMAIN_KEYWORDS: &[(&[&str], BusinessDomain)] = &[
    (&["等保", "等级保护"], BusinessDomain::NetworkSecurity),
    (&["渗透", "漏洞", "代码审计"], BusinessDomain::ApplicationSecurity),
    (&["数据", "数据库", "隐私"], BusinessDomain::DataSecurity),
    (&["soc", "运营", "态势"], BusinessDomain::SecurityOperation),
    (&["管理", "制度", "体系"], BusinessDomain::SecurityManagement),
];

const MODULE_KEYWORDS: &[(&[&str], ContentModule)] = &[
    (&["偏离", "差异"], ContentModule::DeviationExplanation),
    (&["资质", "证书", "业绩"], ContentModule::QualificationProof),
    (&["案例", "项目经历"], ContentModule::CaseIntroduction),
    (&["实施", "计划", "进度"], ContentModule::ImplementationPlan),
    (&["商务", "报价", "合同"], ContentModule::BusinessTerms),
];

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
    let root_idx = find_phase_root(&components);
    let file_name_lower = file_name.to_lowercase();
    let is_generic = root_idx
        .and_then(|idx| components.get(idx))
        .map(|c| c == "通用素材")
        .unwrap_or(false);

    // 1. 路径推断（优先级最高）
    let project_phase = infer_project_phase_from_path(&components, root_idx);
    let business_domain = infer_business_domain_from_path(&components, root_idx);
    let content_module = infer_content_module_from_path(&components, root_idx);

    // 2. 文件名前缀推断（fallback for DocAttr）
    let doc_attr = infer_doc_attr_from_prefix(file_name)
        .or_else(|| infer_doc_attr_from_keywords(&file_name_lower));

    // 3. 安全层级标注
    let security_layer = infer_security_layer(file_name);

    // DocAttr：根据 ContentModule 做二次 fallback
    let doc_attr = doc_attr.or(match content_module {
        Some(ContentModule::TechnicalProposal) => Some(DocAttr::TechnicalProposal),
        Some(ContentModule::ImplementationPlan) => Some(DocAttr::ImplementationPlan),
        Some(ContentModule::BidResponse) => Some(DocAttr::BidResponse),
        Some(ContentModule::ProductMaterial) => Some(DocAttr::TechnicalProposal),
        _ => None,
    });

    // 最终 fallback：ReportMaterial
    let doc_attr = doc_attr.or(Some(DocAttr::ReportMaterial));

    // 路径无法推断时，fallback 到文件名关键词推断（通用素材不 fallback 业务领域）
    let business_domain = if is_generic {
        business_domain
    } else {
        business_domain.or_else(|| {
            DOMAIN_KEYWORDS
                .iter()
                .find(|(kws, _)| kws.iter().any(|kw| file_name_lower.contains(kw)))
                .map(|(_, domain)| *domain)
                .or(Some(BusinessDomain::NetworkSecurity))
        })
    };

    let content_module = content_module.or_else(|| {
        MODULE_KEYWORDS
            .iter()
            .find(|(kws, _)| kws.iter().any(|kw| file_name_lower.contains(kw)))
            .map(|(_, module)| *module)
            .or(Some(ContentModule::TechnicalProposal))
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
    use crate::models::{
        BusinessDomain, ContentModule, DocAttr, ProjectPhase, SecurityLayer,
    };

    // ========== 路径推断测试（标准三级目录）==========

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
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "【投标应答】渗透测试.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::ApplicationSecurity));
        assert_eq!(module, Some(ContentModule::BidResponse));
        assert_eq!(attr, Some(DocAttr::BidResponse));
    }

    #[test]
    fn test_path_inference_contract_data_qualification() {
        let path = "/data/03-合同阶段/数据安全/资质证明/资质证书.pdf";
        let (_attr, domain, module, phase, _layer) =
            infer_categories(path, "资质证书.pdf", "");
        assert_eq!(phase, Some(ProjectPhase::Contract));
        assert_eq!(domain, Some(BusinessDomain::DataSecurity));
        assert_eq!(module, Some(ContentModule::QualificationProof));
    }

    #[test]
    fn test_path_inference_operation_implementation() {
        let path = "/data/投标阶段/安全运营/实施计划/实施进度.xlsx";
        let (_attr, domain, module, phase, _layer) =
            infer_categories(path, "实施进度.xlsx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::SecurityOperation));
        assert_eq!(module, Some(ContentModule::ImplementationPlan));
    }

    #[test]
    fn test_path_inference_management_business_terms() {
        let path = "/data/合同阶段/安全管理/商务条款/合同条款.docx";
        let (_attr, domain, module, phase, _layer) =
            infer_categories(path, "合同条款.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Contract));
        assert_eq!(domain, Some(BusinessDomain::SecurityManagement));
        assert_eq!(module, Some(ContentModule::BusinessTerms));
    }

    // ========== 5 个业务领域 × 3 个项目阶段 全覆盖 ==========

    #[test]
    fn test_all_domain_phase_combinations() {
        let domains = [
            ("网络安全", BusinessDomain::NetworkSecurity),
            ("应用安全", BusinessDomain::ApplicationSecurity),
            ("数据安全", BusinessDomain::DataSecurity),
            ("安全运营", BusinessDomain::SecurityOperation),
            ("安全管理", BusinessDomain::SecurityManagement),
        ];
        let phases = [
            ("01-方案阶段", ProjectPhase::Proposal),
            ("02-投标阶段", ProjectPhase::Bidding),
            ("03-合同阶段", ProjectPhase::Contract),
        ];

        for (phase_dir, expected_phase) in phases {
            for (domain_dir, expected_domain) in domains {
                let path = format!("/NAS/{}/{}/技术方案/测试文件.docx", phase_dir, domain_dir);
                let (_attr, domain, module, phase, _layer) =
                    infer_categories(&path, "测试文件.docx", "");
                assert_eq!(
                    phase,
                    Some(expected_phase),
                    "路径 {} 的阶段推断错误",
                    path
                );
                assert_eq!(
                    domain,
                    Some(expected_domain),
                    "路径 {} 的领域推断错误",
                    path
                );
                assert_eq!(module, Some(ContentModule::TechnicalProposal));
            }
        }
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
    fn test_filename_prefix_implementation_plan() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【实施方案】部署计划.docx", "");
        assert_eq!(attr, Some(DocAttr::ImplementationPlan));
    }

    #[test]
    fn test_filename_prefix_contract_agreement() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【合同协议】服务合同.pdf", "");
        assert_eq!(attr, Some(DocAttr::ContractAgreement));
    }

    #[test]
    fn test_filename_prefix_report_material() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "【汇报材料】项目汇报.pptx", "");
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
    }

    #[test]
    fn test_filename_prefix_no_prefix_fallback() {
        let path = "/data/文件.docx";
        let (attr, _, _, _, _) = infer_categories(path, "普通文档.docx", "");
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
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
        let (_attr, domain, module, phase, layer) =
            infer_categories(path, "[检测]漏洞扫描方案.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::NetworkSecurity));
        assert_eq!(module, Some(ContentModule::TechnicalProposal));
        assert_eq!(layer, Some(SecurityLayer::Detection));
    }

    // ========== 通用素材测试（2.4）==========

    #[test]
    fn test_generic_material_qualification() {
        let path = "/data/00-通用素材/资质证明/公司资质.pdf";
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "公司资质.pdf", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::QualificationProof));
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
    }

    #[test]
    fn test_generic_material_case() {
        let path = "/data/00-通用素材/案例介绍/成功案例.docx";
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "成功案例.docx", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::CaseIntroduction));
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
    }

    #[test]
    fn test_generic_material_product() {
        let path = "/data/00-通用素材/产品资料/产品介绍.docx";
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "产品介绍.docx", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::ProductMaterial));
        assert_eq!(attr, Some(DocAttr::TechnicalProposal));
    }

    #[test]
    fn test_generic_material_template() {
        let path = "/data/00-通用素材/商务模板/报价模板.xlsx";
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "报价模板.xlsx", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::BusinessTerms));
        assert_eq!(attr, Some(DocAttr::ReportMaterial));
    }

    #[test]
    fn test_generic_material_with_prefix() {
        let path = "/data/00-通用素材/资质证明/【汇报材料】资质汇总.docx";
        let (attr, domain, module, phase, _layer) =
            infer_categories(path, "【汇报材料】资质汇总.docx", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::QualificationProof));
        assert_eq!(attr, Some(DocAttr::ReportMaterial)); // 前缀【汇报材料】命中
    }

    // ========== 任意深度路径测试（NAS 根目录不固定）==========

    #[test]
    fn test_arbitrary_path_depth() {
        let path = "/Users/ljn/smart-editor/NAS/02-投标阶段/应用安全/偏离说明/偏离表.docx";
        let (_attr, domain, module, phase, _layer) =
            infer_categories(path, "偏离表.docx", "");
        assert_eq!(phase, Some(ProjectPhase::Bidding));
        assert_eq!(domain, Some(BusinessDomain::ApplicationSecurity));
        assert_eq!(module, Some(ContentModule::DeviationExplanation));
    }

    #[test]
    fn test_arbitrary_path_depth_generic() {
        let path = "/mnt/nas/00-通用素材/产品资料/防火墙产品.pdf";
        let (_attr, domain, module, phase, _layer) =
            infer_categories(path, "防火墙产品.pdf", "");
        assert_eq!(phase, None);
        assert_eq!(domain, None);
        assert_eq!(module, Some(ContentModule::ProductMaterial));
    }

    // ========== 基准测试数据集 + 准确率验证（2.5）==========

    type PathInferenceCase<'a> = (&'a str, &'a str, Option<ProjectPhase>, Option<BusinessDomain>, Option<ContentModule>);

    /// 基准数据集：路径推断用例（期望路径推断命中）
    const PATH_INFERENCE_DATASET: &[PathInferenceCase<'_>] = &[
        // 方案阶段 × 5 领域
        ("/NAS/01-方案阶段/网络安全/技术方案/方案.docx", "方案.docx", Some(ProjectPhase::Proposal), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::TechnicalProposal)),
        ("/NAS/01-方案阶段/应用安全/技术方案/方案.docx", "方案.docx", Some(ProjectPhase::Proposal), Some(BusinessDomain::ApplicationSecurity), Some(ContentModule::TechnicalProposal)),
        ("/NAS/01-方案阶段/数据安全/技术方案/方案.docx", "方案.docx", Some(ProjectPhase::Proposal), Some(BusinessDomain::DataSecurity), Some(ContentModule::TechnicalProposal)),
        ("/NAS/01-方案阶段/安全运营/技术方案/方案.docx", "方案.docx", Some(ProjectPhase::Proposal), Some(BusinessDomain::SecurityOperation), Some(ContentModule::TechnicalProposal)),
        ("/NAS/01-方案阶段/安全管理/技术方案/方案.docx", "方案.docx", Some(ProjectPhase::Proposal), Some(BusinessDomain::SecurityManagement), Some(ContentModule::TechnicalProposal)),
        // 投标阶段 × 5 领域
        ("/NAS/02-投标阶段/网络安全/投标应答/应答.docx", "应答.docx", Some(ProjectPhase::Bidding), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::BidResponse)),
        ("/NAS/02-投标阶段/应用安全/投标应答/应答.docx", "应答.docx", Some(ProjectPhase::Bidding), Some(BusinessDomain::ApplicationSecurity), Some(ContentModule::BidResponse)),
        ("/NAS/02-投标阶段/数据安全/投标应答/应答.docx", "应答.docx", Some(ProjectPhase::Bidding), Some(BusinessDomain::DataSecurity), Some(ContentModule::BidResponse)),
        ("/NAS/02-投标阶段/安全运营/投标应答/应答.docx", "应答.docx", Some(ProjectPhase::Bidding), Some(BusinessDomain::SecurityOperation), Some(ContentModule::BidResponse)),
        ("/NAS/02-投标阶段/安全管理/投标应答/应答.docx", "应答.docx", Some(ProjectPhase::Bidding), Some(BusinessDomain::SecurityManagement), Some(ContentModule::BidResponse)),
        // 合同阶段 × 5 领域
        ("/NAS/03-合同阶段/网络安全/商务条款/条款.docx", "条款.docx", Some(ProjectPhase::Contract), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::BusinessTerms)),
        ("/NAS/03-合同阶段/应用安全/商务条款/条款.docx", "条款.docx", Some(ProjectPhase::Contract), Some(BusinessDomain::ApplicationSecurity), Some(ContentModule::BusinessTerms)),
        ("/NAS/03-合同阶段/数据安全/商务条款/条款.docx", "条款.docx", Some(ProjectPhase::Contract), Some(BusinessDomain::DataSecurity), Some(ContentModule::BusinessTerms)),
        ("/NAS/03-合同阶段/安全运营/商务条款/条款.docx", "条款.docx", Some(ProjectPhase::Contract), Some(BusinessDomain::SecurityOperation), Some(ContentModule::BusinessTerms)),
        ("/NAS/03-合同阶段/安全管理/商务条款/条款.docx", "条款.docx", Some(ProjectPhase::Contract), Some(BusinessDomain::SecurityManagement), Some(ContentModule::BusinessTerms)),
        // 通用素材 × 4 类型
        ("/NAS/00-通用素材/资质证明/资质.pdf", "资质.pdf", None, None, Some(ContentModule::QualificationProof)),
        ("/NAS/00-通用素材/案例介绍/案例.docx", "案例.docx", None, None, Some(ContentModule::CaseIntroduction)),
        ("/NAS/00-通用素材/产品资料/产品.docx", "产品.docx", None, None, Some(ContentModule::ProductMaterial)),
        ("/NAS/00-通用素材/商务模板/模板.xlsx", "模板.xlsx", None, None, Some(ContentModule::BusinessTerms)),
    ];

    #[test]
    fn test_path_inference_accuracy_benchmark() {
        let total = PATH_INFERENCE_DATASET.len();
        let mut correct = 0usize;

        for (path, file_name, expected_phase, expected_domain, expected_module) in
            PATH_INFERENCE_DATASET
        {
            let (_attr, domain, module, phase, _layer) =
                infer_categories(path, file_name, "");

            let phase_ok = phase == *expected_phase;
            let domain_ok = domain == *expected_domain;
            let module_ok = module == *expected_module;

            if phase_ok && domain_ok && module_ok {
                correct += 1;
            }
        }

        let accuracy = (correct as f64) / (total as f64) * 100.0;
        assert!(
            accuracy >= 90.0,
            "路径推断准确率 {}% 低于目标 90%（{}/{} 正确）",
            accuracy,
            correct,
            total
        );
    }

    type FilenameInferenceCase<'a> = (&'a str, Option<DocAttr>, Option<BusinessDomain>, Option<ContentModule>);

    /// 纯文件名推断基准数据集（无路径信息，期望 fallback 命中）
    const FILENAME_INFERENCE_DATASET: &[FilenameInferenceCase<'_>] = &[
        ("【投标应答】方案.docx", Some(DocAttr::BidResponse), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::TechnicalProposal)),
        ("【技术方案】设计.docx", Some(DocAttr::TechnicalProposal), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::TechnicalProposal)),
        ("【实施方案】计划.docx", Some(DocAttr::ImplementationPlan), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::ImplementationPlan)),
        ("【合同协议】合同.pdf", Some(DocAttr::ContractAgreement), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::BusinessTerms)),
        ("【汇报材料】汇报.pptx", Some(DocAttr::ReportMaterial), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::TechnicalProposal)),
        ("偏离表.xlsx", Some(DocAttr::ReportMaterial), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::DeviationExplanation)),
        ("资质证书.pdf", Some(DocAttr::ReportMaterial), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::QualificationProof)),
        ("等保方案.docx", Some(DocAttr::ReportMaterial), Some(BusinessDomain::NetworkSecurity), Some(ContentModule::TechnicalProposal)),
        ("渗透测试报告.pdf", Some(DocAttr::ReportMaterial), Some(BusinessDomain::ApplicationSecurity), Some(ContentModule::TechnicalProposal)),
        ("数据安全方案.docx", Some(DocAttr::ReportMaterial), Some(BusinessDomain::DataSecurity), Some(ContentModule::TechnicalProposal)),
    ];

    #[test]
    fn test_filename_inference_accuracy_benchmark() {
        let total = FILENAME_INFERENCE_DATASET.len();
        let mut correct = 0usize;

        for (file_name, expected_attr, expected_domain, expected_module) in
            FILENAME_INFERENCE_DATASET
        {
            let (attr, domain, module, _phase, _layer) =
                infer_categories("/tmp/unknown.docx", file_name, "");

            let attr_ok = attr == *expected_attr;
            let domain_ok = domain == *expected_domain;
            let module_ok = module == *expected_module;

            if attr_ok && domain_ok && module_ok {
                correct += 1;
            }
        }

        let accuracy = (correct as f64) / (total as f64) * 100.0;
        // 纯文件名推断目标 ~50%，此处实际准确率应更高
        assert!(
            accuracy >= 50.0,
            "文件名推断准确率 {}% 低于目标 50%（{}/{} 正确）",
            accuracy,
            correct,
            total
        );
    }
}
