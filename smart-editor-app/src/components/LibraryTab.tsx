import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Editor } from "@tiptap/core";
import {
  Search,
  FileText,
  Eye,
  Plus,
  Star,
  X,
  Loader2,
  Filter,
} from "lucide-react";

type DocAttr = "投标应答" | "技术方案" | "实施方案" | "合同协议" | "汇报材料";
type BusinessDomain = "网络安全" | "应用安全" | "数据安全" | "安全运营" | "安全管理";
type ContentModule = "技术方案" | "商务条款" | "实施计划" | "资质证明" | "偏离说明" | "案例介绍";
type ProjectPhase = "方案阶段" | "投标阶段" | "合同阶段";

interface Template {
  id?: number | string;
  title: string;
  content: string;
  content_html?: string;
  doc_attr?: DocAttr;
  business_domain?: BusinessDomain;
  security_layer?: string;
  content_module?: ContentModule;
  project_phase?: ProjectPhase;
  tags: string[];
  use_count: number;
  rating: number;
}

interface Filters {
  doc_attr: DocAttr | "全部";
  business_domain: BusinessDomain | "全部";
  content_module: ContentModule | "全部";
  project_phase: ProjectPhase | "全部";
}

const dimensions = [
  {
    key: "doc_attr" as const,
    label: "文档属性",
    options: ["全部", "投标应答", "技术方案", "实施方案", "合同协议", "汇报材料"],
  },
  {
    key: "business_domain" as const,
    label: "业务领域",
    options: ["全部", "网络安全", "应用安全", "数据安全", "安全运营", "安全管理"],
  },
  {
    key: "content_module" as const,
    label: "内容模块",
    options: [
      "全部",
      "技术方案",
      "商务条款",
      "实施计划",
      "资质证明",
      "偏离说明",
      "案例介绍",
    ],
  },
  {
    key: "project_phase" as const,
    label: "项目阶段",
    options: ["全部", "方案阶段", "投标阶段", "合同阶段"],
  },
];

