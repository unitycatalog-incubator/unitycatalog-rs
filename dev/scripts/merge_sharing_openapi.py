#!/usr/bin/env python3
"""Merge the hand-maintained NDJSON query fragment into the generated sharing spec.

The Open Sharing OpenAPI document is generated from `proto/sharing` by the
`google-gnostic-openapi` buf plugin (it covers the discovery + volume + skill
surfaces). The Delta Sharing **query** path (table version / metadata / query /
change-data-feed) is not part of the proto service — those endpoints return
`application/x-ndjson` with custom headers, a contract the generator does not
model — so they live in a hand-maintained fragment and are merged in here.

Usage:
    merge_sharing_openapi.py <generated.yaml> <fragment.yaml> <output.yaml>

The script deep-merges the fragment's `paths` and `components.{schemas,responses}`
into the generated base and writes the combined document to <output.yaml>.
"""

import sys
from pathlib import Path

import yaml


def merge(base: dict, fragment: dict) -> dict:
    """Merge `paths` and `components.{schemas,responses}` from fragment into base."""
    base.setdefault("paths", {})
    for path, item in fragment.get("paths", {}).items():
        if path in base["paths"]:
            raise SystemExit(f"path collision between generated spec and fragment: {path}")
        base["paths"][path] = item

    base_components = base.setdefault("components", {})
    frag_components = fragment.get("components", {})
    for section in ("schemas", "responses"):
        target = base_components.setdefault(section, {})
        for name, value in frag_components.get(section, {}).items():
            # Fragment wins for shared definitions (e.g. error schemas), so the
            # hand-maintained shapes referenced by the query paths are present.
            target[name] = value
    return base


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    generated_path, fragment_path, output_path = (Path(a) for a in sys.argv[1:])

    base = yaml.safe_load(generated_path.read_text())
    fragment = yaml.safe_load(fragment_path.read_text())
    merged = merge(base, fragment)

    header = (
        "# Generated from proto/sharing (google-gnostic-openapi) and merged with\n"
        "# openapi/sharing-query-paths.yaml via dev/scripts/merge_sharing_openapi.py.\n"
        "# Do not edit by hand — see `just generate-openapi-sharing`.\n"
    )
    output_path.write_text(header + yaml.safe_dump(merged, sort_keys=False, width=1000))
    print(f"wrote {output_path} ({len(merged['paths'])} paths)")


if __name__ == "__main__":
    main()
