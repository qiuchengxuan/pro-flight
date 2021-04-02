#!/bin/bash
license=$(mktemp)
cat > $license << EOF
/*
 * This file is part of Proflight.
 *
 * Proflight is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Proflight is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Proflight.  If not, see <http://www.gnu.org/licenses/>.
 */
EOF
tmp=$(mktemp)
root=$(git rev-parse --show-toplevel)
pushd $root
for i in $(git ls-files | grep "\.rs$"); do
    echo "Adding GPL3 header to $i"
    cat $license | cat - $root/$i > $tmp && mv $tmp $root/$i
done
rm -f $license $tmp
popd
