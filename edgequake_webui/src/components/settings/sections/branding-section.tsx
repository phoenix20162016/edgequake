"use client";

/** SPEC-175 Organization / Branding. Tenant managers only. */

import { TenantBrandingCard } from "@/components/settings/tenant-branding-card";
import { SettingsFrame } from "@/components/settings/shell/settings-frame";
import { useSettingsContext } from "@/hooks/use-settings-context";
import { useTranslation } from "react-i18next";

export function BrandingSection() {
  const { t } = useTranslation();
  const { tenantId } = useSettingsContext(null);
  return (
    <SettingsFrame
      title={t("settings.shell.sections.branding", "Branding")}
      lead={t(
        "settings.shell.branding.lead",
        "Set the color your organization sees across EdgeQuake. It applies to every member of this organization.",
      )}
    >
      <TenantBrandingCard tenantId={tenantId} />
    </SettingsFrame>
  );
}
