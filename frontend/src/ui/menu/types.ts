export interface MenuItem {
  id: string;
  label: string;
  disabled?: boolean;
  disabledReason?: string;
  badge?: string;
  /** Present for toggle items: renders as a checkbox menu item with a check glyph when true. */
  checked?: boolean;
  /** With `checked`: exactly one item of a group is chosen, so it renders as a radio item. */
  radio?: boolean;
  /** A rule above this item, to set it apart from the group above. */
  separatorBefore?: boolean;
}
