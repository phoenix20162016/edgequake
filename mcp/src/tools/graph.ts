/**
 * Knowledge graph exploration tools.
 */
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { getClient } from "../client.js";
import { formatError } from "../errors.js";

/** SPEC-162 R15: prefer entity_id; fall back to entity_name → bare slug. */
export function resolveEntityRef(
  entityId: string | undefined,
  entityName: string | undefined,
): string {
  if (entityId && entityId.trim()) {
    const id = entityId.trim();
    if (id.startsWith("ent:")) {
      const rest = id.slice(4);
      const colon = rest.indexOf(":");
      return colon >= 0 ? rest.slice(colon + 1) : rest;
    }
    return id;
  }
  if (entityName && entityName.trim()) {
    return entityName.trim();
  }
  throw new Error("entity_id or entity_name is required");
}

export function registerGraphTools(server: McpServer): void {
  // graph_search_entities
  server.tool(
    "graph_search_entities",
    "Search for entities in the knowledge graph. Entities are people, organizations, technologies, concepts, etc. extracted from documents.",
    {
      search: z
        .string()
        .optional()
        .describe("Search term to filter entities by name"),
      label: z
        .string()
        .optional()
        .describe(
          "Filter by entity type: PERSON, ORGANIZATION, TECHNOLOGY, CONCEPT, EVENT, LOCATION, PRODUCT",
        ),
      limit: z.number().optional().describe("Max results (default: 20)"),
    },
    async (params) => {
      try {
        const client = await getClient();
        const entities = await client.graph.entities.list({
          search: params.search,
          label: params.label,
          per_page: params.limit ?? 20,
        });

        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(
                entities.map((e) => ({
                  name: e.name,
                  label: e.label,
                  description: e.description,
                })),
                null,
                2,
              ),
            },
          ],
        };
      } catch (error) {
        return formatError(error);
      }
    },
  );

  // graph_get_entity
  server.tool(
    "graph_get_entity",
    "Get detailed information about a specific entity including its properties and source documents. Prefer entity_id (ent:workspace:slug).",
    {
      entity_id: z
        .string()
        .optional()
        .describe("Agent entity id (ent:workspace:slug)"),
      entity_name: z
        .string()
        .optional()
        .describe(
          "Legacy entity name (e.g. RUST, OPENAI, MACHINE_LEARNING). Resolved when entity_id is absent.",
        ),
    },
    async (params) => {
      try {
        const client = await getClient();
        const id = resolveEntityRef(params.entity_id, params.entity_name);
        const entity = await client.graph.entities.get(id);

        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(entity, null, 2),
            },
          ],
        };
      } catch (error) {
        return formatError(error);
      }
    },
  );

  // graph_entity_neighborhood
  server.tool(
    "graph_entity_neighborhood",
    "Get an entity's neighborhood — all directly connected entities and their relationships. Prefer entity_id (ent:workspace:slug).",
    {
      entity_id: z
        .string()
        .optional()
        .describe("Agent entity id (ent:workspace:slug)"),
      entity_name: z
        .string()
        .optional()
        .describe("Legacy entity name. Resolved when entity_id is absent."),
    },
    async (params) => {
      try {
        const client = await getClient();
        const id = resolveEntityRef(params.entity_id, params.entity_name);
        const neighborhood = await client.graph.entities.neighborhood(id);

        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(
                {
                  center: {
                    name: neighborhood.center.name,
                    label: neighborhood.center.label,
                    description: neighborhood.center.description,
                  },
                  neighbors: neighborhood.neighbors.map((n) => ({
                    entity: {
                      name: n.entity.name,
                      label: n.entity.label,
                      description: n.entity.description,
                    },
                    relationship: n.relationship,
                    direction: n.direction,
                  })),
                },
                null,
                2,
              ),
            },
          ],
        };
      } catch (error) {
        return formatError(error);
      }
    },
  );

  // graph_search_relationships
  server.tool(
    "graph_search_relationships",
    "Search relationships between entities in the knowledge graph",
    {
      source: z.string().optional().describe("Source entity name"),
      target: z.string().optional().describe("Target entity name"),
      label: z.string().optional().describe("Relationship type/label"),
      limit: z.number().optional().describe("Max results (default: 20)"),
    },
    async (params) => {
      try {
        const client = await getClient();
        const relationships = await client.graph.relationships.list({
          source: params.source,
          target: params.target,
          label: params.label,
          per_page: params.limit ?? 20,
        });

        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(
                relationships.map((r) => ({
                  source: r.source,
                  target: r.target,
                  label: r.label,
                  description: r.description,
                  weight: r.weight,
                })),
                null,
                2,
              ),
            },
          ],
        };
      } catch (error) {
        return formatError(error);
      }
    },
  );
}
