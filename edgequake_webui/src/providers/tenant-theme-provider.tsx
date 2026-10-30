'use client';

/**
 * @module TenantThemeProvider
 * @description Applies the selected tenant's brand color to the whole app.
 *
 * Renders nothing. It owns a single `<style id="eq-tenant-primary">` in `<head>`
 * whose text is produced by `buildPrimaryThemeCss`. With no tenant color the style
 * is removed, so the stylesheet defaults apply unchanged.
 *
 * WHY useLayoutEffect: the accent is written before the browser paints, so a tenant
 * color never flashes the default indigo on first render.
 */

import { buildPrimaryThemeCss } from '@/lib/theme/tenant-primary';
import { selectTenantPrimaryColor, useTenantStore } from '@/stores/use-tenant-store';
import { useLayoutEffect } from 'react';

export const TENANT_PRIMARY_STYLE_ID = 'eq-tenant-primary';

/** Create, update or remove the tenant accent `<style>` element. */
export function applyTenantPrimaryStyle(css: string, doc: Document = document): void {
  const existing = doc.getElementById(TENANT_PRIMARY_STYLE_ID);
  if (!css) {
    existing?.remove();
    return;
  }
  const node = existing ?? doc.createElement('style');
  node.id = TENANT_PRIMARY_STYLE_ID;
  if (node.textContent !== css) node.textContent = css;
  if (!node.isConnected) doc.head.appendChild(node);
}

export function TenantThemeProvider(): null {
  const primaryColor = useTenantStore(selectTenantPrimaryColor);

  useLayoutEffect(() => {
    applyTenantPrimaryStyle(buildPrimaryThemeCss(primaryColor));
  }, [primaryColor]);

  useLayoutEffect(() => () => applyTenantPrimaryStyle(''), []);

  return null;
}

export default TenantThemeProvider;
