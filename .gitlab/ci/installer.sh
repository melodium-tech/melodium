
if [ "$(whoami)" != "root" ]; then
        echo "ERROR Install script must be run as root"
        exit -1
fi

mkdir -p /usr/local/lib
mkdir -p /usr/local/bin

ARCHIVE=$(awk '/^__ARCHIVE__/ {print NR + 1; exit 0; }' "${0}")
tail -n+${ARCHIVE} "${0}" | tar xpz -C "/usr/local/lib"

ln -s -f "/usr/local/lib/$FULL_NAME/melodium" "/usr/local/bin/melodium"

exit 0
__ARCHIVE__
