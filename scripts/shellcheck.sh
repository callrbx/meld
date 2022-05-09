NUM_ERRS=0

SHELL_TO_CHECK="sh"
FOLDERS="scripts"

if command -v shellcheck >/dev/null 2>&1; then
    echo "[+] shellcheck found"
else
    echo "[-] command shellcheck not present" >&2
    exit 1
fi

NUM_ERRS=0
for folder in $FOLDERS; do
    echo "[+] checking ${folder} ..."
    shellcheck --shell="${SHELL_TO_CHECK}" "${folder}"/*
    NUM_ERRS=$(( NUM_ERRS + $? ))
    echo "[+] Total: ${NUM_ERRS}"
done

exit $NUM_ERRS
