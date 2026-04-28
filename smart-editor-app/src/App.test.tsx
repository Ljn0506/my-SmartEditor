import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the app title", () => {
    render(<App />);
    expect(screen.getByText("智能文档助手")).toBeInTheDocument();
  });

  it("renders all tabs", () => {
    render(<App />);
    expect(screen.getByText("需求上传")).toBeInTheDocument();
    expect(screen.getByText("模板库")).toBeInTheDocument();
    expect(screen.getByText("智能推送")).toBeInTheDocument();
    expect(screen.getByText("校对")).toBeInTheDocument();
    expect(screen.getByText("设置")).toBeInTheDocument();
  });

  it("switches to library tab when clicked", () => {
    render(<App />);
    const libraryTab = screen.getByText("模板库");
    fireEvent.click(libraryTab);
    expect(screen.getByPlaceholderText("搜索标题、内容、标签...")).toBeInTheDocument();
  });

  it("switches to settings tab when clicked", () => {
    render(<App />);
    const settingsTab = screen.getByText("设置");
    fireEvent.click(settingsTab);
    expect(screen.getByText("AI 配置")).toBeInTheDocument();
    expect(screen.getByDisplayValue("http://localhost:11434")).toBeInTheDocument();
  });

  it("switches to check tab and shows deviation panel", () => {
    render(<App />);
    const checkTab = screen.getByText("校对");
    fireEvent.click(checkTab);
    expect(screen.getByText("偏离检查")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /开始检查/ })).toBeInTheDocument();
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
