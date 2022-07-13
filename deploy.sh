#!/bin/bash

set -e

socket="/var/run/control.unit.sock"
target=debug

if [ "$1" == "release" ]; then
    target=release
fi

curl -X PUT --data @- --unix-socket $socket http://localhost/config << EOF
{
  "listeners": {
      "*:8080": {
          "pass": "applications/rustapp"
      }
  },
  "applications": {
      "rustapp": {
          "type": "external",
          "working_directory": "$PWD",
          "executable": "$PWD/target/${target}/nginx_libunit_rust"
      }
  }
}
EOF

echo "Deployed $target target."
