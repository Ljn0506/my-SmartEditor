import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Upload,
  Library,
  Sparkles,
  SearchCheck,
  Settings,
  FileText,
  ClipboardCopy,
  AlertTriangle,
  Download,
  Trash2,
  Play,
} from "lucide-react";
import TiptapEditor from "./components/TiptapEditor";
import LibraryTab from "./components/LibraryTab";

// 标签页类型
type TabKey = "upload" | "library" | "push" | "check" | "settings";

const tabs: { key: TabKey; label: string; icon: React.ReactNode }[] = [
  { key: "upload", label: "需求上传", icon: <Upload size={16} /> },
  { key: "library", label: "模板库", icon: <Library size={16} /> },
  { key: "push", label: "智能推送", icon: <Sparkles size={16} /> },
  { key: "check", label: "校对", icon: <SearchCheck size={16} /> },
  { key: "settings", label: "设置", icon: <Settings size={16} /> },
];

function App() {
  const [activeTab, setActiveTab] = useState<TabKey>("upload");
  const [editor, setEditor] = useState<any>(null);
  const [copyToast, setCopyToast] = useState<string | null>(null);

  const handleCopy = async () => {
    if (!editor) return;
    const html = editor.getHTML();
    try {
      await invoke("write_clipboard_html", { html });
      setCopyToast("已复制到剪贴板");
      setTimeout(() => setCopyToast(null), 2000);
    } catch (e: any) {
      setCopyToast(`复制失败: ${String(e)}`);
      setTimeout(() => setCopyToast(null), 3000);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-50 text-gray-800">
      {/* 顶部标签栏 */}
      <header className="bg-white border-b border-gray-200 px-4 py-2 flex items-center gap-1 shadow-sm shrink-0">
        <div className="font-bold text-lg mr-4 text-blue-600">智能文档助手</div>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab.key
                ? "bg-blue-50 text-blue-600"
                : "text-gray-600 hover:bg-gray-100"
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </header>

      {/* 主内容区：左右分栏 */}
      <main className="flex-1 flex overflow-hidden">
        {/* 左侧功能面板 */}
        <div className="w-[45%] min-w-[360px] max-w-[560px] flex flex-col border-r border-gray-200 bg-white">
          <div className="flex-1 overflow-auto">
            {activeTab === "upload" && <UploadTab />}
            {activeTab === "library" && <LibraryTab editor={editor} />}
            {activeTab === "push" && <PushTab />}
            {activeTab === "check" && <CheckTab editor={editor} />}
            {activeTab === "settings" && <SettingsTab />}
          </div>
        </div>

        {/* 右侧编辑器 */}
        <div className="flex-1 flex flex-col min-w-0 bg-white">
          <TiptapEditor onEditorReady={setEditor} />
          {/* 底部操作栏 */}
          <div className="border-t border-gray-200 px-4 py-2.5 flex items-center justify-between bg-gray-50 shrink-0">
            <span className="text-xs text-gray-400">
              {editor ? `字符数: ${editor.getText().length}` : ""}
            </span>
            <button
              onClick={handleCopy}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-white border border-gray-200 rounded-md text-sm text-gray-700 hover:bg-gray-50 transition-colors"
            >
              <ClipboardCopy size={14} />
              复制到剪贴板
            </button>
          </div>
        </div>
      </main>

      {/* 复制提示 Toast */}
      {copyToast && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 rounded-lg bg-gray-800 text-white text-sm shadow-lg z-50">
          {copyToast}
        </div>
      )}
    </div>
  );
}

