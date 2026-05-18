import { Play, Download, Trash2, AlertTriangle, SearchCheck, FileCheck, Wrench } from "lucide-react";
import type { RequirementItem } from "../contexts/RequirementsContext";

export interface DeviationCheckResult {
  id: number;
  section: string;
  requirement_text: string;
  response_text?: string;
  status: string;
  risk_level: string;
  explanation: string;
  suggestion: string;
  paragraph_index?: number;
}

export interface DeviationReport {
  total: number;
  none_count: number;
  positive_count: number;
  minor_count: number;
  major_count: number;
  fatal_risk_count: number;
  items: DeviationCheckResult[];
}

export interface FatalRisk {
  category: string;
  description: string;
  risk_level: string;
  suggestion: string;
}

export interface SelfReviewIssue {
  id: number;
  category: string;
  sub_category: string;
  message: string;
  severity: "Error" | "Warning" | "Info";
  position?: number;
  paragraph_index?: number;
  original?: string;
  suggestion?: string;
  auto_fixable: boolean;
}

export interface SelfReviewReport {
  issues: SelfReviewIssue[];
}

interface CheckResultsPanelProps {
  requirements: RequirementItem[];
  report: DeviationReport | null;
  fatalRisks: FatalRisk[];
  selfReviewReport: SelfReviewReport | null;
  loading: boolean;
  error: string | null;
  onRunCheck: () => void;
  onExportMd: () => void;
  onClear: () => void;
  onJumpToParagraph: (idx: number | undefined) => void;
  onFix?: () => void;
  hasFixableIssues?: boolean;
  isDocFile?: boolean;
  hasRequirements?: boolean;
}

import { statusMeta, severityMeta } from "../constants/meta";

