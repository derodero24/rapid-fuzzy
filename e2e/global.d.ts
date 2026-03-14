interface Window {
  __ready?: boolean;
  __error?: string;
  __results: {
    levenshtein: number;
    levenshteinIdentical: number;
    normalizedLevenshtein: number;
    damerauLevenshtein: number;
    jaro: number;
    jaroWinkler: number;
    sorensenDice: number;
    levenshteinBatch: number[];
    jaroBatch: number[];
    levenshteinMany: number[];
    jaroMany: number[];
    tokenSortRatio: number;
    tokenSetRatio: number;
    partialRatio: number;
    weightedRatio: number;
    search: Array<{ item: string; score: number; index: number; positions: number[] }>;
    searchEmpty: Array<{ item: string; score: number; index: number; positions: number[] }>;
    closest: string | null;
    closestEmpty: string | null;
    indexSize: number;
    indexSearch: Array<{ item: string; score: number; index: number; positions: number[] }>;
    indexClosest: string | null;
    indexSizeAfterAdd: number;
    indexSizeAfterDestroy: number;
    missingExports: string[];
    allExportsPresent: boolean;
  };
}
