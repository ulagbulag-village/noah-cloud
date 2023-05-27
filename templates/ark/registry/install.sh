#!/bin/bash
# Copyright (c) 2023 Ho Kim (ho.kim@ulagbulag.io). All rights reserved.
# Use of this source code is governed by a GPL-3-style license that can be
# found in the LICENSE file.

# Prehibit errors
set -e -o pipefail
# Verbose
set -x

###########################################################
#   Configuration                                         #
###########################################################

# Configure default environment variables
HELM_CHART_DEFAULT="https://helm.twun.io"
NAMESPACE_DEFAULT="ark"

# Set environment variables
HELM_CHART="${HELM_CHART:-$HELM_CHART_DEFAULT}"
NAMESPACE="${NAMESPACE:-$NAMESPACE_DEFAULT}"

###########################################################
#   Configure Helm Channel                                #
###########################################################

echo "- Configuring Helm channel ... "

helm repo add "${NAMESPACE}" "${HELM_CHART}"

###########################################################
#   Install Registry                                      #
###########################################################

echo "- Installing Registry ... "

helm upgrade --install "registry" \
    "${NAMESPACE}/docker-registry" \
    --create-namespace \
    --namespace "${NAMESPACE}" \
    --values "./values.yaml"

###########################################################
#   Install Registry Account                              #
###########################################################

echo "- Installing Registry Account ... "

if
    ! kubectl get secret --no-headers \
        --namespace "${NAMESPACE}" \
        "ark-registry" \
        >/dev/null 2>/dev/null
then
    kubectl create secret docker-registry "ark-registry" \
        --namespace "${NAMESPACE}" \
        --docker-server "http://registry.ark.svc.ops.openark" \
        --docker-username "user" \
        --docker-password "user"
fi

# Finished!
echo "Installed!"
