import { createContext, useContext, useState, ReactNode } from "react";

export interface RequirementItem {
  id: number;
  section: string;
  text: string;
  category: string;
  mandatory: boolean;
  keywords: string[];
  checked: boolean;
}

export interface ParagraphInfo {
  index: number;
  text: string;
  char_offset: number;
}

interface RequirementsContextType {
  // 招标需求（UploadTab 解析）
  requirements: RequirementItem[];
  setRequirements: (items: RequirementItem[]) => void;
  parsedText: string;
  setParsedText: (text: string) => void;
  fileName: string | null;
  setFileName: (name: string | null) => void;
  // 投标文档（CheckTab 选择）
  bidFilePath: string | null;
  setBidFilePath: (path: string | null) => void;
  bidFileName: string | null;
  setBidFileName: (name: string | null) => void;
  bidParagraphs: ParagraphInfo[];
  setBidParagraphs: (paragraphs: ParagraphInfo[]) => void;
  // 检查结果（CheckTab 运行）
  checkReport: any | null;
  setCheckReport: (report: any | null) => void;
  checkFatalRisks: any[];
  setCheckFatalRisks: (risks: any[]) => void;
  checkSelfReviewReport: any | null;
  setCheckSelfReviewReport: (report: any | null) => void;
}

const RequirementsContext = createContext<RequirementsContextType | undefined>(
  undefined
);

export function RequirementsProvider({ children }: { children: ReactNode }) {
  const [requirements, setRequirements] = useState<RequirementItem[]>([]);
  const [parsedText, setParsedText] = useState<string>("");
  const [fileName, setFileName] = useState<string | null>(null);

  // CheckTab 持久化状态
  const [bidFilePath, setBidFilePath] = useState<string | null>(null);
  const [bidFileName, setBidFileName] = useState<string | null>(null);
  const [bidParagraphs, setBidParagraphs] = useState<ParagraphInfo[]>([]);
  const [checkReport, setCheckReport] = useState<any | null>(null);
  const [checkFatalRisks, setCheckFatalRisks] = useState<any[]>([]);
  const [checkSelfReviewReport, setCheckSelfReviewReport] = useState<any | null>(null);

  return (
    <RequirementsContext.Provider
      value={{
        requirements,
        setRequirements,
        parsedText,
        setParsedText,
        fileName,
        setFileName,
        bidFilePath,
        setBidFilePath,
        bidFileName,
        setBidFileName,
        bidParagraphs,
        setBidParagraphs,
        checkReport,
        setCheckReport,
        checkFatalRisks,
        setCheckFatalRisks,
        checkSelfReviewReport,
        setCheckSelfReviewReport,
      }}
    >
      {children}
    </RequirementsContext.Provider>
  );
}

export function useRequirements() {
  const ctx = useContext(RequirementsContext);
  if (!ctx) {
    throw new Error("useRequirements must be used within RequirementsProvider");
  }
  return ctx;
}
