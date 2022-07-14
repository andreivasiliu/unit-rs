#!/bin/bash

set -e

app_name=rustapp
socket="/var/run/control.unit.sock"
target=debug
cwd=$(pwd)
curl="curl --fail-with-body --unix-socket $socket"

if [ "$1" == "release" ]; then
    target=release
fi

echo -n "Configuring: "
$curl -X PUT --data @- http://localhost/config << EOF
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
          "executable": "$cwd/target/${target}/examples/request_info",
          "processes": 4,
      }
  }
}
EOF

echo -n "Reloading: "
$curl -X GET http://localhost/control/applications/$app_name/restart

echo "Deployed $target target."
