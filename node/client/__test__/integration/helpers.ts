import * as fs from "node:fs";
import * as path from "node:path";
import { UnityCatalogClient } from "../../dist/index";

const STATE_FILE = path.join(__dirname, ".server-state.json");

let cachedBaseUrl: string | null = null;

export function getBaseUrl(): string {
  if (cachedBaseUrl) return cachedBaseUrl;
  const state = JSON.parse(fs.readFileSync(STATE_FILE, "utf-8"));
  cachedBaseUrl = state.baseUrl as string;
  return cachedBaseUrl;
}

export function createClient(): UnityCatalogClient {
  return new UnityCatalogClient(getBaseUrl());
}

let counter = 0;

export function uniqueName(prefix: string): string {
  counter++;
  return `${prefix}_${Date.now()}_${counter}`;
}
