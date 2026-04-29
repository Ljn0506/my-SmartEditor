import { SearchCheck } from "lucide-react";

export type Severity = "Error" | "Warning" | "Info";

export interface PunctuationIssue {
  id: number;
  message: string;
  severity: Severity;
  original: string;
  suggestion: string;
  position: number;
}

interface PunctuationPanelProps {
  editor: any;
  issues: PunctuationIssue[];
  loading: boolean;
  onRunCheck: () => void;
  onApplyFixes: () => void;
}

const SEVERITY_META: Record<
  string,
  { bg: string; label: string }
> = {
  Error: { bg: "bg-red-50 text-red-700", label: "错误" },
  Warning: { bg: "bg-amber-50 text-amber-700", label: "警告" },
  Info: { bg: "bg-blue-50 text-blue-700", label: "提示" },
};

function severityMeta(s: Severity) {
  return SEVERITY_META[s] || SEVERITY_META.Info;
}

export default function PunctuationPanel({
  editor,
  issues,
  loading,
  onRunCheck,
  onApplyFixes,
}: PunctuationPanelProps) {
  return (
    <div className="bg-white rounded-lg border border-gray-200">
      <div className="px-4 py-3 border-b border-gray-100 font-medium text-sm flex items-center justify-between">
        <span className="flex items-center gap-2">
          <SearchCheck size={16} className="text-gray-500" />
          格式检查（标点符号）
        </span>
        <div className="flex gap-2">
          <button
            onClick={onApplyFixes}
            disabled={issues.length === 0}
            className={`text-xs px-3 py-1.5 rounded flex items-center gap-1 transition-opacity ${
              issues.length > 0
                ? "bg-emerald-50 text-emerald-600 hover:bg-emerald-100"
                : "bg-emerald-50 text-emerald-600 opacity-0 pointer-events-none"
            }`}
          >
            一键修复 ({issues.length})
          </button>
          <button
            onClick={onRunCheck}
            disabled={loading || !editor}
            className="text-xs px-3 py-1.5 rounded bg-blue-50 text-blue-600 hover:bg-blue-100 flex items-center gap-1 disabled:opacity-50"
          >
            <SearchCheck size={14} />
            {loading ? "检查中..." : "检查标点符号"}
          </button>
        </div>
      </div>
      {issues.length === 0 && !loading && (
        <div className="px-4 py-6 text-center text-gray-400 text-xs">
          点击"检查标点符号"扫描编辑器内容
        </div>
      )}
      {loading && (
        <div className="px-4 py-6 text-center text-gray-400 text-xs">
          检查中...
        </div>
      )}
      {issues.length > 0 && (
        <div className="divide-y divide-gray-100 max-h-48 overflow-auto">
          {issues.map((issue) => (
            <div key={issue.id} className="px-4 py-2.5 text-sm flex items-start gap-3">
              <span
                className={`shrink-0 px-1.5 py-0.5 rounded text-xs font-medium ${severityMeta(issue.severity).bg}`}
              >
                {severityMeta(issue.severity).label}
              </span>
              <div className="flex-1 min-w-0">
                <div className="text-gray-700">{issue.message}</div>
                <div className="text-xs text-gray-500 mt-0.5">
                  <span className="line-through">{issue.original}</span>
                  <span className="mx-1">→</span>
                  <span className="text-emerald-600 font-medium">{issue.suggestion}</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
