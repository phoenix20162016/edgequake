"use client";

/**
 * SPEC-175 — below `md` the nav becomes two selects: scope, then section. Same registry,
 * same gates as the desktop nav (DRY). Changing either select navigates.
 */

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { SettingsAccess } from "@/lib/settings/access";
import {
  getSection,
  scopeHref,
  sectionHref,
  visibleScopes,
  visibleSections,
  type SettingsScope,
  type SettingsSectionId,
} from "@/lib/settings/registry";
import { useRouter } from "next/navigation";

/**
 * Same footprint as the two selects, shown while access is still resolving.
 * Without it the header above the section grew ~50px when the nav mounted (CLS).
 * `h-9` (not `h-8`): the SelectTrigger's `data-[size=default]:h-9` wins over `h-8`,
 * so the rendered selects are 36px tall.
 */
export function MobileSettingsNavSkeleton() {
  return (
    <div className="grid grid-cols-2 gap-2" aria-hidden="true" data-testid="settings-mobile-nav-skeleton">
      <div className="h-9" />
      <div className="h-9" />
    </div>
  );
}

export function MobileSettingsNav({
  access,
  activeSection,
  workspaceSlug,
}: {
  access: SettingsAccess;
  activeSection: SettingsSectionId;
  workspaceSlug: string | null;
}) {
  const router = useRouter();
  const active = getSection(activeSection);
  const scopes = visibleScopes(access);
  const sections = visibleSections(access, active.scope);

  return (
    <div className="grid grid-cols-2 gap-2" data-testid="settings-mobile-nav">
      <Select
        value={active.scope}
        onValueChange={(value) => router.push(scopeHref(value as SettingsScope, workspaceSlug))}
      >
        <SelectTrigger aria-label="Settings scope" className="h-8 text-sm" data-testid="settings-mobile-scope">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {scopes.map((scope) => (
            <SelectItem key={scope.id} value={scope.id}>
              {scope.fallback}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={activeSection}
        onValueChange={(value) => router.push(sectionHref(value as SettingsSectionId, workspaceSlug))}
      >
        <SelectTrigger aria-label="Settings section" className="h-8 text-sm" data-testid="settings-mobile-section">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {sections.map((section) => (
            <SelectItem key={section.id} value={section.id}>
              {section.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
