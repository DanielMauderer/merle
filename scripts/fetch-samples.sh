#!/usr/bin/env bash
# Download Canon CR2/CR3 sample RAWs for local dev and tests.
#
# Source: raw.pixls.us — every file below is released under CC0 1.0 (public
# domain). Checksums are the sha256 values published in the site's index
# (https://raw.pixls.us/json/getrepository.php?set=all).
#
# Usage: scripts/fetch-samples.sh [OUT_DIR]   (default: testdata/raw)
set -euo pipefail

BASE_URL="https://raw.pixls.us/data/Canon"
OUT_DIR="${1:-${MERLE_SAMPLES_DIR:-testdata/raw}}"

# local name | remote path (relative to BASE_URL) | sha256
SAMPLES=(
    "canon-eos-r7-raw.CR3|EOS R7/443A0157.CR3|887439cb1a45becd6a5c85fe75ae10e6c520a9f13a8b9ed077c6cf5d7c37700c"
    "canon-eos-r7-craw.CR3|EOS R7/443A0159.CR3|10a18e3f01ca9cd93408496d3853388ef140830548bfeed4237e40f967eb9d5c"
    "canon-eos-7d.CR2|EOS 7D/RAW_CANON_EOS_7D-raw.CR2|b5e47c5fcf7332ac03e0134926f17a338a42e68c1fd7f83e16f45f4b767544e8"
    "canon-eos-100d.CR2|EOS 100D/IMG_3721.CR2|85e62ea5ea658684964a61696660ffccca2a0c60fc1a2a72b30e6357a095f5e7"
)

url_for() {
    # Spaces in camera directory names must be percent-encoded.
    printf '%s/%s' "$BASE_URL" "${1// /%20}"
}

verify() {
    printf '%s  %s\n' "$2" "$1" | sha256sum --check --status
}

mkdir -p "$OUT_DIR"

for entry in "${SAMPLES[@]}"; do
    IFS='|' read -r name remote sha <<<"$entry"
    dest="$OUT_DIR/$name"

    if [[ -f "$dest" ]] && verify "$dest" "$sha"; then
        echo "ok       $name"
        continue
    fi

    echo "fetching $name  <-  $remote"
    curl --fail --location --retry 3 --silent --show-error \
        --output "$dest.part" "$(url_for "$remote")"

    if ! verify "$dest.part" "$sha"; then
        rm -f "$dest.part"
        echo "error: checksum mismatch for $name" >&2
        exit 1
    fi
    mv "$dest.part" "$dest"
done

{
    echo "# Sample RAW files"
    echo
    echo "Downloaded by \`scripts/fetch-samples.sh\` from <https://raw.pixls.us>."
    echo "All files are released under CC0 1.0 (public domain):"
    echo "<https://creativecommons.org/publicdomain/zero/1.0/>"
    echo
    echo "| File | Origin |"
    echo "| ---- | ------ |"
    for entry in "${SAMPLES[@]}"; do
        IFS='|' read -r name remote _ <<<"$entry"
        echo "| \`$name\` | <$(url_for "$remote")> |"
    done
} >"$OUT_DIR/LICENSE.md"

echo "done: $OUT_DIR"
