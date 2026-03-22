/**
 * Remark plugin that expands <CodeExample snippet="..." /> directives in MDX
 * into tabbed code blocks (Rust / Python / TypeScript) by reading labeled
 * regions from source files in examples/{rust,python,typescript}/.
 *
 * Snippet tagging convention (tag lines are stripped from rendered output):
 *   Rust/TypeScript:  // [snippet:name]  ...  // [/snippet:name]
 *   Python:           # [snippet:name]   ...  # [/snippet:name]
 */

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { visit } from "unist-util-visit";

// Regex to match snippet sentinels in any language
const SNIPPET_START = /^\s*(?:\/\/|#)\s*\[snippet:([a-z_]+)\]\s*$/;
const SNIPPET_END = /^\s*(?:\/\/|#)\s*\[\/snippet:([a-z_]+)\]\s*$/;

/**
 * Parse all snippet regions from a file's content.
 * Returns a Map<snippetName, extractedCode>.
 */
function parseSnippets(content) {
  const lines = content.split("\n");
  const snippets = new Map();
  let currentName = null;
  let currentLines = [];

  for (const line of lines) {
    const startMatch = line.match(SNIPPET_START);
    const endMatch = line.match(SNIPPET_END);

    if (startMatch) {
      currentName = startMatch[1];
      currentLines = [];
    } else if (endMatch && currentName === endMatch[1]) {
      // Dedent: find minimum indentation of non-empty lines
      const nonEmptyLines = currentLines.filter((l) => l.trim().length > 0);
      const minIndent = nonEmptyLines.reduce((min, l) => {
        const indent = l.match(/^(\s*)/)[1].length;
        return Math.min(min, indent);
      }, Infinity);
      const dedentedLines =
        nonEmptyLines.length === 0
          ? currentLines
          : currentLines.map((l) => l.slice(minIndent));
      snippets.set(currentName, dedentedLines.join("\n").trimEnd());
      currentName = null;
      currentLines = [];
    } else if (currentName !== null) {
      currentLines.push(line);
    }
  }

  return snippets;
}

/**
 * Recursively walk a directory and collect all files with the given extensions.
 */
function collectFiles(dir, extensions) {
  const results = [];
  try {
    for (const entry of readdirSync(dir)) {
      const fullPath = join(dir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        results.push(...collectFiles(fullPath, extensions));
      } else if (extensions.some((ext) => entry.endsWith(ext))) {
        results.push(fullPath);
      }
    }
  } catch {
    // Directory doesn't exist yet — return empty
  }
  return results;
}

/**
 * Build the snippet registry by scanning all example source files.
 * Returns a Map<snippetName, { rust?: string, python?: string, typescript?: string }>
 */
function buildRegistry(repoRoot) {
  const registry = new Map();

  const addSnippets = (files, lang) => {
    for (const file of files) {
      const content = readFileSync(file, "utf-8");
      const snippets = parseSnippets(content);
      for (const [name, code] of snippets) {
        if (!registry.has(name)) registry.set(name, {});
        registry.get(name)[lang] = code;
      }
    }
  };

  addSnippets(
    collectFiles(join(repoRoot, "examples/rust/src"), [".rs"]),
    "rust",
  );
  addSnippets(
    collectFiles(join(repoRoot, "examples/python"), [".py"]),
    "python",
  );
  addSnippets(
    collectFiles(join(repoRoot, "examples/typescript/examples"), [".ts"]),
    "typescript",
  );

  return registry;
}

/**
 * Create an MDX JSX element node for a code block wrapped in a TabItem.
 */
function makeTabItem(label, lang, code) {
  return {
    type: "mdxJsxFlowElement",
    name: "TabItem",
    attributes: [{ type: "mdxJsxAttribute", name: "label", value: label }],
    children: [
      {
        type: "code",
        lang,
        value: code,
      },
    ],
  };
}

/**
 * Create the Tabs wrapper containing all language TabItems.
 */
function makeTabsNode(snippetName, snippet) {
  const children = [];

  if (snippet.rust !== undefined) {
    children.push(makeTabItem("Rust", "rust", snippet.rust));
  }
  if (snippet.python !== undefined) {
    children.push(makeTabItem("Python", "python", snippet.python));
  }
  if (snippet.typescript !== undefined) {
    children.push(makeTabItem("TypeScript", "typescript", snippet.typescript));
  }

  if (children.length === 0) {
    throw new Error(
      `Snippet "${snippetName}" was found in the registry but has no language implementations.`,
    );
  }

  return {
    type: "mdxJsxFlowElement",
    name: "Tabs",
    attributes: [],
    children,
  };
}

/**
 * Ensure the required Starlight Tabs/TabItem import exists in the MDX file's
 * AST. Injects it if missing.
 */
function ensureTabsImport(tree) {
  const IMPORT_SOURCE = "@astrojs/starlight/components";
  const IMPORT_STMT = `import { Tabs, TabItem } from "${IMPORT_SOURCE}";`;

  // Check if import already exists
  let found = false;
  visit(tree, "mdxjsEsm", (node) => {
    if (node.value && node.value.includes(IMPORT_SOURCE)) {
      found = true;
    }
  });

  if (!found) {
    tree.children.unshift({
      type: "mdxjsEsm",
      value: IMPORT_STMT,
      data: {
        estree: {
          type: "Program",
          body: [
            {
              type: "ImportDeclaration",
              specifiers: [
                {
                  type: "ImportSpecifier",
                  imported: { type: "Identifier", name: "Tabs" },
                  local: { type: "Identifier", name: "Tabs" },
                },
                {
                  type: "ImportSpecifier",
                  imported: { type: "Identifier", name: "TabItem" },
                  local: { type: "Identifier", name: "TabItem" },
                },
              ],
              source: { type: "Literal", value: IMPORT_SOURCE },
            },
          ],
          sourceType: "module",
        },
      },
    });
  }
}

// Lazily built registry — shared across all files in a single build
let _registry = null;
let _repoRoot = null;

export function remarkCodeSnippets(options = {}) {
  return (tree, file) => {
    // Resolve the repo root: walk up from the MDX file's directory until we
    // find a directory that contains examples/rust/
    if (_registry === null) {
      // Default: use the current working directory (repo root when running
      // `astro build` or `astro dev` from the docs/ directory via npm workspace)
      _repoRoot = options.repoRoot ?? resolve(process.cwd(), "..");
      _registry = buildRegistry(_repoRoot);
    }

    let hasCodeExample = false;

    visit(tree, "mdxJsxFlowElement", (node, index, parent) => {
      if (node.name !== "CodeExample") return;

      // Extract the snippet attribute value
      const snippetAttr = (node.attributes || []).find(
        (a) => a.type === "mdxJsxAttribute" && a.name === "snippet",
      );
      if (!snippetAttr) {
        throw new Error(
          `<CodeExample /> used without a "snippet" attribute in ${file.path ?? "unknown file"}`,
        );
      }

      const snippetName =
        typeof snippetAttr.value === "string"
          ? snippetAttr.value
          : snippetAttr.value?.value;

      if (!_registry.has(snippetName)) {
        throw new Error(
          `Unknown snippet "${snippetName}" referenced in ${file.path ?? "unknown file"}. ` +
            `Available snippets: ${[..._registry.keys()].sort().join(", ")}`,
        );
      }

      const snippet = _registry.get(snippetName);
      const tabsNode = makeTabsNode(snippetName, snippet);

      // Replace the <CodeExample /> node with the <Tabs> node in-place
      parent.children.splice(index, 1, tabsNode);
      hasCodeExample = true;
    });

    if (hasCodeExample) {
      ensureTabsImport(tree);
    }
  };
}
