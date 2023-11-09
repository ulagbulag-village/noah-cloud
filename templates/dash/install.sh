#!/bin/bash
# Copyright (c) 2023 Ho Kim (ho.kim@ulagbulag.io). All rights reserved.
# Use of this source code is governed by a GPL-3-style license that can be
# found in the LICENSE file.

# Prehibit errors
set -e -o pipefail
# Verbose
set -x

###########################################################
#   Install Opentelemetry                                 #
###########################################################

echo "- Installing Opentelemetry ... "
pushd "opentelemetry" && ./install.sh && popd

# Finished!
echo "Installed!"
