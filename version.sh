#!/bin/bash
set -e

current_version=$(cat Cargo.toml | grep 'version = "*"' | head -1 | awk -F = '{ print $2 }' | sed 's/[", ]//g')

major=$(echo $current_version | awk -F . '{print $1}')
minor=$(echo $current_version | awk -F . '{print $2}')
patch=$(echo $current_version | awk -F . '{print $3}')

if [[ $1 == "--major" ]]; then
  major=$(($major + 1))
elif [[ $1 == "--minor" ]]; then
  minor=$(($minor + 1))
else
  patch=$(($patch + 1))
fi

new_version="$major.$minor.$patch"

sed -i '' -e "s/version = \"${current_version}\"/version = \"${new_version}\"/" Cargo.toml
cargo generate-lockfile

cd python

sed -i '' -e "s/version = \"${current_version}\"/version = \"${new_version}\"/" Cargo.toml
sed -i '' -e "s/VERSION = \"${current_version}\"/VERSION = \"${new_version}\"/" setup.py

cargo generate-lockfile

cd ../

git add --all
git commit -m "v${new_version}"
git tag -a v${new_version} -m v${new_version}
git push origin v${new_version}
git push
