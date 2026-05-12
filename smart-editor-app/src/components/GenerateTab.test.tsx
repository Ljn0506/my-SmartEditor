import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import GenerateTab from "./GenerateTab";
import { RequirementsProvider } from "../contexts/RequirementsContext";

const mockInvoke = vi.fn();
const mockAlert = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

function renderWithProvider(ui: React.ReactNode) {
  return render(<RequirementsProvider>{ui}</RequirementsProvider>);
}

describe("GenerateTab", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockAlert.mockReset();
    vi.spyOn(window, "alert").mockImplementation(mockAlert);
  });

  // 场景 1: 卡片渲染
  it("renders cards with title and status", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") {
        return Promise.resolve([
          {
            id: "c1",
            chapter: "1",
            title: "项目概述",
            content: "内容1",
            source_refs: ["file1.docx"],
            document_target: "technical",
            status: "draft",
            generated_by: "ai",
            related_cards: [],
          },
          {
            id: "c2",
            chapter: "2",
            title: "技术方案",
            content: "内容2",
            source_refs: ["file2.docx"],
            document_target: "technical",
            status: "confirmed",
            generated_by: "ai",
            related_cards: [],
          },
        ]);
      }
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    await waitFor(() => {
      expect(screen.getAllByText("项目概述").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("技术方案").length).toBeGreaterThanOrEqual(1);
    });
  });

  // 场景 2: 目标切换
  it("switches document target and loads corresponding cards", async () => {
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") {
        if (args?.documentTarget === "technical") {
          return Promise.resolve([
            {
              id: "t1",
              chapter: "1",
              title: "技术卡片",
              content: "",
              source_refs: [],
              document_target: "technical",
              status: "draft",
              generated_by: "ai",
              related_cards: [],
            },
          ]);
        }
        if (args?.documentTarget === "business") {
          return Promise.resolve([
            {
              id: "b1",
              chapter: "1",
              title: "商务卡片",
              content: "",
              source_refs: [],
              document_target: "business",
              status: "draft",
              generated_by: "ai",
              related_cards: [],
            },
          ]);
        }
      }
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    await waitFor(() => {
      expect(screen.getAllByText("技术卡片").length).toBeGreaterThanOrEqual(1);
    });

    fireEvent.click(screen.getByText("商务响应"));

    await waitFor(() => {
      expect(screen.getAllByText("商务卡片").length).toBeGreaterThanOrEqual(1);
    });

    expect(mockInvoke).toHaveBeenCalledWith("get_cards", {
      documentTarget: "business",
    });
  });

  // 场景 3: 商务审核拦截
  it("blocks one-click confirm for business cards", async () => {
    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") {
        if (args?.documentTarget === "technical") return Promise.resolve([]);
        return Promise.resolve([
          {
            id: "b1",
            chapter: "1",
            title: "商务条款",
            content: "内容",
            source_refs: [],
            document_target: "business",
            status: "draft",
            generated_by: "ai",
            related_cards: [],
          },
        ]);
      }
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    // 切换到商务
    fireEvent.click(screen.getByText("商务响应"));

    await waitFor(() => {
      expect(screen.getAllByText("商务条款").length).toBeGreaterThanOrEqual(1);
    });

    const confirmBtn = screen.getByText("一键确认所有草稿");
    fireEvent.click(confirmBtn);

    expect(mockAlert).toHaveBeenCalledWith(
      "商务卡片涉及敏感条款，必须逐张审核，不支持一键确认"
    );
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "confirm_all_cards",
      expect.anything()
    );
  });

  // 场景 4: 技术卡片批量确认
  it("confirms all technical draft cards with one click", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") {
        return Promise.resolve([
          {
            id: "t1",
            chapter: "1",
            title: "技术1",
            content: "内容1",
            source_refs: [],
            document_target: "technical",
            status: "draft",
            generated_by: "ai",
            related_cards: [],
          },
          {
            id: "t2",
            chapter: "2",
            title: "技术2",
            content: "内容2",
            source_refs: [],
            document_target: "technical",
            status: "draft",
            generated_by: "ai",
            related_cards: [],
          },
        ]);
      }
      if (cmd === "confirm_all_cards") return Promise.resolve(null);
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    await waitFor(() => {
      expect(screen.getAllByText("技术1").length).toBeGreaterThanOrEqual(1);
    });

    const confirmBtn = screen.getByText("一键确认所有草稿");
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("confirm_all_cards", {
        documentTarget: "technical",
      });
    });

    // 状态应变为已确认
    await waitFor(() => {
      expect(screen.getAllByText("已确认").length).toBe(2);
    });
  });

  // 场景 5: 全局参数弹窗
  it("opens global params modal and saves", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") return Promise.resolve([]);
      if (cmd === "save_global_params") return Promise.resolve(null);
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    await waitFor(() => {
      expect(screen.getByText("配置全局参数")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("配置全局参数"));

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "全局参数配置" })
      ).toBeInTheDocument();
    });

    // 填写项目名称和客户名称（必填项）
    const projectInput = screen.getByPlaceholderText("请输入项目名称");
    fireEvent.change(projectInput, { target: { value: "测试项目" } });
    const clientInput = screen.getByPlaceholderText("请输入客户名称");
    fireEvent.change(clientInput, { target: { value: "测试客户" } });

    // 保存
    fireEvent.click(screen.getByRole("button", { name: "保存" }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_global_params", {
        params: expect.objectContaining({ project_name: "测试项目" }),
      });
    });
  });

  // 场景 6: 风险标签渲染
  it("renders risk flag badges", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_global_params") return Promise.resolve(null);
      if (cmd === "get_cards") {
        return Promise.resolve([
          {
            id: "r1",
            chapter: "1",
            title: "报价章节",
            content: "内容",
            source_refs: ["file1.docx"],
            document_target: "technical",
            status: "draft",
            generated_by: "ai",
            related_cards: [],
            risk_flags: ["报价敏感"],
          },
        ]);
      }
      return Promise.resolve([]);
    });

    renderWithProvider(<GenerateTab />);

    await waitFor(() => {
      expect(screen.getByText("报价敏感")).toBeInTheDocument();
    });
  });
});
