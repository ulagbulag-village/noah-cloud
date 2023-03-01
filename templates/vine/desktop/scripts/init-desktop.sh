#!/bin/bash
# Copyright (c) 2023 Ho Kim (ho.kim@ulagbulag.io). All rights reserved.
# Use of this source code is governed by a GPL-3-style license that can be
# found in the LICENSE file.

# Prehibit errors
set -e
# Verbose
set -x

# Skip if already initialized
LOCKFILE="${HOME}/.kiss-lock"
if [ -f "${LOCKFILE}" ]; then
    exec true
fi

# Install 3rd-party libraries
function download_library() {
    # Configure variables
    URL=$1
    filename="${URL##*/}"
    extension="${filename##*.}"

    # Download a file
    wget "${URL}" -o "${filename}"

    # Extract the file
    case "${extension}" in
    "tar")
        tar -xf "${filename}"
        ;;
    "zip")
        unzip -o -q "${filename}"
        ;;
    *)
        echo "Skipping extracting a file: ${filename}"
        ;;
    esac

    # Remove the original file
    rm -f "${filename}"
}

# ## Fonts
# FONT_HOME="${HOME}/.local/share/fonts"
# mkdir -p "${FONT_HOME}" && pushd "${FONT_HOME}"
#   for url in ${KISS_DESKTOP_FONTS_URL}; do
#     download_library "${url}"
#   done
#   fc-cache -f
# popd

# ## Icons
# ICON_HOME="${HOME}/.local/share/icons"
# mkdir -p "${ICON_HOME}" && pushd "${ICON_HOME}"
#   for url in ${KISS_DESKTOP_ICONS_URL}; do
#     download_library "${url}"
#   done
# popd

# ## Themes
# THEME_HOME="${HOME}/.local/share/themes"
# mkdir -p "${THEME_HOME}" && pushd "${THEME_HOME}"
#   for url in ${KISS_DESKTOP_THEMES_URL}; do
#     download_library "${url}"
#   done
# popd

## ZSH Theme
pushd "${HOME}"
sh -c "$(curl -fsSL "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh")"
git clone --depth=1 \
    "https://github.com/romkatv/powerlevel10k.git" \
    "${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k"

### Cleanup ZSH configurations
rm -rf "${HOME}/.zshrc" "${HOME}/.zshrc.pre-oh-my-zsh"
popd

# Download and install templates
pushd "${HOME}"
git init .
git remote add origin "${KISS_DESKTOP_TEMPLATE_GIT}"
git pull origin "${KISS_DESKTOP_TEMPLATE_GIT_BRANCH}"
popd

# Disable screen blanking
xset -dpms
xset s off

# Finished!
exec touch "${LOCKFILE}"