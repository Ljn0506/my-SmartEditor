import { useState, useEffect, useRef } from "react";
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
import { RequirementsProvider } from "./contexts/RequirementsContext";
import CheckResultsPanel, {
  type DeviationReport,
  type FatalRisk,
} from "./components/CheckResultsPanel";


import { useRequirements } from "./contexts/RequirementsContext";

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
    <RequirementsProvider>
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
        {activeTab === "check" ? (
          <CheckTabFullScreen />
        ) : (
          <>
            {/* 左侧功能面板 */}
            <div className="w-[45%] min-w-[360px] max-w-[560px] flex flex-col border-r border-gray-200 bg-white">
              <div className="flex-1 overflow-auto">
                {activeTab === "upload" && <UploadTab />}
                {activeTab === "library" && <LibraryTab editor={editor} />}
                {activeTab === "push" && <PushTab />}
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
          </>
        )}
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
    </RequirementsProvider>
  );
}

// ========== 需求上传标签页 ==========
function UploadTab() {
  const {
    requirements,
    setRequirements,
    parsedText,
    setParsedText,
    fileName,
    setFileName,
  } = useRequirements();

  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleReq = (id: number) => {
    setRequirements(
      requirements.map((r) => (r.id === id ? { ...r, checked: !r.checked } : r))
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
      // TODO: 接入 AI extract_requirements 自动提取需求项
      // 当前保留 mock 数据作为 fallback，实际接入后替换
      if (requirements.length === 0) {
        setRequirements([
          { id: 1, section: "", text: "实现基于角色的访问控制（RBAC）", category: "技术要求", mandatory: true, keywords: ["RBAC", "访问控制"], checked: true },
          { id: 2, section: "", text: "支持 LDAP/AD 域账号集成", category: "技术要求", mandatory: true, keywords: ["LDAP", "AD"], checked: true },
          { id: 3, section: "", text: "密码策略需满足等保 2.0 三级要求", category: "技术要求", mandatory: true, keywords: ["密码策略", "等保"], checked: true },
          { id: 4, section: "", text: "投标人须具备信息安全等级保护测评机构资质", category: "资质要求", mandatory: true, keywords: ["资质", "等保测评"], checked: true },
          { id: 5, section: "", text: "报价须包含三年维保费用", category: "商务要求", mandatory: true, keywords: ["维保", "报价"], checked: false },
        ]);
      }
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
          { name: "文档", extensions: ["docx", "pdf", "xlsx", "txt"] },
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

interface ParagraphInfo {
  index: number;
  text: string;
  char_offset: number;
}

function CheckTabFullScreen() {
  const { requirements } = useRequirements();
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [paragraphs, setParagraphs] = useState<ParagraphInfo[]>([]);
  const [report, setReport] = useState<DeviationReport | null>(null);
  const [fatalRisks, setFatalRisks] = useState<FatalRisk[]>([]);
  const [selfReviewReport, setSelfReviewReport] = useState<any | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [highlightedIdx, setHighlightedIdx] = useState<number | null>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const highlightTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleSelectFile = async () => {
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "文档", extensions: ["docx", "pdf", "txt"] }],
      });
      if (selected && typeof selected === "string") {
        setSelectedFile(selected);
        setFileName(selected.split("/").pop() || selected);
        const result: any = await invoke("parse_document_structured", { filePath: selected });
        setParagraphs(result.paragraphs || []);
        // 清除上次结果
        setReport(null);
        setFatalRisks([]);
        setSelfReviewReport(null);
      }
    } catch (e: any) {
      setError(`选择或解析文件失败: ${String(e)}`);
    }
  };

  const runCheck = async () => {
    if (!selectedFile || paragraphs.length === 0) {
      setError("请先选择投标文档");
      return;
    }
    if (requirements.length === 0) {
      setError("请在「需求上传」Tab 先上传招标文件并提取需求");
      return;
    }
    const bidText = paragraphs.map((p) => p.text).join("\n");

    setLoading(true);
    setError(null);
    setReport(null);
    setFatalRisks([]);
    setSelfReviewReport(null);
    try {
      const reqItems = requirements.filter((r) => r.checked);
      const [result, risks, selfReview] = await Promise.all([
        invoke<DeviationReport>("check_deviation_items", { reqItems, bidText }),
        invoke<FatalRisk[]>("check_fatal_risks_text", { text: bidText }),
        invoke<any>("check_self_review_async", { text: bidText }),
      ]);
      setReport(result);
      setFatalRisks(risks);
      setSelfReviewReport(selfReview);
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
    setReport(null);
    setFatalRisks([]);
    setSelfReviewReport(null);
    setError(null);
  };

  const jumpToParagraph = (paragraphIndex: number | undefined) => {
    if (paragraphIndex === undefined || !previewRef.current) return;
    const el = previewRef.current.querySelector(`[data-paragraph-index="${paragraphIndex}"]`);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      setHighlightedIdx(paragraphIndex);
      if (highlightTimeoutRef.current) {
        clearTimeout(highlightTimeoutRef.current);
      }
      highlightTimeoutRef.current = setTimeout(() => setHighlightedIdx(null), 2500);
    }
  };

  useEffect(() => {
    return () => {
      if (highlightTimeoutRef.current) {
        clearTimeout(highlightTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="flex-1 flex overflow-hidden">
      {/* 左侧结果面板 */}
      <div className="w-[45%] min-w-[360px] max-w-[560px] flex flex-col border-r border-gray-200 bg-white">
        <CheckResultsPanel
          requirements={requirements}
          report={report}
          fatalRisks={fatalRisks}
          selfReviewReport={selfReviewReport}
          loading={loading}
          error={error}
          onRunCheck={runCheck}
          onExportMd={exportMd}
          onClear={clear}
          onJumpToParagraph={jumpToParagraph}
        />
      </div>

      {/* 右侧文档预览区（只读） */}
      <div className="flex-1 flex flex-col min-w-0 bg-white">
        {/* 文件选择头部 */}
        <div className="shrink-0 px-4 py-3 border-b border-gray-200 bg-white flex items-center justify-between">
          <div className="flex items-center gap-2 min-w-0">
            <FileText size={16} className="text-gray-400 shrink-0" />
            <span className="text-sm font-medium text-gray-700 truncate">
              {fileName || "未选择文件"}
            </span>
          </div>
          <button
            onClick={handleSelectFile}
            className="text-xs px-3 py-1.5 rounded bg-blue-50 text-blue-600 hover:bg-blue-100 transition-colors shrink-0"
          >
            {fileName ? "重新选择" : "选择文件"}
          </button>
        </div>

        {/* 预览内容 */}
        <div ref={previewRef} className="flex-1 overflow-auto p-6 space-y-3">
          {paragraphs.length === 0 && (
            <div className="text-center text-gray-400 py-20 text-sm">
              请选择本地 .docx / .pdf / .txt 文件进行校对
            </div>
          )}
          {paragraphs.map((p) => (
            <div
              key={p.index}
              data-paragraph-index={p.index}
              className={`p-3 rounded border transition-colors duration-300 ${
                highlightedIdx === p.index
                  ? "bg-yellow-50 border-yellow-300"
                  : "border-transparent hover:bg-gray-50"
              }`}
            >
              <div className="text-xs text-gray-400 mb-1">第 {p.index + 1} 段</div>
              <p className="text-sm text-gray-700 leading-relaxed whitespace-pre-wrap">{p.text}</p>
            </div>
          ))}
        </div>
      </div>
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
