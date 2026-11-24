export interface MenuItem {
  id: string;
  label: string;
  disabled?: boolean;
  disabledReason?: string;
  badge?: string;
  /** Present for toggle items: renders as a checkbox menu item with a check glyph when true. */
  checked?: boolean;
}