export default function LibraryTab({ editor }: { editor: Editor | null }) {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>({
    doc_attr: "全部",
    business_domain: "全部",
    content_module: "全部",
    project_phase: "全部",
  });
  const [keyword, setKeyword] = useState("");
  const [preview, setPreview] = useState<Template | null>(null);
  const [favorites, setFavorites] = useState<Set<string | number>>(new Set());
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

  const fetchTemplates = useCallback(
    async (searchKeyword?: string) => {
      setLoading(true);
      setError(null);
      try {
        const dbFilters: Record<string, string | null> = {};
        dimensions.forEach((d) => {
          const val = filters[d.key];
          dbFilters[d.key] = val === "全部" ? null : val;
        });

        let result: Template[];
        if (searchKeyword && searchKeyword.trim()) {
          // 先尝试 Meilisearch，失败降级到 SQLite
          try {
            const meiliResult: any[] = await invoke("search_templates_meili", {
              query: searchKeyword.trim(),
              limit: 50,
            });
            result = meiliResult.map((r) => ({
              id: r.id,
              title: r.title,
              content: r.content,
              doc_attr: r.doc_attr,
              business_domain: r.business_domain,
              content_module: r.content_module,
              project_phase: r.project_phase,
              tags: r.tags,
              use_count: 0,
              rating: 0,
            }));
          } catch (err) {
            console.warn("Meilisearch failed, falling back to SQLite:", err);
            result = await invoke("search_templates_db", {
              ...dbFilters,
              keyword: searchKeyword.trim() || null,
              limit: 50,
            });
          }
        } else {
          result = await invoke("search_templates_db", {
            ...dbFilters,
            keyword: null,
            limit: 100,
          });
        }
        setTemplates(result);
      } catch (e: any) {
        setError(String(e));
        setTemplates([]);
      } finally {
        setLoading(false);
      }
    },
    [filters]
  );

  // 初始加载 + 筛选变化时触发
  useEffect(() => {
    fetchTemplates(keyword);
  }, [filters, fetchTemplates]);

  // 搜索框 debounce
  const handleSearchChange = (val: string) => {
    setKeyword(val);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      fetchTemplates(val);
    }, 300);
  };

  const handleInsert = (t: Template) => {
    if (!editor) return;
    const html = t.content_html || `<p>${t.content}</p>`;
    editor.chain().focus().insertContent(html).run();
  };

  const toggleFavorite = (id?: string | number) => {
    if (id === undefined || id === null) return;
    setFavorites((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div className="h-full flex">
      {/* 左侧筛选 */}
      <aside className="w-56 bg-white border-r border-gray-200 p-4 overflow-auto shrink-0">
        <h3 className="font-semibold text-sm mb-3 flex items-center gap-1">
          <Filter size={14} />
          四维筛选
        </h3>
        {dimensions.map((dim) => (
          <div key={dim.key} className="mb-5">
            <div className="text-xs text-gray-500 mb-1.5">{dim.label}</div>
            <div className="space-y-1">
              {dim.options.map((opt) => {
                const active = filters[dim.key] === opt;
                return (
                  <button
                    key={opt}
                    onClick={() =>
                      setFilters((prev) => ({ ...prev, [dim.key]: opt }))
                    }
                    aria-pressed={active}
                    className={`w-full text-left px-2.5 py-1 rounded text-sm transition-colors ${
                      active
                        ? "bg-blue-50 text-blue-600 font-medium"
                        : "text-gray-600 hover:bg-gray-50"
                    }`}
                  >
                    {active ? "●" : "○"} {opt}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </aside>

      {/* 右侧内容 */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* 搜索栏 */}
        <div className="shrink-0 p-4 border-b border-gray-200 bg-white">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search
                size={16}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
              />
              <input
                type="text"
                value={keyword}
                onChange={(e) => handleSearchChange(e.target.value)}
                placeholder="搜索标题、内容、标签..."
                aria-label="搜索模板"
                className="w-full pl-9 pr-4 py-2 rounded-lg border border-gray-200 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
          {error && (
            <div className="mt-2 text-xs text-red-600 bg-red-50 rounded px-3 py-2">
              {error}
            </div>
          )}
        </div>

        {/* 列表区 */}
        <div className="flex-1 overflow-auto p-4">
          {loading && templates.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-400 text-sm">
              <Loader2 size={18} className="animate-spin mr-2" />
              加载中...
            </div>
          ) : templates.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-400 text-sm">
              <FileText size={32} className="mb-2" />
              暂无模板，请调整筛选条件或搜索关键词
            </div>
          ) : (
            <div className="space-y-3">
              {templates.map((t, i) => (
                <div
                  key={t.id ?? i}
                  className="bg-white rounded-lg border border-gray-200 p-4 hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex-1 min-w-0">
                      <div className="font-medium mb-1 truncate">{t.title}</div>
                      <div className="text-xs text-gray-500 mb-2 flex flex-wrap gap-x-3 gap-y-1">
                        {t.doc_attr && <span>属性: {t.doc_attr}</span>}
                        {t.business_domain && (
                          <span>领域: {t.business_domain}</span>
                        )}
                        {t.content_module && (
                          <span>模块: {t.content_module}</span>
                        )}
                        {t.project_phase && <span>阶段: {t.project_phase}</span>}
                      </div>
                      <div className="flex flex-wrap gap-1.5">
                        {t.tags.map((tag) => (
                          <span
                            key={tag}
                            className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    </div>
                    <div className="shrink-0 flex items-center gap-1">
                      <button
                        onClick={() => toggleFavorite(t.id)}
                        title="收藏"
                        className={`p-1.5 rounded hover:bg-gray-100 ${
                          t.id && favorites.has(t.id)
                            ? "text-amber-500"
                            : "text-gray-400"
                        }`}
                      >
                        <Star
                          size={16}
                          fill={
                            t.id && favorites.has(t.id)
                              ? "currentColor"
                              : "none"
                          }
                        />
                      </button>
                    </div>
                  </div>
                  <div className="mt-3 flex gap-2">
                    <button
                      onClick={() => setPreview(t)}
                      className="text-xs px-3 py-1.5 rounded border border-gray-200 hover:bg-gray-50 flex items-center gap-1"
                    >
                      <Eye size={14} />
                      预览
                    </button>
                    <button
                      onClick={() => handleInsert(t)}
                      disabled={!editor}
                      className="text-xs px-3 py-1.5 rounded bg-blue-50 text-blue-600 hover:bg-blue-100 flex items-center gap-1 disabled:opacity-50"
                    >
                      <Plus size={14} />
                      插入
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* 预览弹窗 */}
      {preview && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl w-[640px] max-h-[80vh] flex flex-col">
            <div className="px-5 py-3 border-b border-gray-200 flex items-center justify-between">
              <h3 className="font-semibold text-sm">{preview.title}</h3>
              <button
                onClick={() => setPreview(null)}
                className="p-1 rounded hover:bg-gray-100 text-gray-500"
              >
                <X size={18} />
              </button>
            </div>
            <div className="flex-1 overflow-auto p-5 text-sm leading-relaxed">
              <div
                dangerouslySetInnerHTML={{
                  __html:
                    preview.content_html ||
                    `<p>${preview.content
                      .replace(/&/g, "&amp;")
                      .replace(/</g, "&lt;")
                      .replace(/>/g, "&gt;")
                      .replace(/\n/g, "<br/>")}</p>`,
                }}
              />
            </div>
            <div className="px-5 py-3 border-t border-gray-200 flex justify-end gap-2">
              <button
                onClick={() => setPreview(null)}
                className="px-4 py-2 rounded border border-gray-200 text-sm hover:bg-gray-50"
              >
                关闭
              </button>
              <button
                onClick={() => {
                  handleInsert(preview);
                  setPreview(null);
                }}
                disabled={!editor}
                className="px-4 py-2 rounded bg-blue-600 text-white text-sm hover:bg-blue-700 disabled:opacity-50"
              >
                插入到编辑器
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
