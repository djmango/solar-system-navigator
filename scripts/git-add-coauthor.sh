#!/usr/bin/env bash
# Used by git rebase --exec to append the djmango co-author trailer.
set -euo pipefail

CO_AUTHOR='Co-authored-by: Sulaiman Khan Ghori <28496988+djmango@users.noreply.github.com>'
MSG=$(git log -1 --pretty=%B)

if echo "$MSG" | grep -q 'Co-authored-by: Sulaiman Khan Ghori'; then
  exit 0
fi

git commit --amend -m "${MSG}

${CO_AUTHOR}"
