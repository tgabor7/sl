#!/bin/bash

TAG=`git tag --sort version:refname | tail -n1`

echo "version=$TAG"

# update_major_tag() {
#   local major=`echo $TAG | cut -d. -f1`
#   local minor=`echo $TAG | cut -d. -f2`
#   local patch=`echo $TAG | cut -d. -f3`
#   local new_major=$((major+1))
#   local new_tag="$new_major.0.0"
#   echo "version=$new_tag"
# }

# update_minor_tag() {
#   local major=`echo $TAG | cut -d. -f1`
#   local minor=`echo $TAG | cut -d. -f2`
#   local patch=`echo $TAG | cut -d. -f3`
#   local new_minor=$((minor+1))
#   local new_tag="$major.$new_minor.0"
#   echo "version=$new_tag"
# }

# update_patch_tag() {
#   local major=`echo $TAG | cut -d. -f1`
#   local minor=`echo $TAG | cut -d. -f2`
#   local patch=`echo $TAG | cut -d. -f3`
#   local new_patch=$((patch+1))
#   local new_tag="$major.$minor.$new_patch"
#   echo "version=$new_tag"
# }

# if [ "$1" == "major" ]; then
#   update_major_tag
# elif [ "$1" == "minor" ]; then
#   update_minor_tag
# elif [ "$1" == "patch" ]; then
#   update_patch_tag
# else
#   echo "Usage: $0 [major|minor|patch]"
#   exit 1
# fi

