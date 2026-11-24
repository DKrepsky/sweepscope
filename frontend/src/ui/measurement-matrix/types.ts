export type MeasurementCell = {
  state: 'valid' | 'unverified' | 'invalid' | 'stale';
  text: string;
  unit?: string;
  footnote?: number;
  tooltip?: string;
  warning?: boolean;
};

/** `color` is a design token name such as `color-ch1`. */
export interface MeasurementColumn {
  id: string;
  label: string;
  color: string;
}

export interface MeasurementRow {
  id: string;
  label: string;
  cells: Record<string, MeasurementCell>;
}

export interface MeasurementFootnote {
  key: number;
  text: string;
}
