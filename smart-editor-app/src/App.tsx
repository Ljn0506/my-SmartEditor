import { useState } from "react";
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

  const handleCopy = async () => {
    if (!editor) return;
    const html = editor.getHTML();
    // TODO: 通过 Tauri 剪贴板 API 写入系统剪贴板
    console.log("Copy HTML:", html.substring(0, 200) + "...");
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
            {activeTab === "library" && <LibraryTab />}
            {activeTab === "push" && <PushTab />}
            {activeTab === "check" && <CheckTab />}
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

// ========== 模板库标签页 ==========
function LibraryTab() {
  const [activeFilter, setActiveFilter] = useState("全部");
  const filters = ["全部", "投标应答", "技术方案", "实施方案", "合同协议"];

  const templates = [
    {
      title: "等保 2.0 三级通用技术方案",
      domain: "网络安全",
      module: "技术方案",
      phase: "投标",
      tags: ["#等保2.0", "#三级", "#通用要求"],
    },
    {
      title: "RBAC 权限管理设计方案",
      domain: "应用安全",
      module: "技术方案",
      phase: "方案",
      tags: [],
    },
    {
      title: "渗透测试服务投标文件",
      domain: "网络安全",
      module: "偏离说明",
      phase: "投标",
      tags: [],
    },
  ];

  return (
    <div className="h-full flex">
      {/* 左侧筛选 */}
      <aside className="w-56 bg-white border-r border-gray-200 p-4 overflow-auto">
        <h3 className="font-semibold text-sm mb-3">文档属性</h3>
        <div className="space-y-1 mb-6">
          {filters.map((f) => (
            <button
              key={f}
              onClick={() => setActiveFilter(f)}
              className={`w-full text-left px-3 py-1.5 rounded text-sm ${
                activeFilter === f
                  ? "bg-blue-50 text-blue-600 font-medium"
                  : "text-gray-600 hover:bg-gray-50"
              }`}
            >
              {activeFilter === f ? "●" : "○"} {f}
            </button>
          ))}
        </div>
        <h3 className="font-semibold text-sm mb-3">业务领域</h3>
        <div className="space-y-1 mb-6 text-sm text-gray-600">
          <div className="px-3 py-1">● 全部</div>
          <div className="px-3 py-1">○ 网络安全</div>
          <div className="px-3 py-1">○ 应用安全</div>
          <div className="px-3 py-1">○ 数据安全</div>
        </div>
        <h3 className="font-semibold text-sm mb-3">内容模块</h3>
        <div className="space-y-1 text-sm text-gray-600">
          <div className="px-3 py-1">● 全部</div>
          <div className="px-3 py-1">○ 技术方案</div>
          <div className="px-3 py-1">○ 商务条款</div>
          <div className="px-3 py-1">○ 偏离说明</div>
        </div>
      </aside>

      {/* 右侧模板列表 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="flex gap-2 mb-4">
          <input
            type="text"
            placeholder="搜索模板..."
            className="flex-1 px-4 py-2 rounded-lg border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
            搜索
          </button>
        </div>
        <div className="space-y-3">
          {templates.map((t, i) => (
            <div
              key={i}
              className="bg-white rounded-lg border border-gray-200 p-4 hover:shadow-md transition-shadow"
            >
              <div className="font-medium mb-1">{t.title}</div>
              <div className="text-sm text-gray-500 mb-2">
                领域: {t.domain} · 模块: {t.module} · 阶段: {t.phase}
              </div>
              <div className="flex items-center gap-2">
                {t.tags.map((tag) => (
                  <span
                    key={tag}
                    className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600"
                  >
                    {tag}
                  </span>
                ))}
              </div>
              <div className="mt-3 flex gap-2">
                <button className="text-xs px-3 py-1 rounded border border-gray-200 hover:bg-gray-50">
                  预览
                </button>
                <button className="text-xs px-3 py-1 rounded border border-gray-200 hover:bg-gray-50">
                  插入
                </button>
                <button className="text-xs px-3 py-1 rounded border border-gray-200 hover:bg-gray-50">
                  收藏
                </button>
              </div>
            </div>
          ))}
        </div>
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

// ========== 校对面板标签页 ==========
function CheckTab() {
  return (
    <div className="h-full p-6 overflow-auto">
      <h2 className="text-base font-semibold mb-4 flex items-center gap-2">
        <SearchCheck size={18} />
        文档校对
      </h2>
      <div className="flex gap-2 mb-4">
        {["全部", "偏离项", "废标风险", "格式", "术语", "数据一致性"].map(
          (f, i) => (
            <button
              key={f}
              className={`px-3 py-1.5 rounded-md text-sm ${
                i === 0
                  ? "bg-blue-50 text-blue-600 font-medium"
                  : "text-gray-600 hover:bg-gray-100"
              }`}
            >
              {f}
            </button>
          )
        )}
      </div>
      <div className="space-y-4">
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="text-sm font-medium text-amber-600 mb-2">⚠️ 偏离项 (2)</h3>
          <div className="space-y-2 text-sm">
            <div className="flex items-start gap-2 p-2 bg-amber-50 rounded">
              <input type="checkbox" className="mt-0.5" />
              <div>
                <div>3.2 节：缺少等保三级要求说明（招标文件 2.3 节强制要求）</div>
                <div className="text-xs text-gray-500 mt-1">
                  风险等级：高 [定位] [建议修复]
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="text-sm font-medium text-red-600 mb-2">❌ 废标风险项 (1)</h3>
          <div className="space-y-2 text-sm">
            <div className="flex items-start gap-2 p-2 bg-red-50 rounded">
              <input type="checkbox" className="mt-0.5" />
              <div>
                <div>缺少法人代表签字页（招标文件 P15 明确要求）</div>
                <div className="text-xs text-gray-500 mt-1">
                  风险等级：致命 [定位]
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="mt-4 flex gap-3">
        <button className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm hover:bg-blue-700">
          一键修复所有格式问题
        </button>
        <button className="px-4 py-2 border border-gray-200 rounded-lg text-sm hover:bg-gray-50">
          导出校对报告
        </button>
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