export default function CheckResultsPanel({
  requirements,
  report,
  fatalRisks,
  selfReviewReport,
  loading,
  error,
  onRunCheck,
  onExportMd,
  onClear,
  onJumpToParagraph,
  onFix,
  hasFixableIssues,
  isDocFile,
  hasRequirements,
}: CheckResultsPanelProps) {
  const checkedCount = requirements.filter((r) => r.checked).length;
  const hasResults = report || fatalRisks.length > 0 || (selfReviewReport && selfReviewReport.issues.length > 0);
  const canRunCheck = hasRequirements && requirements.some((r) => r.checked);

  // 分类自查问题
  const issues = selfReviewReport?.issues || [];
  const consistencyIssues = issues.filter((i) => i.category === "consistency");
  const qualityIssues = issues.filter((i) => i.category === "quality");
  const formatIssues = issues.filter((i) => i.category === "format");

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* 头部操作区 */}
      <div className="shrink-0 p-4 border-b border-gray-200 bg-white space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold flex items-center gap-2">
            <SearchCheck size={18} />
            校对检查
          </h2>
          {hasResults && (
            <button
              onClick={onClear}
              className="text-xs flex items-center gap-1 text-gray-500 hover:text-red-600"
            >
              <Trash2 size={14} />
              重置
            </button>
          )}
        </div>

        {/* 需求同步状态 */}
        <div className="flex items-center gap-2 text-xs">
          <FileCheck size={14} className={requirements.length > 0 ? "text-green-600" : "text-gray-400"} />
          <span className={requirements.length > 0 ? "text-green-700" : "text-gray-500"}>
            {requirements.length > 0
              ? `已同步 ${checkedCount}/${requirements.length} 条需求`
              : "未同步招标需求（请先在「需求上传」Tab 上传文件）"}
          </span>
        </div>

        {error && (
          <div className="text-xs text-red-600 bg-red-50 rounded px-3 py-2">
            {error}
          </div>
        )}

        {!hasRequirements && (
          <div className="text-xs text-amber-600 bg-amber-50 rounded px-3 py-2">
            请先前往「需求上传」解析招标文件
          </div>
        )}

        <button
          onClick={onRunCheck}
          disabled={loading || !canRunCheck}
          className="w-full flex items-center justify-center gap-2 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
        >
          <Play size={16} />
          {loading ? "检查中..." : "检查投标文件"}
        </button>
      </div>

      {/* 结果展示区 */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {!hasResults && !loading && (
          <div className="text-center text-gray-400 py-12 text-sm">
            点击「检查投标文件」开始校对
          </div>
        )}

        {report && (
          <>
            {/* 统计卡片 */}
            <div className="grid grid-cols-5 gap-2">
              {[
                { label: "完全响应", count: report.none_count, color: "bg-green-100 text-green-700" },
                { label: "正偏离", count: report.positive_count, color: "bg-blue-100 text-blue-700" },
                { label: "轻微偏离", count: report.minor_count, color: "bg-amber-100 text-amber-700" },
                { label: "重大偏离", count: report.major_count, color: "bg-red-100 text-red-700" },
                { label: "致命风险", count: fatalRisks.length, color: "bg-gray-100 text-gray-700" },
              ].map((c) => (
                <div key={c.label} className={`rounded-lg p-2 text-center ${c.color}`}>
                  <div className="text-lg font-bold">{c.count}</div>
                  <div className="text-xs">{c.label}</div>
                </div>
              ))}
            </div>

            {/* 偏离风险 */}
            <SectionCard title={`偏离风险 (${report.items.length})`} icon={<AlertTriangle size={16} />}>
              <div className="divide-y divide-gray-100">
                {report.items.map((it) => (
                  <button
                    key={it.id}
                    onClick={() => onJumpToParagraph(it.paragraph_index)}
                    className={`w-full text-left px-3 py-2.5 text-sm border-l-4 hover:bg-gray-50 transition-colors ${statusMeta(it.status).bg.replace(/bg-[^ ]+/, "")}`}
                    style={{ borderLeftColor: statusMeta(it.status).border }}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-gray-800 truncate">#{it.id} {it.requirement_text}</div>
                        {it.response_text && (
                          <div className="text-xs text-gray-500 mt-0.5">应答：{it.response_text}</div>
                        )}
                        <div className="text-xs text-gray-500 mt-0.5">{it.explanation}</div>
                        {it.paragraph_index !== undefined && (
                          <div className="text-xs text-blue-600 mt-0.5">点击定位到第 {it.paragraph_index + 1} 段</div>
                        )}
                      </div>
                      <span className={`shrink-0 px-2 py-0.5 rounded text-xs font-medium border ${statusMeta(it.status).bg}`}>
                        {statusMeta(it.status).label}
                      </span>
                    </div>
                  </button>
                ))}
              </div>
            </SectionCard>

            {/* 废标风险 — 置顶红色 */}
            {fatalRisks.length > 0 && (
              <SectionCard
                title={`废标风险 (${fatalRisks.length})`}
                icon={<AlertTriangle size={16} className="text-red-600" />}
                headerClass="text-red-700 bg-red-50 border-red-100"
              >
                <div className="divide-y divide-red-50">
                  {fatalRisks.map((r, i) => (
                    <div key={i} className="px-3 py-2.5 text-sm">
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-red-700">{r.category}</div>
                          <div className="text-gray-600 text-xs mt-0.5">{r.description}</div>
                        </div>
                        <span className="shrink-0 px-2 py-0.5 rounded text-xs font-medium bg-red-50 text-red-700 border border-red-200">
                          {r.risk_level}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </SectionCard>
            )}
          </>
        )}

        {/* 内容一致性 */}
        {consistencyIssues.length > 0 && (
          <SectionCard title={`内容一致性 (${consistencyIssues.length})`} icon={<SearchCheck size={16} />}>
            <IssueList issues={consistencyIssues} onJump={onJumpToParagraph} />
          </SectionCard>
        )}

        {/* 内容质量 */}
        {qualityIssues.length > 0 && (
          <SectionCard title={`内容质量 (${qualityIssues.length})`} icon={<SearchCheck size={16} />}>
            <IssueList issues={qualityIssues} onJump={onJumpToParagraph} />
          </SectionCard>
        )}

        {/* 格式问题 */}
        {formatIssues.length > 0 && (
          <SectionCard title={`格式问题 (${formatIssues.length})`} icon={<SearchCheck size={16} />}>
            <IssueList issues={formatIssues} onJump={onJumpToParagraph} />
          </SectionCard>
        )}

        {/* 一键修复 + 导出按钮 */}
        {hasResults && (
          <div className="space-y-2">
            {onFix && (
              <button
                onClick={onFix}
                disabled={!hasFixableIssues || isDocFile}
                className="w-full flex items-center justify-center gap-2 py-2 bg-emerald-600 text-white rounded-lg text-sm font-medium hover:bg-emerald-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
              >
                <Wrench size={14} />
                {isDocFile ? "请转换为 .docx 后使用一键修复" : "一键修复可修复项"}
              </button>
            )}
            {report && (
              <button
                onClick={onExportMd}
                className="w-full flex items-center justify-center gap-2 py-2 border border-gray-200 rounded-lg text-sm text-gray-700 hover:bg-gray-50 transition-colors"
              >
                <Download size={14} />
                导出偏离报告（Markdown）
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function SectionCard({
  title,
  icon,
  children,
  headerClass,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  headerClass?: string;
}) {
  const [expanded, setExpanded] = useState(true);
  return (
    <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <button
        onClick={() => setExpanded((v) => !v)}
        className={`w-full px-4 py-2.5 border-b border-gray-100 font-medium text-sm flex items-center justify-between ${headerClass || ""}`}
      >
        <span className="flex items-center gap-2">{icon}{title}</span>
        <span className="text-gray-400 text-xs">{expanded ? "收起" : "展开"}</span>
      </button>
      {expanded && children}
    </div>
  );
}

import { useState } from "react";

function IssueList({
  issues,
  onJump,
}: {
  issues: SelfReviewIssue[];
  onJump: (idx: number | undefined) => void;
}) {
  return (
    <div className="divide-y divide-gray-100">
      {issues.map((issue) => (
        <button
          key={issue.id}
          onClick={() => onJump(issue.paragraph_index)}
          className="w-full text-left px-3 py-2.5 text-sm hover:bg-gray-50 transition-colors flex items-start gap-3"
        >
          <span className={`shrink-0 px-1.5 py-0.5 rounded text-xs font-medium border ${severityMeta(issue.severity).bg}`}>
            {severityMeta(issue.severity).label}
          </span>
          <div className="flex-1 min-w-0">
            <div className="text-gray-700">{issue.message}</div>
            {issue.original && issue.suggestion && (
              <div className="text-xs text-gray-500 mt-0.5">
                <span className="line-through">{issue.original.slice(0, 40)}</span>
                <span className="mx-1">→</span>
                <span className="text-emerald-600 font-medium">{issue.suggestion}</span>
              </div>
            )}
            {issue.paragraph_index !== undefined && (
              <div className="text-xs text-blue-600 mt-0.5">点击定位到第 {issue.paragraph_index + 1} 段</div>
            )}
          </div>
        </button>
      ))}
    </div>
  );
}
