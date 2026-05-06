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

interface RequirementsContextType {
  requirements: RequirementItem[];
  setRequirements: (items: RequirementItem[]) => void;
  parsedText: string;
  setParsedText: (text: string) => void;
  fileName: string | null;
  setFileName: (name: string | null) => void;
}

const RequirementsContext = createContext<RequirementsContextType | undefined>(
  undefined
);

export function RequirementsProvider({ children }: { children: ReactNode }) {
  const [requirements, setRequirements] = useState<RequirementItem[]>([]);
  const [parsedText, setParsedText] = useState<string>("");
  const [fileName, setFileName] = useState<string | null>(null);

  return (
    <RequirementsContext.Provider
      value={{
        requirements,
        setRequirements,
        parsedText,
        setParsedText,
        fileName,
        setFileName,
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
