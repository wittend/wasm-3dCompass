# Sphinx configuration for the 3-Axis Field Compass documentation.
# Built by Read the Docs (see .readthedocs.yaml) and by CI.

project = "3-Axis Field Compass"
author = "Dave Witten"
copyright = "2026, Dave Witten"
release = "0.1.0"
version = "0.1"

extensions = [
    "sphinx.ext.autosectionlabel",
]

# Make section titles usable as :ref: targets across pages.
autosectionlabel_prefix_document = True

exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

language = "en"

html_theme = "furo"
html_title = "3-Axis Field Compass"
html_theme_options = {
    "source_repository": "https://github.com/wittend/wasm-3dCompass",
    "source_branch": "main",
    "source_directory": "docs/",
    "navigation_with_keys": True,
}
