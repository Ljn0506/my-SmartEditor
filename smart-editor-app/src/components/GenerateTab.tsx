import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Sparkles,
  FileText,
  CheckCircle2,
  Circle,
  XCircle,
  Loader2,
  Save,
  Wand2,
  Play,
  SearchCheck,
  ArrowRight,
  RefreshCw,
} from "lucide-react";
import { useRequirements } from "../contexts/RequirementsContext";

export interface CardOutline {
  id: string;
  chapter: string;
  title: string;
  document_target: string;
}

export interface ParamPlaceholder {
  key: string;
  label: string;
  default_value?: string;
}

export interface Card {
  id: string;
  chapter: string;
  title: string;
  content: string;
  source_refs: string[];
  document_target: string;
  status: "draft" | "confirmed" | "rejected";
  generated_by: string;
  related_cards: string[];
  param_placeholders?: ParamPlaceholder[];
}

export interface GenerateTabProps {
  onNavigateToCheck?: () => void;
}

export default function GenerateTab({ onNavigateToCheck }: GenerateTabProps) {
  const { requirements, parsedText, fileName } = useRequirements();
  const [docTarget, setDocTarget] = useState<"technical" | "business">("technical");
  const [outlines, setOutlines] = useState<CardOutline[]>([]);
  const [cards, setCards] = useState<Card[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [generatingOutline, setGeneratingOutline] = useState(false);
  const [generatingCardId, setGeneratingCardId] = useState<string | null>(null);
  const [generatingAll, setGeneratingAll] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editContent, setEditContent] = useState<string>("");
  const [consistencyReport, setConsistencyReport] = useState<any>(null);
  const [checkingConsistency, setCheckingConsistency] = useState(false);
  const [showConsistency, setShowConsistency] = useState(false);

  const hasRequirements = requirements.length > 0 && parsedText.length > 0;
  const selectedCard = cards.find((c) => c.id === selectedId);

  // 当选中卡片变化时，同步编辑内容
  useEffect(() => {
    if (selectedCard) {
      setEditContent(selectedCard.content);
    } else {
      setEditContent("");
    }
  }, [selectedCard?.id]);

  const handleGenerateOutline = async () => {
    if (!hasRequirements) return;
    setGeneratingOutline(true);
    setError(null);
    try {
      const result: CardOutline[] = await invoke("generate_outline", {
        requirementsText: parsedText,
        docType: docTarget === "technical" ? "Technical" : "Business",
      });
      setOutlines(result);
      // 初始化为空卡片（仅大纲）
      const emptyCards: Card[] = result.map((o) => ({
        ...o,
        content: "",
        source_refs: [],
        status: "draft",
        generated_by: "ai",
        related_cards: [],
      }));
      setCards(emptyCards);
      setSelectedId(emptyCards[0]?.id || null);
      // 保存到后端
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: emptyCards,
      });
    } catch (e: any) {
      setError(`生成大纲失败: ${String(e)}`);
    } finally {
      setGeneratingOutline(false);
    }
  };

  const handleGenerateCard = async (outline: CardOutline) => {
    if (!hasRequirements) return;
    setGeneratingCardId(outline.id);
    setError(null);
    try {
      const card: Card = await invoke("generate_card", {
        outline,
        requirementsText: parsedText,
        references: [],
        docType: docTarget === "technical" ? "Technical" : "Business",
      });
      setCards((prev) =>
        prev.map((c) => (c.id === card.id ? card : c))
      );
      // 同步更新后端
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: cards.map((c) => (c.id === card.id ? card : c)),
      });
    } catch (e: any) {
      setError(`生成内容失败: ${String(e)}`);
    } finally {
      setGeneratingCardId(null);
    }
  };

  const handleGenerateAll = async () => {
    if (!hasRequirements || outlines.length === 0) return;
    setGeneratingAll(true);
    setError(null);
    try {
      let updatedCards = [...cards];
      for (const outline of outlines) {
        const card: Card = await invoke("generate_card", {
          outline,
          requirementsText: parsedText,
          references: [],
          docType: docTarget === "technical" ? "Technical" : "Business",
        });
        updatedCards = updatedCards.map((c) => (c.id === card.id ? card : c));
        setCards(updatedCards);
      }
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: updatedCards,
      });
    } catch (e: any) {
      setError(`批量生成失败: ${String(e)}`);
    } finally {
      setGeneratingAll(false);
    }
  };

  const handleSaveEdit = async () => {
    if (!selectedCard) return;
    const updated: Card = { ...selectedCard, content: editContent };
    const next = cards.map((c) => (c.id === updated.id ? updated : c));
    setCards(next);
    try {
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: next,
      });
    } catch (e: any) {
      setError(`保存失败: ${String(e)}`);
    }
  };

  const handleConfirmCard = async (card: Card) => {
    const updated: Card = { ...card, status: "confirmed" };
    const next = cards.map((c) => (c.id === updated.id ? updated : c));
    setCards(next);
    try {
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: next,
      });
    } catch (e: any) {
      setError(`确认失败: ${String(e)}`);
    }
  };

  const handleRegenerate = async () => {
    if (!selectedCard || !hasRequirements) return;
    // 若已编辑，先确认
    if (editContent !== selectedCard.content) {
      if (!confirm("已编辑内容将被覆盖，是否继续？")) {
        return;
      }
    }
    const outline = outlines.find((o) => o.id === selectedCard.id);
    if (!outline) return;
    setGeneratingCardId(selectedCard.id);
    setError(null);
    try {
      const card: Card = await invoke("generate_card", {
        outline,
        requirementsText: parsedText,
        references: [],
        docType: docTarget === "technical" ? "Technical" : "Business",
      });
      const next = cards.map((c) => (c.id === card.id ? card : c));
      setCards(next);
      setEditContent(card.content);
      await invoke("save_cards", {
        documentTarget: docTarget,
        cards: next,
      });
    } catch (e: any) {
      setError(`换素材生成失败: ${String(e)}`);
    } finally {
      setGeneratingCardId(null);
    }
  };

  const handleCheckConsistency = async () => {
    if (cards.length === 0) return;
    setCheckingConsistency(true);
    setError(null);
    try {
      const report = await invoke("check_consistency", { cards });
      setConsistencyReport(report);
      setShowConsistency(true);
    } catch (e: any) {
      setError(`一致性检查失败: ${String(e)}`);
    } finally {
      setCheckingConsistency(false);
    }
  };

  const handleConfirmAll = async () => {
    try {
      await invoke("confirm_all_cards", { documentTarget: docTarget });
      const next = cards.map((c) =>
        c.status === "draft" ? { ...c, status: "confirmed" as const } : c
      );
      setCards(next);
    } catch (e: any) {
      setError(`一键确认失败: ${String(e)}`);
    }
  };

  const statusIcon = (status: Card["status"]) => {
    switch (status) {
      case "confirmed":
        return <CheckCircle2 size={14} className="text-emerald-500" />;
      case "rejected":
        return <XCircle size={14} className="text-red-500" />;
      default:
        return <Circle size={14} className="text-gray-400" />;
    }
  };

  const statusLabel = (status: Card["status"]) => {
    switch (status) {
      case "confirmed":
        return "已确认";
      case "rejected":
        return "已驳回";
      default:
        return "草稿";
    }
  };

  return (
    <div className="flex-1 flex overflow-hidden">
      {/* 左侧卡片列表 */}
      <div className="w-[45%] min-w-[360px] max-w-[560px] flex flex-col border-r border-gray-200 bg-white">
        {/* 头部 */}
        <div className="shrink-0 px-4 py-3 border-b border-gray-200 bg-white">
          <h2 className="text-base font-semibold mb-3 flex items-center gap-2">
            <Sparkles size={18} />
            智能生成
          </h2>

          {/* 文档类型切换 */}
          <div className="flex bg-gray-100 rounded-lg p-1 mb-3">
            <button
              onClick={() => setDocTarget("technical")}
              className={`flex-1 py-1.5 text-sm rounded-md font-medium transition-colors ${
                docTarget === "technical"
                  ? "bg-white text-blue-600 shadow-sm"
                  : "text-gray-600 hover:text-gray-800"
              }`}
            >
              技术方案
            </button>
            <button
              onClick={() => setDocTarget("business")}
              className={`flex-1 py-1.5 text-sm rounded-md font-medium transition-colors ${
                docTarget === "business"
                  ? "bg-white text-blue-600 shadow-sm"
                  : "text-gray-600 hover:text-gray-800"
              }`}
            >
              商务响应
            </button>
          </div>

          {/* 需求来源 */}
          <div className="text-xs text-gray-500 mb-3">
            {hasRequirements ? (
              <span className="flex items-center gap-1">
                <FileText size={12} />
                需求来源: {fileName || "已解析"} ({requirements.length} 项需求)
              </span>
            ) : (
              <span className="text-amber-600">
                请在「需求上传」Tab 先上传招标文件
              </span>
            )}
          </div>

          {/* 生成大纲按钮 */}
          <button
            onClick={handleGenerateOutline}
            disabled={!hasRequirements || generatingOutline}
            className="w-full py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:bg-gray-300 transition-colors flex items-center justify-center gap-2"
          >
            {generatingOutline ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <Wand2 size={16} />
            )}
            {generatingOutline ? "生成中..." : "生成章节大纲"}
          </button>
        </div>

        {/* 卡片列表 */}
        <div className="flex-1 overflow-auto p-3 space-y-2">
          {cards.length === 0 && (
            <div className="text-center text-gray-400 py-10 text-sm">
              点击「生成章节大纲」开始创作
            </div>
          )}
          {cards.map((card) => {
            const isSelected = selectedId === card.id;
            const hasContent = card.content.length > 0;
            const materialShort = card.source_refs.length <= 1;
            return (
              <div
                key={card.id}
                onClick={() => setSelectedId(card.id)}
                className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                  isSelected
                    ? "bg-blue-50 border-blue-300"
                    : "bg-white border-gray-200 hover:border-gray-300"
                }`}
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs font-mono text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded">
                        {card.chapter}
                      </span>
                      <span className="text-sm font-medium text-gray-800 truncate">
                        {card.title}
                      </span>
                      {materialShort && hasContent && (
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-red-50 text-red-600 font-medium shrink-0">
                          素材不足
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2 mb-1">
                      {statusIcon(card.status)}
                      <span className="text-xs text-gray-500">
                        {statusLabel(card.status)}
                      </span>
                      {!hasContent && (
                        <span className="text-xs text-amber-600">待生成</span>
                      )}
                    </div>
                    {card.source_refs.length > 0 && (
                      <div className="text-[10px] text-gray-400 truncate">
                        来源: {card.source_refs.join(", ")}
                      </div>
                    )}
                  </div>
                  {card.status === "draft" && hasContent && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleConfirmCard(card);
                      }}
                      className="text-xs px-2 py-1 rounded bg-emerald-50 text-emerald-600 hover:bg-emerald-100 shrink-0"
                    >
                      确认
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {/* 底部操作 */}
        {cards.length > 0 && (
          <div className="shrink-0 px-4 py-3 border-t border-gray-200 bg-gray-50 space-y-2">
            <button
              onClick={handleGenerateAll}
              disabled={generatingAll}
              className="w-full py-2 bg-white border border-gray-200 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-50 disabled:bg-gray-100 transition-colors flex items-center justify-center gap-2"
            >
              {generatingAll ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Play size={14} />
              )}
              {generatingAll ? "生成中..." : "一键生成全部章节"}
            </button>
            <button
              onClick={handleCheckConsistency}
              disabled={checkingConsistency || cards.length === 0}
              className="w-full py-2 bg-white border border-gray-200 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-50 disabled:bg-gray-100 transition-colors flex items-center justify-center gap-2"
            >
              {checkingConsistency ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <SearchCheck size={14} />
              )}
              {checkingConsistency ? "检查中..." : "一致性检查"}
            </button>
            <button
              onClick={handleConfirmAll}
              disabled={cards.every((c) => c.status !== "draft")}
              className="w-full py-2 bg-white border border-gray-200 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-50 disabled:bg-gray-100 transition-colors flex items-center justify-center gap-2"
            >
              <CheckCircle2 size={14} />
              一键确认所有草稿
            </button>
            {onNavigateToCheck && cards.some((c) => c.status === "confirmed") && (
              <button
                onClick={onNavigateToCheck}
                className="w-full py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors flex items-center justify-center gap-2"
              >
                <ArrowRight size={14} />
                进入校对
              </button>
            )}
          </div>
        )}
      </div>

      {/* 右侧内容编辑器 */}
      <div className="flex-1 flex flex-col min-w-0 bg-white">
        {selectedCard ? (
          <>
            {/* 编辑器头部 */}
            <div className="shrink-0 px-4 py-3 border-b border-gray-200 bg-white flex items-center justify-between">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-0.5">
                  <span className="text-xs font-mono text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded">
                    {selectedCard.chapter}
                  </span>
                  <span className="text-sm font-medium text-gray-700 truncate">
                    {selectedCard.title}
                  </span>
                </div>
                {selectedCard.source_refs.length > 0 && (
                  <div className="text-[10px] text-gray-400 truncate">
                    来源: {selectedCard.source_refs.join(", ")}
                  </div>
                )}
              </div>
              <div className="flex items-center gap-2 shrink-0">
                {selectedCard.content && (
                  <button
                    onClick={handleRegenerate}
                    disabled={generatingCardId === selectedCard.id}
                    className="text-xs px-3 py-1.5 rounded bg-amber-50 text-amber-700 hover:bg-amber-100 disabled:bg-gray-100 flex items-center gap-1"
                  >
                    {generatingCardId === selectedCard.id ? (
                      <Loader2 size={12} className="animate-spin" />
                    ) : (
                      <RefreshCw size={12} />
                    )}
                    换素材
                  </button>
                )}
                {!selectedCard.content && (
                  <button
                    onClick={() => handleGenerateCard(outlines.find((o) => o.id === selectedCard.id)!)}
                    disabled={generatingCardId === selectedCard.id}
                    className="text-xs px-3 py-1.5 rounded bg-blue-50 text-blue-600 hover:bg-blue-100 disabled:bg-gray-100 flex items-center gap-1"
                  >
                    {generatingCardId === selectedCard.id ? (
                      <Loader2 size={12} className="animate-spin" />
                    ) : (
                      <Sparkles size={12} />
                    )}
                    生成内容
                  </button>
                )}
                <button
                  onClick={handleSaveEdit}
                  className="text-xs px-3 py-1.5 rounded bg-gray-100 text-gray-700 hover:bg-gray-200 flex items-center gap-1"
                >
                  <Save size={12} />
                  保存
                </button>
              </div>
            </div>

            {/* 编辑区 */}
            <div className="flex-1 overflow-auto p-4 space-y-3">
              <textarea
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                className="w-full min-h-[400px] p-4 text-sm text-gray-800 leading-relaxed bg-gray-50 rounded-lg border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-200 resize-none font-mono"
                placeholder={selectedCard.content ? "" : "点击「生成内容」让 AI 撰写本章..."}
              />
              {selectedCard.param_placeholders && selectedCard.param_placeholders.length > 0 && (
                <div className="bg-amber-50 border border-amber-200 rounded-lg p-3">
                  <p className="text-xs font-medium text-amber-800 mb-2">⚠️ 检测到参数占位符，请替换为真实值：</p>
                  <div className="flex flex-wrap gap-2">
                    {selectedCard.param_placeholders.map((p) => (
                      <span
                        key={p.key}
                        className="text-xs px-2 py-1 rounded bg-amber-100 text-amber-700 font-mono"
                        title={p.label}
                      >
                        [PARAM:{p.key}]
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">
            请在左侧选择章节进行编辑
          </div>
        )}
      </div>

      {/* 一致性检查结果弹窗 */}
      {showConsistency && consistencyReport && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-[520px] max-h-[80vh] overflow-auto shadow-xl">
            <h3 className="text-base font-semibold mb-3">一致性检查结果</h3>
            <p className="text-sm text-gray-600 mb-3">
              共检查 {consistencyReport.total_checked} 张卡片
              {consistencyReport.issues?.length > 0
                ? `，发现 ${consistencyReport.issues.length} 处不一致`
                : "，未发现不一致"}
            </p>
            {consistencyReport.issues?.length > 0 && (
              <div className="space-y-2 mb-4">
                {consistencyReport.issues.map((issue: any, idx: number) => (
                  <div
                    key={idx}
                    className={`p-3 rounded-lg border ${
                      issue.severity === "Error"
                        ? "bg-red-50 border-red-200"
                        : "bg-amber-50 border-amber-200"
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span className={`text-xs px-1.5 py-0.5 rounded font-medium ${
                        issue.severity === "Error"
                          ? "bg-red-100 text-red-700"
                          : "bg-amber-100 text-amber-700"
                      }`}>
                        {issue.severity === "Error" ? "严重" : "警告"}
                      </span>
                      <span className="text-sm font-medium text-gray-800">{issue.parameter}</span>
                    </div>
                    <div className="text-xs text-gray-600 space-y-0.5">
                      <p>期望: {issue.expected_value}</p>
                      <p>实际: {issue.actual_value}</p>
                      <p className="text-gray-400">位置: {issue.location}</p>
                    </div>
                  </div>
                ))}
              </div>
            )}
            <div className="flex justify-end">
              <button
                onClick={() => setShowConsistency(false)}
                className="px-4 py-2 rounded text-sm text-gray-600 hover:bg-gray-100"
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 错误提示 */}
      {error && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 rounded-lg bg-red-600 text-white text-sm shadow-lg z-50">
          {error}
        </div>
      )}
    </div>
  );
}
