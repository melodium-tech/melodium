#!/usr/bin/env bash
set -e

while getopts "t:e:v:z:" option
do 
  case $option in
    t)
      TARGET="$OPTARG"
      ;;
    e)
      EXECUTABLE="$OPTARG"
      ;;
    v)
      VERSION="$OPTARG"
      ;;
    z)
      ZIP_FORMAT="$OPTARG"
      ;;
  esac
done

if [[ -z "$TARGET" \
  || -z "$EXECUTABLE" \
  || -z "$VERSION" \
  || -z "$ZIP_FORMAT" \
]]
then
  >&2 echo "All required parameters are not set"
  exit 1
else
  echo "Packaging for target '$TARGET' as $ZIP_FORMAT (executable '$EXECUTABLE') with version '$VERSION'"
fi

TMP_DIR=$(mktemp --directory)
DIR_NAME="melodium-$VERSION"
DIR="$TMP_DIR/$DIR_NAME"

mkdir "$DIR"

cp --recursive melodium/README.md LICENSE melodium/CHANGELOG.md "$DIR"
cp "target/$TARGET/release/$EXECUTABLE" "$DIR"

case "$ZIP_FORMAT" in
    "tgz")
        tar --create --gzip --file "melodium-${VERSION}_${TARGET}.tar.gz" --directory "$TMP_DIR" "$DIR_NAME"
        ;;
    "tbz")
        tar --create --bzip2 --file "melodium-${VERSION}_${TARGET}.tar.bz2" --directory "$TMP_DIR" "$DIR_NAME"
        ;;
    "txz")
        tar --create --xz --file "melodium-${VERSION}_${TARGET}.tar.xz" --directory "$TMP_DIR" "$DIR_NAME"
        ;;
    "zip")
        ( cd "$TMP_DIR" ; zip -r "melodium-${VERSION}_${TARGET}.zip" "$DIR_NAME" )
        mv "$TMP_DIR/melodium-${VERSION}_${TARGET}.zip" .
        ;;
    "sh")
        tar --create --gzip --file installer_files.tar.gz --directory "$TMP_DIR" "$DIR_NAME"
        echo "#!/usr/bin/env bash" > "melodium-${VERSION}_${TARGET}.sh"
        echo "FULL_NAME=\"melodium-${VERSION}\"" >> "melodium-${VERSION}_${TARGET}.sh"
        cat $(pwd)/.gitlab/ci/installer.sh >> "melodium-${VERSION}_${TARGET}.sh"
        cat installer_files.tar.gz >> "melodium-${VERSION}_${TARGET}.sh"
        rm installer_files.tar.gz
        ;;
esac

rm -rf "$TMP_DIR"
