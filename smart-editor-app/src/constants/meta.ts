/**
 * 共享状态/严重度元数据常量
 * 提取自 CheckResultsPanel、DeviationPanel、PunctuationPanel
 */

export const STATUS_META: Record<
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

export const SEVERITY_META: Record<string, { bg: string; label: string }> = {
  Error: { bg: "bg-red-50 text-red-700 border-red-200", label: "错误" },
  Warning: {
    bg: "bg-amber-50 text-amber-700 border-amber-200",
    label: "警告",
  },
  Info: { bg: "bg-blue-50 text-blue-700 border-blue-200", label: "提示" },
};

export function statusMeta(s: string) {
  return STATUS_META[s] || STATUS_META.Major;
}

export function severityMeta(s: string) {
  return SEVERITY_META[s] || SEVERITY_META.Info;
}
