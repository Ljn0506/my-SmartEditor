//! 文档分类推断公共模块

use crate::models::{BusinessDomain, ContentModule, DocAttr};

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

/// 根据文件名推断文档的多维分类属性
pub fn infer_categories(
    file_name: &str,
    _text: &str,
) -> (
    Option<DocAttr>,
    Option<BusinessDomain>,
    Option<ContentModule>,
) {
    let lower = file_name.to_lowercase();

    let doc_attr = if lower.contains("投标") || lower.contains("应答") {
        Some(DocAttr::BidResponse)
    } else if lower.contains("技术方案") || lower.contains("设计方案") {
        Some(DocAttr::TechnicalProposal)
    } else if lower.contains("实施") {
        Some(DocAttr::ImplementationPlan)
    } else if lower.contains("合同") || lower.contains("协议") {
        Some(DocAttr::ContractAgreement)
    } else {
        Some(DocAttr::ReportMaterial)
    };

    let business_domain = if lower.contains("等保") || lower.contains("等级保护") {
        Some(BusinessDomain::NetworkSecurity)
    } else if lower.contains("渗透") || lower.contains("漏洞") || lower.contains("代码审计")
    {
        Some(BusinessDomain::ApplicationSecurity)
    } else if lower.contains("数据") || lower.contains("数据库") || lower.contains("隐私") {
        Some(BusinessDomain::DataSecurity)
    } else if lower.contains("soc") || lower.contains("运营") || lower.contains("态势") {
        Some(BusinessDomain::SecurityOperation)
    } else if lower.contains("管理") || lower.contains("制度") || lower.contains("体系") {
        Some(BusinessDomain::SecurityManagement)
    } else {
        Some(BusinessDomain::NetworkSecurity)
    };

    let content_module = if lower.contains("偏离") || lower.contains("差异") {
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
    };

    (doc_attr, business_domain, content_module)
}
