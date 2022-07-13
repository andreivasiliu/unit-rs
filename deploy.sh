#!/bin/bash

set -e

app_name=rustapp
socket="/var/run/control.unit.sock"
target=debug
cwd=$(pwd)

if [ "$1" == "release" ]; then
    target=release
fi

echo -n "Configuring: "
curl -X PUT --data @- --unix-socket $socket http://localhost/config << EOF
{
  "listeners": {
      "*:8080": {
          "pass": "applications/$app_name"
      }
  },
  "applications": {
      "$app_name": {
          "type": "external",
          "working_directory": "$cwd",
          "executable": "$cwd/target/${target}/nginx_libunit_rust"
      }
  }
}
EOF

echo -n "Reloading: "
curl -X GET --unix-socket $socket http://localhost/control/applications/$app_name/restart

echo "Deployed $target target."
