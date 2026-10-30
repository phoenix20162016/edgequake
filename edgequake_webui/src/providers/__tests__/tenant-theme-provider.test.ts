// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { TENANT_PRIMARY_STYLE_ID, applyTenantPrimaryStyle } from "../tenant-theme-provider";
import { selectTenantPrimaryColor, useTenantStore } from "@/stores/use-tenant-store";
import type { Tenant } from "@/types";

const styleNode = () => document.getElementById(TENANT_PRIMARY_STYLE_ID);

afterEach(() => {
  applyTenantPrimaryStyle("");
  useTenantStore.getState().reset();
});

describe("applyTenantPrimaryStyle", () => {
  it("writes one style element in head and updates it in place", () => {
    applyTenantPrimaryStyle(":root { --primary: #0f766e; }");
    applyTenantPrimaryStyle(":root { --primary: #15803d; }");
    const nodes = document.querySelectorAll(`#${TENANT_PRIMARY_STYLE_ID}`);
    expect(nodes).toHaveLength(1);
    expect(nodes[0].parentElement).toBe(document.head);
    expect(styleNode()?.textContent).toContain("#15803d");
  });

  it("removes the element when there is no tenant color", () => {
    applyTenantPrimaryStyle(":root { --primary: #0f766e; }");
    applyTenantPrimaryStyle("");
    expect(styleNode()).toBeNull();
  });
});

function tenant(id: string, primary_color: string | null): Tenant {
  return { id, name: id, primary_color } as Tenant;
}

describe("selectTenantPrimaryColor", () => {
  it("returns null when no tenant is selected", () => {
    useTenantStore.setState({ selectedTenantId: null, tenants: [] });
    expect(selectTenantPrimaryColor(useTenantStore.getState())).toBeNull();
  });

  it("uses the live tenant record when the list is loaded", () => {
    useTenantStore.getState().setTenants([tenant("a", "#0f766e"), tenant("b", null)]);
    useTenantStore.getState().selectTenant("a");
    expect(selectTenantPrimaryColor(useTenantStore.getState())).toBe("#0f766e");
    useTenantStore.getState().selectTenant("b");
    expect(selectTenantPrimaryColor(useTenantStore.getState())).toBeNull();
  });

  it("falls back to the persisted last-known color before the list loads", () => {
    useTenantStore.getState().setTenants([tenant("a", "#be123c")]);
    useTenantStore.setState({ tenants: [] });
    useTenantStore.getState().selectTenant("a");
    expect(selectTenantPrimaryColor(useTenantStore.getState())).toBe("#be123c");
  });

  it("keeps the cache per tenant when switching between tenants", () => {
    useTenantStore.getState().setTenants([tenant("a", "#6d28d9"), tenant("b", "#475569")]);
    expect(useTenantStore.getState().primaryColorByTenant).toEqual({ a: "#6d28d9", b: "#475569" });
  });
});
