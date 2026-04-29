import { Upload, Play, Download, Trash2, AlertTriangle } from "lucide-react";

export interface DeviationCheckResult {
  id: number;
  section: string;
  requirement_text: string;
  response_text?: string;
  status: string;
  risk_level: string;
  explanation: string;
  suggestion: string;
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

interface DeviationPanelProps {
  reqFile: string | null;
  bidFile: string | null;
  report: DeviationReport | null;
  fatalRisks: FatalRisk[];
  loading: boolean;
  error: string | null;
  onPickFile: (type: "req" | "bid") => void;
  onRunCheck: () => void;
  onExportMd: () => void;
  onClear: () => void;
}

const STATUS_META: Record<
  string,
  { border: string; label: string; bg: string }
> = {
  None: {
    border: "#22c55e",
    label: "完全响应",
    bg: "bg-green-50 text-green-700 border-green-200",
  },
  Positive: {
    border: "#3b82f6",
    label: "正偏离",
    bg: "bg-blue-50 text-blue-700 border-blue-200",
  },
  Minor: {
    border: "#f59e0b",
    label: "轻微偏离",
    bg: "bg-amber-50 text-amber-700 border-amber-200",
  },
  Major: {
    border: "#ef4444",
    label: "重大偏离",
    bg: "bg-red-50 text-red-700 border-red-200",
  },
};

function statusMeta(s: string) {
  return STATUS_META[s] || STATUS_META.Major;
}

export default function DeviationPanel({
  reqFile,
  bidFile,
  report,
  fatalRisks,
  loading,
  error,
  onPickFile,
  onRunCheck,
  onExportMd,
  onClear,
}: DeviationPanelProps) {
  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* 头部上传区 */}
      <div className="shrink-0 p-4 border-b border-gray-200 bg-white space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold flex items-center gap-2">
            <AlertTriangle size={18} />
            偏离检查
          </h2>
          {(reqFile || bidFile || report) && (
            <button
              onClick={onClear}
              className="text-xs flex items-center gap-1 text-gray-500 hover:text-red-600"
            >
              <Trash2 size={14} />
              重置
            </button>
          )}
        </div>

        <div className="grid grid-cols-2 gap-3">
          {/* 招标文件 */}
          <div
            role="button"
            tabIndex={0}
            onClick={() => onPickFile("req")}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onPickFile("req");
              }
            }}
            className={`border-2 border-dashed rounded-lg p-3 text-center cursor-pointer transition-colors ${
              reqFile
                ? "border-green-400 bg-green-50"
                : "border-gray-300 hover:border-blue-400"
            }`}
          >
            <Upload size={20} className={`mx-auto mb-1 ${reqFile ? "text-green-600" : "text-gray-400"}`} />
            <p className="text-xs font-medium text-gray-700">
              {reqFile ? reqFile.split(/[/\\]/).pop() : "点击选择招标文件"}
            </p>
          </div>
          {/* 投标文档 */}
          <div
            role="button"
            tabIndex={0}
            onClick={() => onPickFile("bid")}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onPickFile("bid");
              }
            }}
            className={`border-2 border-dashed rounded-lg p-3 text-center cursor-pointer transition-colors ${
              bidFile
                ? "border-green-400 bg-green-50"
                : "border-gray-300 hover:border-blue-400"
            }`}
          >
            <Upload size={20} className={`mx-auto mb-1 ${bidFile ? "text-green-600" : "text-gray-400"}`} />
            <p className="text-xs font-medium text-gray-700">
              {bidFile ? bidFile.split(/[/\\]/).pop() : "点击选择投标文档"}
            </p>
          </div>
        </div>

        {error && (
          <div className="text-xs text-red-600 bg-red-50 rounded px-3 py-2">
            {error}
          </div>
        )}

        <button
          onClick={onRunCheck}
          disabled={loading || !reqFile || !bidFile}
          className="w-full flex items-center justify-center gap-2 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
        >
          <Play size={16} />
          {loading ? "检查中..." : "开始检查"}
        </button>
      </div>

      {/* 结果展示区 */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {!report && !loading && (
          <div className="text-center text-gray-400 py-12 text-sm">
            上传招标文件和投标文档后，点击"开始检查"查看偏离分析结果
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

            {/* 偏离项列表 */}
            <div className="bg-white rounded-lg border border-gray-200">
              <div className="px-4 py-3 border-b border-gray-100 font-medium text-sm flex items-center justify-between">
                <span>偏离项详情 ({report.items.length})</span>
                <button
                  onClick={onExportMd}
                  className="text-xs flex items-center gap-1 text-blue-600 hover:text-blue-700"
                >
                  <Download size={14} />
                  导出报告
                </button>
              </div>
              <div className="divide-y divide-gray-100">
                {report.items.map((it) => (
                  <div
                    key={it.id}
                    className={`px-4 py-3 text-sm border-l-4 ${statusMeta(it.status).bg.replace(/bg-[^ ]+/, "")}`}
                    style={{
                      borderLeftColor:
                        statusMeta(it.status).border,
                    }}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1">
                        <div className="font-medium text-gray-800">
                          #{it.id} {it.requirement_text}
                        </div>
                        {it.response_text && (
                          <div className="text-xs text-gray-500 mt-1">
                            应答：{it.response_text}
                          </div>
                        )}
                        <div className="text-xs text-gray-500 mt-1">
                          {it.explanation}
                        </div>
                        <div className="text-xs text-blue-600 mt-1">
                          建议：{it.suggestion}
                        </div>
                      </div>
                      <span
                        className={`shrink-0 px-2 py-0.5 rounded text-xs font-medium border ${statusMeta(it.status).bg}`}
                      >
                        {statusMeta(it.status).label}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 废标风险 */}
            {fatalRisks.length > 0 && (
              <div className="bg-white rounded-lg border border-red-200">
                <div className="px-4 py-3 border-b border-red-100 font-medium text-sm text-red-700 flex items-center gap-2">
                  <AlertTriangle size={16} />
                  废标风险项 ({fatalRisks.length})
                </div>
                <div className="divide-y divide-red-50">
                  {fatalRisks.map((r, i) => (
                    <div key={i} className="px-4 py-3 text-sm">
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1">
                          <div className="font-medium text-red-700">
                            {r.category}
                          </div>
                          <div className="text-gray-600 mt-0.5">
                            {r.description}
                          </div>
                          <div className="text-xs text-blue-600 mt-1">
                            建议：{r.suggestion}
                          </div>
                        </div>
                        <span className="shrink-0 px-2 py-0.5 rounded text-xs font-medium bg-red-50 text-red-700 border border-red-200">
                          {r.risk_level}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
