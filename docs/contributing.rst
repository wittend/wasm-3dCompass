Contributing
============

Issues and pull requests are welcome on the GitHub repository.

Before submitting a change:

#. Build the WebAssembly module and copy it to ``web/compass.wasm`` (see :doc:`building`).
#. Run the harness locally and check the affected controls by hand.
#. Build the documentation with ``sphinx-build -W``. Update the pages in ``docs/``
   if you change behaviour, the exported API, or the layer flags.

Licence
-------

The project is licensed under the Apache License, Version 2.0. By contributing
you agree that your contribution is licensed on the same terms.
