#!/usr/bin/fish

function module_basename --argument path
    set mod_name (basename -s .wasm $path)
    echo $mod_name
end

for arg in $argv
    set mod_name (module_basename $arg)
    set dir_name (dirname $arg)
    wamrc --target=armv7 --enable-simd --opt-level=0 -o "$dir_name/$mod_name.wamr" $arg
end
