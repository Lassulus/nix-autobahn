#!/bin/sh
LIBS_NOT_FOUND=$(ldd "$1" | grep 'not found' | sed 's/ .*//;s/\t//' | sort | uniq)
SELECTED_LIBS=""

# Iterate over every dependencie of binary
for x in $LIBS_NOT_FOUND; do
  candidates=$(nix-locate -w --top-level "$x")

  # Check if one possible condidate has already been selected
  already_selected=false
  for new_lib in $candidates; do
    for slib in $SELECTED_LIBS; do
      if [ "$new_lib" = "$slib" ]; then
        already_selected=true
      fi
    done
  done

  # If it has been already selected skip entry
  if [ $already_selected = true ]; then
    continue
  fi

  # If more then one option possible
  if [ "$(echo "$candidates" | wc -l)" -gt 1 ]; then
    SELECTED_LIBS="$SELECTED_LIBS $(echo "$candidates" | fzf --ansi | sed 's/ .*//')"
    # If only one option viable choose it automatically
  else
    SELECTED_LIBS="$SELECTED_LIBS $(echo "$candidates" | sed 's/ .*//')"
  fi
done
echo "$SELECTED_LIBS"

