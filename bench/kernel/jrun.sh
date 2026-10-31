# Author: kuehbajo
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”),
# to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
# and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

# The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

# THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
# INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
# OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF
# OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#!/bin/bash

FILES=(
    binarysearch
    bitcount
    bitonic
    bsort
    complex_updates
    cosf_custom
    cosf_libm
    countnegative
    cubic_custom
    cubic_libm
    deg2rad
    fac
    fft
    filterbank
    fir2dim
    iir
    insertsort
    isqrt
    jfdctint
    lms
    ludcmp
    matrix1
    md5
    minver
    pm_custom
    pm_libm
    prime
    quicksort_custom
    quicksort_libm
    rad2deg
    recursion
    sha
    st_custom
    st_libm
)

if [[ -n "$OPT" ]]; then
    OPT=($OPT)
else
    OPT=(
    	0
    	0
    	0
    	0
    	0
    	1
    )
fi

PROFILES=(O0 O1 O2 O3 Os Oz)

# Directories
SRC_DIR_BASE="../target/thumbv7em-none-eabihf"
DST_DIR_BASE="jrun_bin"
LOG_FILE_BASE="measured"

for i in "${!OPT[@]}"; do
    if [ "${OPT[$i]}" -ne 1 ]; then
        continue
    fi
    
    PROFILE="${PROFILES[$i]}"
    
    # Paths
    SRC_DIR="$SRC_DIR_BASE/$PROFILE"
    DST_DIR="${DST_DIR_BASE}_${PROFILE}"
    LOG_FILE="${LOG_FILE_BASE}_${PROFILE}.txt"
    
    echo "========================================"
    echo "Building profile: $PROFILE"
    echo "========================================"
    
    # Create destination directory
    mkdir -p "$DST_DIR"

    # clears logs on each run
    # > "$LOG_FILE"

    # Loop through selected binaries
    for FILE in "${FILES[@]}"; do
        echo "Building $FILE ($PROFILE)"

        # Build only the selected binary
        cargo ba --bin "$FILE" -F log --profile "$PROFILE"

        # Check build result
        if [ $? -ne 0 ]; then
            echo "Build failed for $FILE ($PROFILE)"
            continue
        fi

        # Verify binary exists
        if [ -f "$SRC_DIR/$FILE" ]; then
            cp "$SRC_DIR/$FILE" "$DST_DIR/$FILE.elf"

            echo "Copied:"
            echo "  $SRC_DIR/$FILE -> $DST_DIR/$FILE.elf"

            # Run benchmark / measurement
            JRun -d XMC4500-1024 --rtt -s "$DST_DIR/$FILE.elf" | tee -a "$LOG_FILE"
        else
            echo "File not found: $SRC_DIR/$FILE"
        fi
        # sleep 1
    done

    # Cleanup
    rm -r "$DST_DIR"
done
