#!/bin/bash

pushd $OUT_DIR

extracted_folder_name="protoc"
if [ $# -eq 1 ]; then
    extracted_folder_name="$1"
fi

if [ -f "$extracted_folder_name/bin/protoc" ]; then
    exit 0
fi

wget https://github.com/protocolbuffers/protobuf/releases/download/v21.8/protoc-21.8-osx-universal_binary.zip -O protoc_binary.zip

unzip protoc_binary.zip -d "$extracted_folder_name"

popd

echo "DONE!"


