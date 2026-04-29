import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Upload,
  Library,
  Sparkles,
  SearchCheck,
  Settings,
  FileText,
  ClipboardCopy,
} from "lucide-react";
import TiptapEditor from "./components/TiptapEditor";
import LibraryTab from "./components/LibraryTab";
import DeviationPanel, {
  type DeviationReport,
  type FatalRisk,
} from "./components/DeviationPanel";
import PunctuationPanel, { type PunctuationIssue } from "./components/PunctuationPanel";

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
  const [importProgress, setImportProgress] = useState<{ current: number; total: number; phase: string } | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("import_progress", (event: any) => {
      const payload = event.payload as { current: number; total: number; phase: string };
      setImportProgress((prev) => {
        if (prev && prev.current === payload.current && prev.phase === payload.phase) {
          return prev;
        }
        return payload;
      });
      if (payload.current >= payload.total && payload.phase === "index") {
        setTimeout(() => setImportProgress(null), 3000);
      }
    })
      .then((f) => {
        unlisten = f;
      })
      .catch(() => {
        // 非 Tauri 环境（如测试），忽略
      });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

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
        <div
          role="status"
          aria-live="polite"
          className="fixed bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 rounded-lg bg-gray-800 text-white text-sm shadow-lg z-50"
        >
          {copyToast}
        </div>
      )}

      {/* NAS 导入进度 */}
      {importProgress && (
        <div
          role="progressbar"
          aria-valuenow={importProgress.current}
          aria-valuemin={0}
          aria-valuemax={importProgress.total}
          aria-label="导入进度"
          className="fixed bottom-16 left-1/2 -translate-x-1/2 w-64 bg-white rounded-lg shadow-lg border border-gray-200 p-3 z-50"
        >
          <div className="text-xs text-gray-600 mb-1.5 flex justify-between">
            <span>{importProgress.phase === "parse" ? "正在解析文件..." : "正在建立索引..."}</span>
            <span>{importProgress.current}/{importProgress.total}</span>
          </div>
          <div className="h-1.5 bg-gray-100 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-300"
              style={{ width: `${Math.min(100, (importProgress.current / importProgress.total) * 100)}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

// ========== 需求上传标签页 ==========
function UploadTab() {
  const [isDragging, setIsDragging] = useState(false);
  const [fileName, setFileName] = useState<string | null>(null);
  const [parsedText, setParsedText] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
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

  const handleParseFile = async (filePath: string, name: string) => {
    setIsLoading(true);
    setError(null);
    setParsedText("");
    try {
      const text: string = await invoke("parse_document", { filePath });
      setFileName(name);
      setParsedText(text);
    } catch (e: any) {
      setError(`解析失败: ${String(e)}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = e.dataTransfer.files;
    if (files.length === 0) return;
    const file = files[0];
    // Tauri 桌面环境中 File 对象可能暴露 path 属性
    const path = (file as any).path;
    if (path) {
      await handleParseFile(path, file.name);
    } else {
      setError("无法获取文件路径，请使用点击选择文件");
    }
  };

  const handleSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "文档", extensions: ["docx", "doc", "pdf", "xlsx", "txt"] },
        ],
      });
      if (selected && typeof selected === "string") {
        const name = selected.split("/").pop() || selected;
        await handleParseFile(selected, name);
      }
    } catch (e: any) {
      setError(`选择文件失败: ${String(e)}`);
    }
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
          role="button"
          tabIndex={0}
          onDragEnter={() => setIsDragging(true)}
          onDragLeave={() => setIsDragging(false)}
          onDragOver={(e) => e.preventDefault()}
          onDrop={handleDrop}
          onClick={handleSelect}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              handleSelect();
            }
          }}
          className={`border-2 border-dashed rounded-xl p-10 text-center cursor-pointer transition-colors ${
            isDragging
              ? "border-blue-400 bg-blue-50"
              : "border-gray-300 bg-white hover:border-gray-400"
          }`}
        >
          <FileText size={40} className="mx-auto text-gray-400 mb-3" />
          {isLoading ? (
            <p className="text-blue-600 font-medium">正在解析文件...</p>
          ) : fileName ? (
            <>
              <p className="text-gray-800 font-medium">{fileName}</p>
              <p className="text-gray-400 text-sm mt-1">点击重新选择文件</p>
            </>
          ) : (
            <>
              <p className="text-gray-600 font-medium">拖入文件到这里 或 点击选择</p>
              <p className="text-gray-400 text-sm mt-1">
                支持: .docx .doc .pdf .xlsx .txt
              </p>
            </>
          )}
        </div>
        {error && (
          <p className="mt-2 text-sm text-red-600">{error}</p>
        )}
      </section>

      {/* 解析结果预览 */}
      {parsedText && (
        <section>
          <h2 className="text-base font-semibold mb-3">解析结果预览</h2>
          <div className="bg-gray-50 rounded-lg border border-gray-200 p-4 max-h-64 overflow-auto">
            <pre className="text-xs text-gray-700 whitespace-pre-wrap font-mono leading-relaxed">
              {parsedText.length > 2000 ? parsedText.slice(0, 2000) + "\n...（内容过长，已截断）" : parsedText}
            </pre>
          </div>
        </section>
      )}

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

  const exportMd = async () => {
    if (!report) return;
    try {
      const md: string = await invoke("export_deviation_report_markdown", { report });
      const blob = new Blob([md], { type: "text/markdown" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "偏离检查报告.md";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: any) {
      setError(`导出失败: ${String(e)}`);
    }
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
    editor
      .chain()
      .focus()
      .command(({ tr, state }: any) => {
        for (const issue of sorted) {
          const from = issue.position + 1; // ProseMirror 位置从 1 开始
          const to = from + issue.original.length;
          // 安全检查：位置有效且内容匹配才替换
          if (from < 1 || to > state.doc.content.size) continue;
          const currentText = state.doc.textBetween(from, to);
          if (currentText !== issue.original) continue;
          // 使用 insertText 保留 marks 和 undo 历史
          tr.insertText(issue.suggestion, from, to);
        }
        return true;
      })
      .run();
    setPunctIssues([]);
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <DeviationPanel
        reqFile={reqFile}
        bidFile={bidFile}
        report={report}
        fatalRisks={fatalRisks}
        loading={loading}
        error={error}
        onPickFile={pickFile}
        onRunCheck={runCheck}
        onExportMd={exportMd}
        onClear={clear}
      />
      <PunctuationPanel
        editor={editor}
        issues={punctIssues}
        loading={punctLoading}
        onRunCheck={runPunctuationCheck}
        onApplyFixes={applyPunctuationFixes}
      />
    </div>
  );
}

// ========== 设置标签页 ==========
function SettingsTab() {
  const [aiConfig, setAiConfig] = useState<{
    provider: string;
    base_url: string;
    api_key: string;
    model: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    invoke("get_ai_config")
      .then((cfg: any) => {
        setAiConfig({
          provider: cfg.provider,
          base_url: cfg.base_url,
          api_key: cfg.api_key || "",
          model: cfg.model,
        });
      })
      .catch((e) => setMsg(`加载配置失败: ${String(e)}`))
      .finally(() => setLoading(false));
  }, []);

  const handleSave = async () => {
    if (!aiConfig) return;
    setSaving(true);
    setMsg(null);
    try {
      await invoke("update_ai_config", {
        ai: {
          provider: aiConfig.provider,
          base_url: aiConfig.base_url,
          api_key: aiConfig.api_key || null,
          model: aiConfig.model,
        },
      });
      setMsg("配置已保存");
      setTimeout(() => setMsg(null), 3000);
    } catch (e: any) {
      setMsg(`保存失败: ${String(e)}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="h-full p-6 flex items-center justify-center text-gray-400 text-sm">
        加载配置中...
      </div>
    );
  }

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
              <label htmlFor="ai-mode" className="text-sm text-gray-600 block mb-1">AI 提供商</label>
              <select
                id="ai-mode"
                value={aiConfig?.provider || "Ollama"}
                onChange={(e) =>
                  setAiConfig((prev) =>
                    prev ? { ...prev, provider: e.target.value } : null
                  )
                }
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              >
                <option value="Ollama">本地 Ollama</option>
                <option value="Claude">Claude API</option>
                <option value="DeepSeek">DeepSeek API</option>
              </select>
            </div>
            <div>
              <label htmlFor="ai-base-url" className="text-sm text-gray-600 block mb-1">API 地址</label>
              <input
                id="ai-base-url"
                type="text"
                value={aiConfig?.base_url || ""}
                onChange={(e) =>
                  setAiConfig((prev) =>
                    prev ? { ...prev, base_url: e.target.value } : null
                  )
                }
                placeholder="http://localhost:11434"
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              />
            </div>
            <div>
              <label htmlFor="ai-api-key" className="text-sm text-gray-600 block mb-1">API Key</label>
              <input
                id="ai-api-key"
                type="password"
                value={aiConfig?.api_key || ""}
                onChange={(e) =>
                  setAiConfig((prev) =>
                    prev ? { ...prev, api_key: e.target.value } : null
                  )
                }
                placeholder="sk-..."
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              />
            </div>
            <div>
              <label htmlFor="ai-model" className="text-sm text-gray-600 block mb-1">模型</label>
              <input
                id="ai-model"
                type="text"
                value={aiConfig?.model || ""}
                onChange={(e) =>
                  setAiConfig((prev) =>
                    prev ? { ...prev, model: e.target.value } : null
                  )
                }
                placeholder="qwen2.5:14b"
                className="w-full px-3 py-2 rounded border border-gray-200 text-sm"
              />
            </div>
            {msg && (
              <p className={`text-xs ${msg.includes("失败") ? "text-red-600" : "text-emerald-600"}`}>
                {msg}
              </p>
            )}
            <button
              onClick={handleSave}
              disabled={saving || !aiConfig}
              className="w-full py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:bg-gray-300 transition-colors"
            >
              {saving ? "保存中..." : "保存配置"}
            </button>
          </div>
        </div>
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="font-medium mb-3">知识库配置</h3>
          <div>
            <label htmlFor="nas-path" className="text-sm text-gray-600 block mb-1">NAS 扫描路径</label>
            <input
              id="nas-path"
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