// ========== 需求上传标签页 ==========
function UploadTab() {
  const [isDragging, setIsDragging] = useState(false);
  const [requirements, setRequirements] = useState<
    { id: number; text: string; checked: boolean; category: string }[]
  >([
    { id: 1, text: "实现基于角色的访问控制（RBAC）", checked: true, category: "技术要求" },
    { id: 2, text: "支持 LDAP/AD 域账号集成", checked: true, category: "技术要求" },
    { id: 3, text: "密码策略需满足等保 2.0 三级要求", checked: true, category: "技术要求" },
    { id: 4, text: "投标人须具备信息安全等级保护测评机构资质", checked: true, category: "资质要求" },
    { id: 5, text: "报价须包含三年维保费用", checked: false, category: "商务要求" },
  ]);

  const toggleReq = (id: number) => {
    setRequirements((prev) =>
      prev.map((r) => (r.id === id ? { ...r, checked: !r.checked } : r))
    );
  };

  return (
    <div className="h-full overflow-auto p-6 space-y-6">
      {/* 文件上传区 */}
      <section>
        <h2 className="text-base font-semibold mb-3 flex items-center gap-2">
          <Upload size={18} />
          需求文件上传
        </h2>
        <div
          onDragEnter={() => setIsDragging(true)}
          onDragLeave={() => setIsDragging(false)}
          onDragOver={(e) => e.preventDefault()}
          onDrop={(e) => {
            e.preventDefault();
            setIsDragging(false);
            // TODO: 调用 Rust 后端解析文件
            console.log("Dropped files:", e.dataTransfer.files);
          }}
          className={`border-2 border-dashed rounded-xl p-10 text-center cursor-pointer transition-colors ${
            isDragging
              ? "border-blue-400 bg-blue-50"
              : "border-gray-300 bg-white hover:border-gray-400"
          }`}
        >
          <FileText size={40} className="mx-auto text-gray-400 mb-3" />
          <p className="text-gray-600 font-medium">拖入文件到这里 或 点击选择</p>
          <p className="text-gray-400 text-sm mt-1">
            支持: .docx .doc .pdf .xlsx .txt
          </p>
        </div>
      </section>

      {/* 需求要点提取 */}
      <section>
        <h2 className="text-base font-semibold mb-3">需求要点提取</h2>
        <div className="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
          {requirements.map((req) => (
            <label
              key={req.id}
              className="flex items-start gap-3 px-4 py-3 hover:bg-gray-50 cursor-pointer"
            >
              <input
                type="checkbox"
                checked={req.checked}
                onChange={() => toggleReq(req.id)}
                className="mt-1 w-4 h-4 text-blue-600 rounded border-gray-300"
              />
              <div className="flex-1">
                <span className={req.checked ? "" : "text-gray-400 line-through"}>
                  {req.text}
                </span>
                <span className="ml-2 text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-500">
                  {req.category}
                </span>
              </div>
            </label>
          ))}
        </div>
        <button className="mt-2 text-sm text-blue-600 hover:text-blue-700 font-medium">
          + 添加自定义要点
        </button>
      </section>

      {/* 知识库关联推荐 */}
      <section>
        <h2 className="text-base font-semibold mb-3">知识库关联推荐</h2>
        <div className="space-y-2">
          {[
            { title: "RBAC 权限设计最佳实践", score: 95 },
            { title: "LDAP 集成技术方案", score: 91 },
            { title: "等保 2.0 三级通用要求", score: 88 },
          ].map((item, i) => (
            <div
              key={i}
              className="bg-white rounded-lg border border-gray-200 px-4 py-3 flex items-center justify-between"
            >
              <div className="flex items-center gap-2">
                <FileText size={16} className="text-gray-400" />
                <span className="text-sm">{item.title}</span>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-xs text-gray-500">相关度: {item.score}%</span>
                <button className="text-xs px-2 py-1 rounded bg-blue-50 text-blue-600 hover:bg-blue-100">
                  引用
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* 生成按钮 */}
      <div className="flex gap-3">
        <button className="flex-1 bg-blue-600 text-white py-2.5 rounded-lg font-medium hover:bg-blue-700 transition-colors">
          生成技术方案草稿
        </button>
        <button className="flex-1 bg-emerald-600 text-white py-2.5 rounded-lg font-medium hover:bg-emerald-700 transition-colors">
          生成商务响应文档
        </button>
      </div>
    </div>
  );
}

// ========== 智能推送标签页 ==========
function PushTab() {
  return (
    <div className="h-full p-6">
      <h2 className="text-base font-semibold mb-4 flex items-center gap-2">
        <Sparkles size={18} />
        智能内容推送
      </h2>
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <p className="text-gray-500 text-center py-10">
          在编辑器中输入内容，系统将自动识别关键词并推送相关模板...
        </p>
      </div>
    </div>
  );
}

// ========== 偏离检查面板 ==========

type DeviationStatus = "None" | "Positive" | "Minor" | "Major";

interface DeviationCheckResult {
  id: number;
  section: string;
  requirement_text: string;
  response_text: string | null;
  status: DeviationStatus;
  risk_level: string;
  explanation: string;
  suggestion: string;
}

interface DeviationReport {
  total: number;
  none_count: number;
  positive_count: number;
  minor_count: number;
  major_count: number;
  fatal_risk_count: number;
  items: DeviationCheckResult[];
}

interface FatalRisk {
  category: string;
  description: string;
  risk_level: string;
  suggestion: string;
}

interface PunctuationIssue {
  id: number;
  message: string;
  severity: string;
  original: string;
  suggestion: string;
  position: number;
}

function statusLabel(s: DeviationStatus): string {
  const map: Record<DeviationStatus, string> = {
    None: "完全响应",
    Positive: "正偏离",
    Minor: "轻微偏离",
    Major: "重大偏离",
  };
  return map[s];
}

function statusColor(s: DeviationStatus): string {
  const map: Record<DeviationStatus, string> = {
    None: "bg-green-50 text-green-700 border-green-200",
    Positive: "bg-blue-50 text-blue-700 border-blue-200",
    Minor: "bg-amber-50 text-amber-700 border-amber-200",
    Major: "bg-red-50 text-red-700 border-red-200",
  };
  return map[s];
}

function CheckTab({ editor }: { editor: any }) {
  const [reqFile, setReqFile] = useState<string | null>(null);
  const [bidFile, setBidFile] = useState<string | null>(null);
  const [report, setReport] = useState<DeviationReport | null>(null);
  const [fatalRisks, setFatalRisks] = useState<FatalRisk[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [punctIssues, setPunctIssues] = useState<PunctuationIssue[]>([]);
  const [punctLoading, setPunctLoading] = useState(false);

  const pickFile = async (type: "req" | "bid") => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "文档", extensions: ["docx", "doc", "pdf", "xlsx", "txt"] },
      ],
    });
    if (selected && typeof selected === "string") {
      if (type === "req") setReqFile(selected);
      else setBidFile(selected);
      setError(null);
    }
  };

  const runCheck = async () => {
    if (!reqFile || !bidFile) {
      setError("请同时上传招标文件和投标文档");
      return;
    }
    setLoading(true);
    setError(null);
    setReport(null);
    setFatalRisks([]);
    try {
      const result: DeviationReport = await invoke("check_deviation_files", {
        bidPath: bidFile,
        reqPath: reqFile,
      });
      setReport(result);

      const bidText: string = await invoke("parse_document", {
        filePath: bidFile,
      });
      const risks: FatalRisk[] = await invoke("check_fatal_risks_text", {
        text: bidText,
      });
      setFatalRisks(risks);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const exportMd = () => {
    if (!report) return;
    let md = "# 偏离检查报告\n\n";
    md += `| 总项 | 完全响应 | 正偏离 | 轻微偏离 | 重大偏离 |\n`;
    md += `|------|----------|--------|----------|----------|\n`;
    md += `| ${report.total} | ${report.none_count} | ${report.positive_count} | ${report.minor_count} | ${report.major_count} |\n\n`;
    md += "| 序号 | 章节 | 要求 | 状态 | 风险 | 说明 | 建议 |\n";
    md += "|------|------|------|------|------|------|------|\n";
    report.items.forEach((it) => {
      md += `| ${it.id} | ${it.section} | ${it.requirement_text} | ${statusLabel(it.status)} | ${it.risk_level} | ${it.explanation} | ${it.suggestion} |\n`;
    });
    if (fatalRisks.length > 0) {
      md += "\n## 废标风险项\n\n";
      fatalRisks.forEach((r) => {
        md += `- **${r.category}**：${r.description}（${r.risk_level}）→ ${r.suggestion}\n`;
      });
    }
    const blob = new Blob([md], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "偏离检查报告.md";
    a.click();
    URL.revokeObjectURL(url);
  };

  const clear = () => {
    setReqFile(null);
    setBidFile(null);
    setReport(null);
    setFatalRisks([]);
    setError(null);
    setPunctIssues([]);
  };

  const runPunctuationCheck = async () => {
    if (!editor) {
      setError("编辑器未就绪");
      return;
    }
    const text = editor.getText();
    setPunctLoading(true);
    setError(null);
    try {
      const issues: PunctuationIssue[] = await invoke("check_punctuation", { text });
      setPunctIssues(issues);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setPunctLoading(false);
    }
  };

  const applyPunctuationFixes = () => {
    if (!editor || punctIssues.length === 0) return;
    // 从后往前替换，避免位置偏移
    const sorted = [...punctIssues].sort((a, b) => b.position - a.position);
    let text = editor.getText();
    for (const issue of sorted) {
      const before = text.slice(0, issue.position);
      const after = text.slice(issue.position + issue.original.length);
      text = before + issue.suggestion + after;
    }
    editor.chain().focus().setContent(`<p>${text.replace(/\n/g, "</p><p>")}</p>`).run();
    setPunctIssues([]);
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* 头部上传区 */}
      <div className="shrink-0 p-4 border-b border-gray-200 bg-white space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold flex items-center gap-2">
            <SearchCheck size={18} />
            偏离检查
          </h2>
          {(reqFile || bidFile || report) && (
            <button
              onClick={clear}
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
            onClick={() => pickFile("req")}
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
            onClick={() => pickFile("bid")}
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
          onClick={runCheck}
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
                  onClick={exportMd}
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
                    className={`px-4 py-3 text-sm border-l-4 ${statusColor(it.status).replace(/bg-[^ ]+/, "")}`}
                    style={{
                      borderLeftColor:
                        it.status === "None"
                          ? "#22c55e"
                          : it.status === "Positive"
                          ? "#3b82f6"
                          : it.status === "Minor"
                          ? "#f59e0b"
                          : "#ef4444",
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
                        className={`shrink-0 px-2 py-0.5 rounded text-xs font-medium border ${statusColor(it.status)}`}
                      >
                        {statusLabel(it.status)}
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

        {/* 格式检查 */}
        <div className="bg-white rounded-lg border border-gray-200">
          <div className="px-4 py-3 border-b border-gray-100 font-medium text-sm flex items-center justify-between">
            <span className="flex items-center gap-2">
              <SearchCheck size={16} className="text-gray-500" />
              格式检查（标点符号）
            </span>
            <div className="flex gap-2">
              {punctIssues.length > 0 && (
                <button
                  onClick={applyPunctuationFixes}
                  className="text-xs px-3 py-1.5 rounded bg-emerald-50 text-emerald-600 hover:bg-emerald-100 flex items-center gap-1"
                >
                  一键修复 ({punctIssues.length})
                </button>
              )}
              <button
                onClick={runPunctuationCheck}
                disabled={punctLoading || !editor}
                className="text-xs px-3 py-1.5 rounded bg-blue-50 text-blue-600 hover:bg-blue-100 flex items-center gap-1 disabled:opacity-50"
              >
                <SearchCheck size={14} />
                {punctLoading ? "检查中..." : "检查标点符号"}
              </button>
            </div>
          </div>
          {punctIssues.length === 0 && !punctLoading && (
            <div className="px-4 py-6 text-center text-gray-400 text-xs">
              点击"检查标点符号"扫描编辑器内容
            </div>
          )}
          {punctLoading && (
            <div className="px-4 py-6 text-center text-gray-400 text-xs">
              检查中...
            </div>
          )}
          {punctIssues.length > 0 && (
            <div className="divide-y divide-gray-100 max-h-48 overflow-auto">
              {punctIssues.map((issue) => (
                <div key={issue.id} className="px-4 py-2.5 text-sm flex items-start gap-3">
                  <span
                    className={`shrink-0 px-1.5 py-0.5 rounded text-xs font-medium ${
                      issue.severity === "error"
                        ? "bg-red-50 text-red-700"
                        : "bg-amber-50 text-amber-700"
                    }`}
                  >
                    {issue.severity === "error" ? "错误" : "警告"}
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
      </div>
    </div>
  );
}

// ========== 设置标签页 ==========
function SettingsTab() {
  return (
    <div className="h-full p-6">
      <h2 className="text-base font-semibold mb-4 flex items-center gap-2">
        <Settings size={18} />
        设置
      </h2>
      <div className="max-w-xl space-y-4">
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="font-medium mb-3">AI 配置</h3>
          <div className="space-y-3">
            <div>
              <label className="text-sm text-gray-600 block mb-1">AI 模式</label>
              <select className="w-full px-3 py-2 rounded border border-gray-200 text-sm">
                <option>本地 Ollama（优先）</option>
                <option>云端 API</option>
                <option>自动切换</option>
              </select>
            </div>
            <div>
              <label className="text-sm text-gray-600 block mb-1">Ollama 地址</label>
              <input
                type="text"
                defaultValue="http://localhost:11434"
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              />
            </div>
            <div>
              <label className="text-sm text-gray-600 block mb-1">云端 API Key</label>
              <input
                type="password"
                placeholder="sk-..."
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              />
            </div>
          </div>
        </div>
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="font-medium mb-3">知识库配置</h3>
          <div>
            <label className="text-sm text-gray-600 block mb-1">NAS 扫描路径</label>
            <input
              type="text"
              placeholder="/Volumes/NAS/投标文件"
              className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
