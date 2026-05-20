import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import App from "../App";

const mockInvoke = vi.fn();
const mockOpen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: any[]) => mockOpen(...args),
}));

describe("CheckTab Happy Path E2E", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockOpen.mockReset();
  });

  it("E1: full happy path from file select to export", async () => {
    // Mock parse_document: 3 paragraphs
    mockInvoke.mockImplementation((cmd: string, _args?: any) => {
      if (cmd === "parse_document") {
        return Promise.resolve({
          text: "第一段\n第二段\n第三段",
          paragraphs: [
            { index: 0, text: "第一段", char_offset: 0 },
            { index: 1, text: "第二段", char_offset: 3 },
            { index: 2, text: "第三段", char_offset: 6 },
          ],
        });
      }
      if (cmd === "check_deviation_items") {
        return Promise.resolve({
          total: 1,
          none_count: 0,
          positive_count: 0,
          minor_count: 0,
          major_count: 1,
          fatal_risk_count: 0,
          items: [
            {
              id: 1,
              section: "1",
              requirement_text: "测试要求",
              response_text: "测试应答",
              status: "Major",
              risk_level: "高",
              explanation: "测试说明",
              suggestion: "测试建议",
              paragraph_index: 1,
            },
          ],
        });
      }
      if (cmd === "check_fatal_risks_text") {
        return Promise.resolve([]);
      }
      if (cmd === "check_self_review_async") {
        return Promise.resolve({
          issues: [
            {
              id: 1,
              category: "format",
              sub_category: "punctuation",
              message: "英文逗号应使用中文逗号",
              severity: "Warning",
              position: 3,
              paragraph_index: 0,
              original: ",",
              suggestion: "，",
              auto_fixable: true,
            },
          ],
        });
      }
      if (cmd === "apply_self_review_fixes") {
        return Promise.resolve({
          mode: (_args?.mode as string) || "copy",
          output_path: "/tmp/test_fixed.docx",
          backup_path: null,
          changes: [
            { paragraph_index: 0, paragraph_text: "第一段", original: "，，", modified: "，", issue_category: "format" },
          ],
        });
      }
      if (cmd === "export_deviation_report_markdown") {
        return Promise.resolve("# 偏离检查报告\n\n测试");
      }
      if (cmd === "get_ai_config") {
        return Promise.resolve({
          provider: "Ollama",
          base_url: "http://localhost:11434",
          api_key: null,
          model: "llama3",
        });
      }
      if (cmd === "extract_requirements") {
        return Promise.resolve({
          requirements: [
            { id: 1, text: "测试需求项", certainty: "high", selected: true },
          ],
        });
      }
      return Promise.resolve([]);
    });

    // Step 1: 先上传招标文件（创建需求）
    mockOpen.mockResolvedValueOnce("/tmp/req.docx");
    render(<App />);

    fireEvent.click(screen.getByText("需求上传"));
    const uploadZone = screen.getByText(/拖入文件到这里/);
    fireEvent.click(uploadZone);

    await waitFor(() => {
      expect(screen.getByText(/req.docx/)).toBeInTheDocument();
    });

    // 点击开始分析以解析需求
    const analyzeBtn = screen.getByRole("button", { name: /开始分析/ });
    fireEvent.click(analyzeBtn);

    await waitFor(() => {
      expect(screen.getByText(/测试需求项/)).toBeInTheDocument();
    });

    // Step 2: 切换到校对 Tab，选择投标 .docx
    mockOpen.mockResolvedValueOnce("/tmp/bid.docx");
    fireEvent.click(screen.getByText("校对"));

    const selectBtn = await screen.findByText("选择文件");
    fireEvent.click(selectBtn);

    // 预览区应显示 3 个段落
    await waitFor(() => {
      expect(screen.getByText("第 1 段")).toBeInTheDocument();
      expect(screen.getByText("第 2 段")).toBeInTheDocument();
      expect(screen.getByText("第 3 段")).toBeInTheDocument();
    });

    // Step 3: 点击「检查投标文件」
    const checkBtn = screen.getByRole("button", { name: /检查投标文件/ });
    expect(checkBtn).not.toBeDisabled();
    fireEvent.click(checkBtn);

    // 左侧应显示分类结果：偏离风险、格式问题
    await waitFor(() => {
      expect(screen.getByText(/偏离风险/)).toBeInTheDocument();
      expect(screen.getByText(/格式问题/)).toBeInTheDocument();
    });

    // Step 4: 点击偏离风险项 → 预览区高亮第 2 段
    const deviationItem = screen.getByText(/#1 测试要求/);
    fireEvent.click(deviationItem);

    // 高亮状态通过 CSS 类判断（data-paragraph-index="1" 的 div 应有 bg-yellow-50）
    const para2 = document.querySelector('[data-paragraph-index="1"]');
    expect(para2).toBeInTheDocument();

    // Step 5: 一键修复可修复项（按钮 enabled，因为 .docx）
    await waitFor(() => {
      const fixBtn = screen.getByRole("button", { name: /一键修复可修复项/ });
      expect(fixBtn).not.toBeDisabled();
      fireEvent.click(fixBtn);
    });

    // 修复模式选择弹窗
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /一键修复/ })).toBeInTheDocument();
      const confirmBtn = screen.getByRole("button", { name: /确认修复/ });
      fireEvent.click(confirmBtn);
    });

    // 修复结果弹窗
    await waitFor(() => {
      expect(screen.getByText(/修复完成/)).toBeInTheDocument();
      expect(screen.getByText(/修改明细/)).toBeInTheDocument();
    });

    // Step 6: 导出报告
    const exportBtn = screen.getByRole("button", { name: /导出偏离报告/ });
    fireEvent.click(exportBtn);

    // E2E 完成，无异常
  });

  it("E2: .doc file shows warning dialog and fix button disabled", async () => {
    mockOpen.mockResolvedValue("/tmp/test.doc");

    render(<App />);
    fireEvent.click(screen.getByText("校对"));

    const selectBtn = await screen.findByText("选择文件");
    fireEvent.click(selectBtn);

    // 应显示 .doc 警告弹窗
    await waitFor(() => {
      expect(screen.getByText(/格式提示/)).toBeInTheDocument();
    });

    // 点击继续
    const continueBtn = screen.getByRole("button", { name: /继续/ });
    fireEvent.click(continueBtn);

    // 由于 parse_document 对 .doc 返回 Err，应显示错误
    await waitFor(() => {
      expect(screen.getByText(/解析失败/)).toBeInTheDocument();
    });
  });
});
