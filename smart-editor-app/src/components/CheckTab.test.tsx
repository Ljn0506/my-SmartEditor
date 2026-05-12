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

describe("CheckTab (ReviewTab)", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockOpen.mockReset();
  });

  // F4: requirements 为空时「检查投标文件」按钮 disabled + 显示提示
  it("disables check button and shows hint when no requirements", () => {
    render(<App />);
    fireEvent.click(screen.getByText("校对"));

    const checkBtn = screen.getByRole("button", { name: /检查投标文件/ });
    expect(checkBtn).toBeDisabled();
    expect(screen.getByText(/请先前往「需求上传」/)).toBeInTheDocument();
  });

  // F4: 有 requirements 后按钮 enabled
  it("enables check button when requirements exist", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "parse_document") {
        return Promise.resolve({
          text: "招标要求：投标人必须具备法人资格。",
          paragraphs: [{ index: 0, text: "招标要求：投标人必须具备法人资格。", char_offset: 0 }],
        });
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
            { id: 1, text: "具备法人资格", certainty: "high", selected: true },
          ],
        });
      }
      return Promise.resolve([]);
    });
    mockOpen.mockResolvedValue("/tmp/test.docx");

    render(<App />);

    // 先上传招标文件
    fireEvent.click(screen.getByText("需求上传"));
    const uploadZone = screen.getByText(/拖入文件到这里/);
    fireEvent.click(uploadZone);

    await waitFor(() => {
      expect(screen.getByText(/test.docx/)).toBeInTheDocument();
    });

    // 切换到校对 Tab
    fireEvent.click(screen.getByText("校对"));
    await waitFor(() => {
      const checkBtn = screen.getByRole("button", { name: /检查投标文件/ });
      expect(checkBtn).not.toBeDisabled();
    });
  });

  // F5: 切换 Tab 后再切回，CheckTab 状态保留（Context 管理）
  it("retains check state when switching tabs", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "parse_document") {
        return Promise.resolve({
          text: "测试段落。",
          paragraphs: [{ index: 0, text: "测试段落。", char_offset: 0 }],
        });
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
            { id: 1, text: "测试需求", certainty: "high", selected: true },
          ],
        });
      }
      return Promise.resolve([]);
    });
    mockOpen.mockResolvedValue("/tmp/test.docx");

    render(<App />);

    // 切换到校对 Tab，选择文件
    fireEvent.click(screen.getByText("校对"));
    const selectBtn = await screen.findByText("选择文件");
    fireEvent.click(selectBtn);

    await waitFor(() => {
      expect(screen.getByText(/test.docx/)).toBeInTheDocument();
    });

    // 切换到设置 Tab
    fireEvent.click(screen.getByText("设置"));
    await waitFor(() => {
      expect(screen.getByText("AI 配置")).toBeInTheDocument();
    });

    // 切回校对 Tab，文件状态应保留
    fireEvent.click(screen.getByText("校对"));
    await waitFor(() => {
      expect(screen.getByText(/test.docx/)).toBeInTheDocument();
    });
  });

  // F1: .doc 文件选择 → 拦截提示（模拟选择 .doc 文件）
  it("shows warning dialog when selecting .doc file", async () => {
    mockOpen.mockResolvedValue("/tmp/test.doc");

    render(<App />);
    fireEvent.click(screen.getByText("校对"));

    const selectBtn = await screen.findByText("选择文件");
    fireEvent.click(selectBtn);

    await waitFor(() => {
      expect(screen.getByText(/格式提示/)).toBeInTheDocument();
      expect(screen.getByText(/.doc 格式较旧/)).toBeInTheDocument();
    });
  });

  // F2: .docx 文件选择 → 预览区渲染段落数（mock parse_document 返回 paragraphs）
  it("renders preview paragraphs after selecting .docx", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
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
      return Promise.resolve([]);
    });
    mockOpen.mockResolvedValue("/tmp/test.docx");

    render(<App />);
    fireEvent.click(screen.getByText("校对"));

    const selectBtn = await screen.findByText("选择文件");
    fireEvent.click(selectBtn);

    await waitFor(() => {
      // 预览区应渲染 3 个段落
      expect(screen.getByText("第 1 段")).toBeInTheDocument();
      expect(screen.getByText("第 2 段")).toBeInTheDocument();
      expect(screen.getByText("第 3 段")).toBeInTheDocument();
    });
  });
});
