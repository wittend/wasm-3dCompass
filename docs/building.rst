Building
========

The project has two build products:

* ``web/compass.wasm``, the WebAssembly geometry module, built from ``src/lib.rs``.
* The documentation in ``docs/``, built with Sphinx and the Furo theme.

WebAssembly module
------------------

Requirements: a Rust toolchain (``rustup``) with the ``wasm32-unknown-unknown``
target.

.. code-block:: sh

   rustup target add wasm32-unknown-unknown
   cargo build --release --target wasm32-unknown-unknown
   cp target/wasm32-unknown-unknown/release/wasm_3dcompass.wasm web/compass.wasm

The release profile optimises for size (``opt-level = "s"``, LTO, and
``panic = "abort"``). The module has no dependencies and no imports.

Documentation
-------------

.. code-block:: sh

   python3 -m venv .venv
   . .venv/bin/activate
   pip install -r docs/requirements.txt
   sphinx-build -W -b html docs docs/_build/html

Read the Docs builds the same site from ``.readthedocs.yaml``. The build is set
to fail on warnings, so keep ``sphinx-build -W`` clean before pushing.

Continuous integration
----------------------

``.github/workflows/ci.yml`` runs on push and pull request:

1. Builds the WebAssembly module and checks that it exports the expected functions.
2. Builds the Sphinx documentation with warnings treated as errors.

The same workflow file is read by Gitea Actions when Actions is enabled on the
Gitea instance, so one file serves both hosts.
