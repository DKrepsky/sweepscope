/** Attributes the element that opens a Menu must carry (WAI-ARIA menu button). */
export function menuTriggerAttributes(open: boolean) {
  return { 'aria-haspopup': 'menu', 'aria-expanded': open } as const;
}
