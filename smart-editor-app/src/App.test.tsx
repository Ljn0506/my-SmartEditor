import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve([])),
}));

describe("App", () => {
  it("renders the app title", () => {
    render(<App />);
    expect(screen.getByText("智能文档助手")).toBeInTheDocument();
  });

  it("renders all tabs", () => {
    render(<App />);
    expect(screen.getByText("需求上传")).toBeInTheDocument();
    expect(screen.getByText("智能生成")).toBeInTheDocument();
    expect(screen.getByText("校对")).toBeInTheDocument();
    expect(screen.getByText("设置")).toBeInTheDocument();
  });

  it("switches to generate tab when clicked", async () => {
    render(<App />);
    const generateTab = screen.getByText("智能生成");
    fireEvent.click(generateTab);
    expect(screen.getByRole("button", { name: /生成章节大纲/ })).toBeInTheDocument();
  });

  it("switches to settings tab when clicked", async () => {
    render(<App />);
    const settingsTab = screen.getByText("设置");
    fireEvent.click(settingsTab);
    await waitFor(() => {
      expect(screen.getByText("AI 配置")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("http://localhost:11434")).toBeInTheDocument();
    });
  });

  it("switches to check tab and shows deviation panel", () => {
    render(<App />);
    const checkTab = screen.getByText("校对");
    fireEvent.click(checkTab);
    expect(screen.getByText("校对检查")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /检查投标文件/ })).toBeInTheDocument();
  });

  it("shows upload drop zone by default", () => {
    render(<App />);
    expect(screen.getByText("需求文件上传")).toBeInTheDocument();
    expect(screen.getByText(/拖入文件到这里/)).toBeInTheDocument();
  });

  it("shows copy button in editor area", () => {
    render(<App />);
    expect(screen.getByText("复制到剪贴板")).toBeInTheDocument();
  });
});
