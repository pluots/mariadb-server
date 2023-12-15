#!/usr/bin/env python3
"""`cargo metadata` provides us with a JSON-formatted version of all workspace member
manifests. This tool parses

See more at <https://doc.rust-lang.org/cargo/commands/cargo-metadata.html>.

We return a key-value list separated by `;`. We use `|` for a list separator within
values. All keys start with `from_helper` so they can be set directly in CMake.
"""

import json
import subprocess as sp
import sys
from dataclasses import dataclass
from pathlib import Path

SOURCE_DIR = Path(__file__).parent.parent
PLUGIN_PATH = SOURCE_DIR.joinpath("rust").joinpath("plugins")
EXAMPLE_PLUGIN_PATH = SOURCE_DIR.joinpath("rust").joinpath("examples")
HELPER_PFX = "from_helper_"


@dataclass
class Plugin:
    # Name of the package to cargo
    cargo_name: str
    is_example: bool
    manifest_path: Path
    meta: dict

    def name(self) -> str:
        """Cargo name in snake case"""
        return self.cargo_name.lower().replace("-", "_").strip()

    def cmake_var_prefix(self) -> str:
        """The prefix we use for all CMake variables"""
        return f"{HELPER_PFX}plugin_{self.name()}"

    def mdb_features(self) -> list[str]:
        """List any specific features from the `mariadb` crate that affect linkage"""
        mdb_dep = next(
            (dep for dep in self.meta["dependencies"] if dep["name"] == "mariadb"), None
        )
        if mdb_dep is None:
            return []
        return mdb_dep["features"]

    def create_env(self) -> dict:
        """Create the variables we want to have in cmake"""
        ex_pfx_upper = "EXAMPLE_" if self.is_example else ""

        features = self.mdb_features()
        needs_storage = "storage" in features
        needs_service_sql = "service-sql" in features
        needs_any_services = any(f.startswith("service-") for f in features)

        return {
            "cargo_name": self.cargo_name,
            "cache_name": f"PLUGIN_{ex_pfx_upper}{self.name().upper()}",
            "cmake_target_name": f"plugin_{self.name()}",
            "mariadb_features": features,
            "is_example": "TRUE" if self.is_example else "FALSE",
            "needs_storage": needs_storage,
            "needs_service_sql": needs_service_sql,
            "needs_any_services": needs_any_services,
            # "target_basename": self.meta
            # "staticlib_fname":
        }


def main():
    ws_manifest = SOURCE_DIR.joinpath("Cargo.toml")
    print(f"reading workspace manifest from {ws_manifest}", file=sys.stderr)

    # Use `cargo metadata` to load workspace members
    ws_meta = json.loads(
        sp.check_output(
            [
                "cargo",
                "metadata",
                "--manifest-path",
                ws_manifest,
                "--format-version=1",
                "--no-deps",
            ]
        )
    )

    plugins: list[Plugin] = []

    for pkg_meta in ws_meta["packages"]:
        manifest_path = Path(pkg_meta["manifest_path"])

        # Keep normal plugins and examples, ignore any other crates
        if PLUGIN_PATH in manifest_path.parents:
            plugins.append(Plugin(pkg_meta["name"], False, manifest_path, pkg_meta))
        elif EXAMPLE_PLUGIN_PATH in manifest_path.parents:
            plugins.append(Plugin(pkg_meta["name"], True, manifest_path, pkg_meta))

    ret = f"{HELPER_PFX}all_plugins="
    ret += "|".join(p.name() for p in plugins)
    ret += ";"

    for plugin in plugins:
        pfx = plugin.cmake_var_prefix()
        ret += ";".join(f"{pfx}_{k}={v}" for (k, v) in plugin.create_env().items())
        ret += ";"

    ret = ret.rstrip(";")

    print(ret)


if __name__ == "__main__":
    main()
