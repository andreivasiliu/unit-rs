# Vendored unit-dev 1.27.0 header files

This directory contains vendored header files copied from the `unit-dev` deb
package, from NGINX Unit's repositories found at their [installation guide].

[installation guide]: https://unit.nginx.org/installation/

They inherit the Apache-2.0 license of NGINX Unit.

These are only used for docs.rs builds in order to bypass the dependency
requirement. Note that these are not enough to actually build this crate, as
the `libunit.a` library is missing, and building it requires ~50 more header
files and some source files from NGINX Unit.
