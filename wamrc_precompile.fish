#!/usr/bin/fish

function module_basename --argument path
    set mod_name (basename -s .wasm $path)
    echo $mod_name
end

for arg in $argv
    echo $arg
    set mod_name (module_basename $arg)
    set dir_name (dirname $arg)
    # --target=armv7 --target-abi=gnueabihf
    wamrc --opt-level=0 -o "$mod_name.wamr" $arg
    zip -m "$dir_name/$mod_name-wamr.zip" "$mod_name.wamr"
end
