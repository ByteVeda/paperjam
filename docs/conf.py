import os
import sys

sys.path.insert(0, os.path.abspath("../py_src"))

project = "paperjam"
copyright = "2024, paperjam contributors"
author = "paperjam contributors"
release = "0.1.0"

extensions = [
    "myst_parser",
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "sphinx.ext.intersphinx",
    "sphinx_copybutton",
    "sphinxcontrib.mermaid",
]

myst_enable_extensions = ["colon_fence", "deflist"]

html_theme = "furo"
html_title = "paperjam"
html_theme_options = {
    "sidebar_hide_name": False,
    "navigation_with_keys": True,
}

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "pandas": ("https://pandas.pydata.org/docs", None),
}

autodoc_member_order = "bysource"
autodoc_typehints = "description"
napoleon_google_docstring = True
