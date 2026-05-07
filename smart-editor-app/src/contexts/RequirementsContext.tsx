import { createContext, useContext, useState, ReactNode } from "react";
import type { DeviationReport, FatalRisk, SelfReviewReport } from "../components/CheckResultsPanel";

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
  checkReport: DeviationReport | null;
  setCheckReport: (report: DeviationReport | null) => void;
  checkFatalRisks: FatalRisk[];
  setCheckFatalRisks: (risks: FatalRisk[]) => void;
  checkSelfReviewReport: SelfReviewReport | null;
  setCheckSelfReviewReport: (report: SelfReviewReport | null) => void;
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
  const [checkReport, setCheckReport] = useState<DeviationReport | null>(null);
  const [checkFatalRisks, setCheckFatalRisks] = useState<FatalRisk[]>([]);
  const [checkSelfReviewReport, setCheckSelfReviewReport] = useState<SelfReviewReport | null>(null);

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
