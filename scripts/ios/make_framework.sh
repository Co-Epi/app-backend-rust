set -e

OUTPUT_DIR="." 
PRODUCT_NAME="CoEpiCore"
RUST_REPO_ROOT_DIR="."
PUBLIC_HEADERS_FOLDER_PATH="./src/ios/c_headers/"
LIBRARY_PATH="./target/"

FRAMEWORK_PATH="${OUTPUT_DIR}/${PRODUCT_NAME}.framework"

mkdir -p "${FRAMEWORK_PATH}/Versions/A/Headers"

cp -a "${RUST_REPO_ROOT_DIR}/${PUBLIC_HEADERS_FOLDER_PATH}/" \
      "${FRAMEWORK_PATH}/Versions/A/Headers"

cp -a "${RUST_REPO_ROOT_DIR}/${LIBRARY_PATH}/${PRODUCT_NAME}" \
	  "${FRAMEWORK_PATH}/Versions/A/"

ln -sfh A "${FRAMEWORK_PATH}/Versions/Current"
ln -sfh Versions/Current/Headers "${FRAMEWORK_PATH}/Headers"
ln -sfh "Versions/Current/${PRODUCT_NAME}" \
        "${FRAMEWORK_PATH}/${PRODUCT_NAME}"


